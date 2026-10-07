"""Original mutable-v1 server rejection; preserves its approved CLI-only support."""
import argparse
import json
from pathlib import Path
import shutil
import sys
import tomllib
from pack_startup_native import serve

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'crates/oh_save/tools'))
from check_save_current import files, git, header, sha
from save_reference import pack_hash, verify_capture


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    source, out = args.capture.resolve(), args.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    record = json.loads((source / 'result.json').read_text(encoding='utf-8'))
    assert record['head'] == git('rev-parse', 'HEAD')
    assert 'evidence_mode' not in record, 'original frozen capture required'
    assert record['fixture_sha256'] == sha(ROOT / 'crates/oh_save/tests/fixtures/m1-v1.ohsave')
    report = json.loads((source / 'capture-1.stdout').read_text(encoding='utf-8')); verify_capture(report)
    policy = tomllib.loads((ROOT / 'crates/oh_data/policies/legacy-validation.toml').read_text(encoding='utf-8'))
    original = next(s for s in policy['sources'] if s['id'] == 'mutable-v1')
    original_entry = next(e for e in policy['entries'] if e['source_id'] == 'mutable-v1')
    packs = out / 'packs'
    shutil.copytree(source / 'run-1/pack', packs / 'testland')
    pack = packs / 'testland'; before = files(pack)
    assert before == original['files'] and pack_hash(pack) == int(original_entry['pack_content_hash'], 16)
    save = source / 'run-1/paused.ohsave'; save_before = sha(save)
    receipts = []
    for force in (False, True):
        name = 'historical-force' if force else 'historical'
        receipt = serve(packs, out, name, save, force, False, 'testland_name')
        errors = (out / f'{name}.stderr').read_text(encoding='utf-8')
        assert receipt['passed'] and receipt['native_exit'] == 1 and receipt['http_status'] is None and receipt['ws_status'] is None
        assert not receipt['ctrl_c_sent']
        assert 'missing ko message value testland_name' in errors and 'missing en message value testland_name' in errors
        assert before == files(pack) and save_before == sha(save)
        receipts.append(receipt)
    (out / 'result.json').write_text(json.dumps(dict(head=record['head'], dirty=bool(git('status', '--porcelain')),
        input_header=header(save), source_files=before,
        save_sha256=save_before, source_preserved=True, save_preserved=True, receipts=receipts), indent=2)+'\n')
    print(json.dumps(dict(total=2, native_exits=[r['native_exit'] for r in receipts], http_ws_absent=True)))


if __name__ == '__main__':
    main()
