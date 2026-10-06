#!/usr/bin/env python3
"""Capture actual native M1 saves/restarts; require all three exact-HEAD OS results."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys
from save_reference import verify_capture, canonical, fnv, verify_ledger, pack_hash

RUNNERS = ('ubuntu-latest', 'windows-latest', 'macos-latest')
ROOT = Path(__file__).resolve().parents[3]

def run(args, folder, name):
    result = subprocess.run(args, cwd=ROOT, text=True, capture_output=True)
    (folder / f'{name}.stdout').write_text(result.stdout, encoding='utf-8')
    (folder / f'{name}.stderr').write_text(result.stderr, encoding='utf-8')
    evidence = {'command': args, 'exit': result.returncode}
    (folder / f'{name}.command.json').write_text(json.dumps(evidence), encoding='utf-8')
    if result.returncode:
        raise ValueError(f'{name}: exit {result.returncode}: {result.stderr}')
    return result.stdout

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def capture(folder):
    folder = folder.resolve()
    folder.mkdir(parents=True, exist_ok=True)
    marker = folder / 'result.json'
    marker.unlink(missing_ok=True)
    run(['cargo', 'build', '-p', 'oh_save', '--example', 'save_fixture', '--locked'], folder, 'build-helper')
    run(['cargo', 'build', '-p', 'oh_cli', '--locked'], folder, 'build-cli')
    suffix = '.exe' if platform.system() == 'Windows' else ''
    helper = ROOT / f'target/debug/examples/save_fixture{suffix}'
    cli = ROOT / f'target/debug/oh_cli{suffix}'
    reports, restarts, hashes = [], [], []
    for index in (1, 2):
        out = folder / f'run-{index}'
        report = json.loads(run([str(helper), 'capture', str(out)], folder, f'capture-{index}'))
        verify_capture(report)
        assert pack_hash(out / 'pack') == report['pack']['content_hash']
        restart = json.loads(run([str(helper), 'resume', str(out / 'pack'), str(out / 'saved.ohsave')], folder, f'restart-{index}'))
        assert report['pid'] != restart['pid'], 'fresh native processes required'
        assert restart['initial'] == report['split']
        assert restart['resumed'] == report['continuous']
        assert restart['initial_hash'] == report['split_hash']
        assert restart['resumed_hash'] == report['continuous_hash']
        verify_ledger(restart['resumed'])
        assert fnv(canonical(restart['resumed'])) == restart['resumed_hash']
        output = run([str(cli), 'resume', '--load', str(out / 'saved.ohsave'), '--pack', str(out / 'pack'), '--days', '2', '--hash-out'], folder, f'cli-resume-{index}')
        assert output.strip() == report['continuous_hash']
        reports.append(report)
        restarts.append(restart)
        hashes.append(sha(out / 'saved.ohsave'))
    assert hashes[0] == hashes[1]
    committed = ROOT / 'crates/oh_save/tests/fixtures/m1-v1.ohsave'
    assert hashes[0] == sha(committed), 'fresh save bytes must match committed fixture'
    expected = json.loads((committed.parent / 'm1-v1.expected.json').read_text())
    assert hashes[0] == expected['fixture_sha256']
    assert reports[0]['split_hash'] == expected['split_hash']
    assert reports[0]['continuous_hash'] == expected['resumed_hash']
    assert hashlib.sha256(canonical(reports[0]['split'])).hexdigest() == expected['canonical_sha256']
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    dirty = subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True)
    result = {'head': head, 'dirty': bool(dirty), 'platform': platform.system(), 'fixture_sha256': hashes[0],
              'fixture_repeats': hashes, 'split_hash': reports[0]['split_hash'],
              'resumed_hash': reports[0]['continuous_hash'], 'helper_sha256': sha(helper),
              'cli_sha256': sha(cli), 'pack_hash': reports[0]['pack']['content_hash'],
              'canonical_sha256': hashlib.sha256(canonical(reports[0]['split'])).hexdigest()}
    marker.write_text(json.dumps(result, indent=2), encoding='utf-8')
    print(json.dumps(result))

def compare(root):
    expected = {f'save-{os}' for os in RUNNERS}
    if {p.name for p in root.iterdir()} != expected:
        raise ValueError('exactly all three OS artifacts required')
    results = []
    for runner in RUNNERS:
        folder = root / f'save-{runner}'
        result = json.loads((folder / 'result.json').read_text())
        assert result['dirty'] is False, 'clean exact-HEAD capture required'
        for index in (1, 2):
            report = json.loads((folder / f'capture-{index}.stdout').read_text())
            restart = json.loads((folder / f'restart-{index}.stdout').read_text())
            verify_capture(report)
            assert pack_hash(folder / f'run-{index}/pack') == report['pack']['content_hash']
            assert report['pid'] != restart['pid']
            assert restart['initial'] == report['split'] and restart['resumed'] == report['continuous']
            for stage in ('capture', 'restart', 'cli-resume'):
                command = json.loads((folder / f'{stage}-{index}.command.json').read_text())
                assert command['exit'] == 0
                argv = command['command']
                if stage == 'cli-resume':
                    assert argv[1:3] == ['resume', '--load'] and argv[-3:] == ['--days', '2', '--hash-out']
                    assert (folder / f'cli-resume-{index}.stdout').read_text().strip() == result['resumed_hash']
                else:
                    assert argv[1] == ('capture' if stage == 'capture' else 'resume')
            save = folder / f'run-{index}/saved.ohsave'
            assert sha(save) == result['fixture_sha256']
            assert report['split_hash'] == restart['initial_hash'] == result['split_hash']
            assert report['continuous_hash'] == restart['resumed_hash'] == result['resumed_hash']
        assert result['fixture_repeats'] == [result['fixture_sha256']] * 2
        results.append(result)
    current = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    for field in ('head', 'fixture_sha256', 'split_hash', 'resumed_hash', 'pack_hash', 'canonical_sha256'):
        assert len({str(r[field]) for r in results}) == 1, f'OS mismatch: {field}'
    assert results[0]['head'] == current, 'exact checkout HEAD required'
    assert {r['platform'] for r in results} == {'Linux', 'Windows', 'Darwin'}
    print(json.dumps({field: results[0][field] for field in ('head', 'fixture_sha256', 'split_hash', 'resumed_hash')}))

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='mode', required=True)
    sub.add_parser('capture').add_argument('--out', type=Path, required=True)
    sub.add_parser('compare').add_argument('--root', type=Path, required=True)
    args = parser.parse_args()
    try:
        capture(args.out) if args.mode == 'capture' else compare(args.root)
    except (AssertionError, KeyError, ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f'save determinism: {error}', file=sys.stderr)
        sys.exit(1)
