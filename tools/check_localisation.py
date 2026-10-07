#!/usr/bin/env python3
"""REQ-LOC-03: static client uses + locale parity through the real Fluent AST.

Dynamic DTO names are checked by pack validate. Computed client keys cannot be
proved by static scanning; catalog parity checks their shipped values in both
languages. Literal occurrences conservatively count as usage (unused warnings).
"""
import argparse
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def collect(sources, packs):
    uses = []
    literal = re.compile(r'''["']([a-z][a-z0-9_-]*)["']''')
    direct = re.compile(r'''\bt\s*\(\s*["']([a-z][a-z0-9_-]*)["']''')
    for path in sorted(sources.rglob('*')):
        if path.suffix not in ('.ts', '.tsx') or '.test.' in path.name or 'proto' in path.parts:
            continue
        text = path.read_text(encoding='utf-8')
        for match in literal.finditer(text):
            prefix = text[:match.start(1)]
            uses.append(dict(key=match.group(1), path=str(path), line=prefix.count('\n')+1,
                             column=len(prefix.rsplit('\n', 1)[-1])+1, required=False))
        for match in direct.finditer(text):
            prefix = text[:match.start(1)]
            uses.append(dict(key=match.group(1), path=str(path), line=prefix.count('\n')+1,
                             column=len(prefix.rsplit('\n', 1)[-1])+1, required=True))
    # Actual registered DTO generated names: oh_proto uses these prefixes.
    import tomllib
    for pack in packs:
        for filename, kind, prefix in [('resources.toml', 'resource', 'resource-'),
                                      ('buildings.toml', 'building', 'building-'),
                                      ('terrain.toml', 'terrain', '')]:
            path = pack/'common'/filename
            if path.exists():
                for entry in tomllib.loads(path.read_text(encoding='utf-8'))[kind]:
                    uses.append(dict(key=prefix+entry['id'], path=str(path), line=1, column=1, required=True))
    return uses


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sources', type=Path, default=ROOT/'client/src')
    parser.add_argument('--catalogs', type=Path, default=ROOT/'client/public/locales')
    parser.add_argument('--pack', action='append', type=Path)
    parser.add_argument('--out', type=Path, default=ROOT/'target/evidence/WP-24/client-uses.json')
    args = parser.parse_args()
    # Only optional occurrences already present in either catalog count as use;
    # direct t('key') calls remain required even when absent from both.
    keys = set()
    for lang in ('ko', 'en'):
        keys.update(re.findall(r'^([a-zA-Z][\w-]*)\s*=', (args.catalogs/f'{lang}.ftl').read_text(encoding='utf-8'), re.M))
    uses = [{k: v for k, v in u.items() if k != 'required'}
            for u in collect(args.sources, args.pack or [ROOT/'data/packs/testland'])
            if u['required'] or u['key'] in keys]
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(uses, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    return subprocess.run(['cargo', 'run', '-p', 'oh_data', '--locked', '--example', 'catalog_check', '--',
                           str(args.catalogs/'ko.ftl'), str(args.catalogs/'en.ftl'), str(args.out)], cwd=ROOT).returncode


if __name__ == '__main__':
    raise SystemExit(main())
