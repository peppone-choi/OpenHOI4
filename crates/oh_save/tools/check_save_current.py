#!/usr/bin/env python3
"""Separate current M1 evidence. Never replaces the frozen v1 capture/comparer."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import sys
import tomllib
from save_reference import canonical, fnv, pack_hash, signed, string, varint, verify_capture, verify_ledger

ROOT = Path(__file__).resolve().parents[3]
RUNNERS = ('ubuntu-latest', 'windows-latest', 'macos-latest')
PLATFORMS = dict(zip(RUNNERS, ('Linux', 'Windows', 'Darwin')))
MODE = 'current-m1-v1'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(['git', '--no-optional-locks', *args], cwd=ROOT, text=True).strip()


def source_identity():
    index = Path(git('rev-parse', '--git-path', 'index'))
    if not index.is_absolute():
        index = ROOT / index
    return dict(head=git('rev-parse', 'HEAD'), status=git('status', '--porcelain'),
                tracked_index_sha256=hashlib.sha256(git('ls-files', '-s').encode()).hexdigest(),
                raw_index_sha256=sha(index))


def save_source_boundary(folder, name, identity):
    index = Path(git('rev-parse', '--git-path', 'index'))
    if not index.is_absolute():
        index = ROOT / index
    raw = index.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == identity['raw_index_sha256']
    (folder / f'source-{name}.index').write_bytes(raw)
    (folder / f'source-{name}.json').write_text(json.dumps(identity, indent=2)+'\n')


def files(root):
    found = []
    def walk(directory):
        for path in sorted(directory.iterdir()):
            stat = path.lstat()
            assert not path.is_symlink() and not getattr(stat, 'st_file_attributes', 0) & 0x400
            if path.is_dir():
                walk(path)
            else:
                found.append(dict(path=path.relative_to(root).as_posix(), bytes=stat.st_size, sha256=sha(path)))
    walk(root)
    return sorted(found, key=lambda f: f['path'])


def effective_defines(root):
    merged = tomllib.loads((root / 'defines.toml').read_text(encoding='utf-8'))
    overlay = tomllib.loads((root / 'scenarios/m1/defines.toml').read_text(encoding='utf-8'))
    for system, items in overlay.items():
        merged.setdefault(system, {}).update(items)
    # Independent tagged Postcard encoding for this actual integer-only M1 input.
    # A future coefficient type must extend the reference, never silently coerce floats.
    def number(n):
        assert type(n) is int and -(1 << 63) <= n < 1 << 63, 'integer M1 defines reference required'
        return b'\x00' + signed(n)
    def value(v):
        return b'\x01' + varint(len(v)) + b''.join(number(n) for n in v) if isinstance(v, list) else b'\x00' + number(v)
    encoded = varint(len(merged))
    for system, items in sorted(merged.items()):
        encoded += string(system) + varint(len(items))
        for item, v in sorted(items.items()):
            encoded += string(item) + value(v)
    return merged, int(fnv(encoded), 16)


def header(path):
    raw = path.read_bytes()
    limits = tomllib.loads((ROOT / 'crates/oh_save/defines.toml').read_text(encoding='utf-8'))['save']
    assert 10 < len(raw) <= limits['file_max_bytes'] and raw[:4] == b'OHSV'
    assert int.from_bytes(raw[4:6], 'little') == 1
    length = int.from_bytes(raw[6:10], 'little')
    assert 0 < length <= limits['header_max_bytes'] and 10 + length < len(raw)
    data = raw[10:10 + length]; offset = 0
    def byte():
        nonlocal offset
        assert offset < len(data)
        n = data[offset]; offset += 1; return n
    def uint(bits=64):
        n = 0; start = offset
        for shift in range(0, bits, 7):
            b = byte(); n |= (b & 127) << shift
            if b < 128:
                assert n < 1 << bits and data[start:offset] == varint(n)
                return n
        raise AssertionError('noncanonical/overflow header integer')
    def text():
        nonlocal offset
        n = uint(); assert n <= limits['string_max_bytes'] and offset + n <= len(data)
        result = data[offset:offset + n].decode(); offset += n; return result
    engine = text(); version = uint(16); scenario = text(); count = uint()
    assert count == 1
    packs = [dict(id=text(), version=text(), content_hash=uint()) for _ in range(count)]
    date = dict(year=uint(32), month=byte(), day=byte())
    tick, seed, state_hash = uint(), uint(), uint()
    players_count = uint(); assert players_count <= limits['map_entries_max']
    players = [uint(16) for _ in range(players_count)]
    saved_at = uint(); saved_at = (saved_at >> 1) ^ -(saved_at & 1)
    tag = byte(); assert tag in (0, 1)
    definitions = uint() if tag else None
    defines = uint(); assert offset == len(data) and version == 1
    return dict(engine_version=engine, format_version=version, scenario_id=scenario, packs=packs,
                game_date=date, tick=tick, seed=seed, state_hash=f'{state_hash:016x}',
                player_nations=players, saved_at_utc=saved_at, definitions_hash=definitions,
                effective_defines_hash=defines)


def paused_dto(report):
    dto = copy.deepcopy(report['split'])
    dto['state'].update(paused=True, speed=5)
    dto['queue'] = [q for q in dto['queue'] if q['tick'] > 48]
    return dto


def check_header(value, dto, pack, defines_hash):
    assert value['engine_version'] == tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['workspace']['package']['version']
    assert value['format_version'] == 1 and value['scenario_id'] == dto['state']['scenario'] == 'm1'
    assert value['packs'] == [pack]
    assert value['game_date'] == dto['state']['date'] and value['tick'] == dto['state']['tick']
    assert value['seed'] == dto['state']['seed'] == 7
    assert value['state_hash'] == fnv(canonical(dto))
    assert value['definitions_hash'] == dto['world']['definitions_hash']
    assert value['effective_defines_hash'] == defines_hash
    assert value['saved_at_utc'] == 0 and value['player_nations'] == []


def run(args, folder, name, native=False):
    process = subprocess.Popen(args, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = process.communicate()
    (folder / f'{name}.stdout').write_bytes(stdout)
    (folder / f'{name}.stderr').write_bytes(stderr)
    receipt = dict(command=args, cwd=str(ROOT), pid=process.pid, exit=process.returncode, head=git('rev-parse', 'HEAD'))
    if native:
        receipt['exe_sha256'] = sha(Path(args[0]))
    (folder / f'{name}.command.json').write_text(json.dumps(receipt, indent=2) + '\n')
    assert process.returncode == 0, f'{name}: native exit {process.returncode}: {stderr.decode("utf-8", errors="replace")}'
    return stdout.decode('utf-8')


def capture(folder):
    folder = folder.resolve(); folder.mkdir(parents=True, exist_ok=False)
    before = source_identity()
    save_source_boundary(folder, 'before', before)
    run(['cargo', 'build', '-p', 'oh_save', '--example', 'save_fixture', '--locked'], folder, 'build-helper')
    run(['cargo', 'build', '-p', 'oh_cli', '--locked'], folder, 'build-cli')
    suffix = '.exe' if platform.system() == 'Windows' else ''
    helper = ROOT / f'target/debug/examples/save_fixture{suffix}'
    cli = ROOT / f'target/debug/oh_cli{suffix}'
    runs = []
    for index in (1, 2):
        out = folder / f'run-{index}'
        report = json.loads(run([str(helper), 'capture-current', str(out)], folder, f'capture-{index}', True))
        inputs = files(out / 'pack'); save_sha, paused_sha = sha(out / 'saved.ohsave'), sha(out / 'paused.ohsave')
        run([str(helper), 'resume', str(out / 'pack'), str(out / 'saved.ohsave')], folder, f'restart-{index}', True)
        run([str(cli), 'resume', '--load', str(out / 'saved.ohsave'), '--pack', str(out / 'pack'), '--days', '2', '--hash-out'], folder, f'cli-resume-{index}', True)
        assert inputs == files(out / 'pack') and save_sha == sha(out / 'saved.ohsave') and paused_sha == sha(out / 'paused.ohsave')
        defines, defines_hash = effective_defines(out / 'pack')
        runs.append(dict(pack=report['pack'], source_files=inputs, save_sha256=save_sha, paused_sha256=paused_sha,
                         header=header(out / 'saved.ohsave'), paused_header=header(out / 'paused.ohsave'),
                         effective_defines=defines, effective_defines_hash=defines_hash,
                         split_hash=report['split_hash'], resumed_hash=report['continuous_hash'],
                         split=report['split'], continuous=report['continuous'],
                         canonical_hex=canonical(report['split']).hex(),
                         continuous_canonical_hex=canonical(report['continuous']).hex(),
                         canonical_sha256=hashlib.sha256(canonical(report['split'])).hexdigest()))
    after = source_identity(); assert before == after, 'source/index changed during current capture'
    save_source_boundary(folder, 'after', after)
    result = dict(evidence_schema=1, evidence_mode=MODE, head=before['head'], dirty=bool(before['status']),
                  checkout_root=str(ROOT), platform=platform.system(), source=dict(before=before, after=after),
                  helper_sha256=sha(helper), cli_sha256=sha(cli), runs=runs,
                  original_fixture_sha256=sha(ROOT / 'crates/oh_save/tests/fixtures/m1-v1.ohsave'),
                  identity_difference='current pack includes manifest name translations; frozen original source remains unchanged')
    (folder / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    verify_folder(folder, require_clean=False)
    print(json.dumps(result))


def normalized(path):
    return str(path).replace('\\', '/')


def command(folder, result, stage, index):
    name = f'{stage}-{index}'
    receipt = json.loads((folder / f'{name}.command.json').read_text(encoding='utf-8'))
    assert type(receipt['exit']) is int and receipt['exit'] == 0
    assert type(receipt['pid']) is int and receipt['pid'] > 0
    assert receipt['head'] == result['head'] and receipt['cwd'] == result['checkout_root']
    assert re.fullmatch('[0-9a-f]{64}', receipt['exe_sha256'])
    (folder / f'{name}.stderr').read_bytes(); (folder / f'{name}.stdout').read_bytes()
    argv = receipt['command']; executable = normalized(argv[0]).split('/')[-1].removesuffix('.exe')
    if stage == 'capture':
        assert executable == 'save_fixture' and len(argv) == 3 and argv[1] == 'capture-current'
        assert normalized(argv[2]).endswith(f'/run-{index}')
    elif stage == 'restart':
        assert executable == 'save_fixture' and len(argv) == 4 and argv[1] == 'resume'
        assert normalized(argv[2]).endswith(f'/run-{index}/pack') and normalized(argv[3]).endswith(f'/run-{index}/saved.ohsave')
    else:
        assert executable == 'oh_cli' and len(argv) == 9
        assert argv[1:3] == ['resume', '--load'] and argv[4] == '--pack' and argv[-3:] == ['--days', '2', '--hash-out']
        assert normalized(argv[3]).endswith(f'/run-{index}/saved.ohsave') and normalized(argv[5]).endswith(f'/run-{index}/pack')
    assert receipt['exe_sha256'] == result['cli_sha256' if stage == 'cli-resume' else 'helper_sha256']
    return receipt


def verify_folder(folder, require_clean=True):
    result = json.loads((folder / 'result.json').read_text(encoding='utf-8'))
    assert result['evidence_schema'] == 1 and result['evidence_mode'] == MODE
    assert re.fullmatch('[0-9a-f]{40}', result['head']) and result['head'] == git('rev-parse', 'HEAD')
    assert type(result['dirty']) is bool
    assert result['source']['before'] == result['source']['after']
    assert result['source']['before']['head'] == result['head']
    for boundary in ('before', 'after'):
        assert json.loads((folder / f'source-{boundary}.json').read_text(encoding='utf-8')) == result['source'][boundary]
        assert sha(folder / f'source-{boundary}.index') == result['source'][boundary]['raw_index_sha256']
    assert result['dirty'] == bool(result['source']['before']['status'])
    if require_clean:
        assert result['dirty'] is False, 'clean exact HEAD required'
    assert len(result['runs']) == 2
    for field in ('helper_sha256', 'cli_sha256', 'original_fixture_sha256'):
        assert re.fullmatch('[0-9a-f]{64}', result[field])
    assert result['original_fixture_sha256'] == sha(ROOT / 'crates/oh_save/tests/fixtures/m1-v1.ohsave')
    for stage in ('build-helper', 'build-cli'):
        receipt = json.loads((folder / f'{stage}.command.json').read_text(encoding='utf-8'))
        assert type(receipt['exit']) is int and receipt['exit'] == 0
        assert receipt['head'] == result['head'] and receipt['cwd'] == result['checkout_root']
        (folder / f'{stage}.stdout').read_bytes(); (folder / f'{stage}.stderr').read_bytes()
    reports = []; pids = []
    for index, record in enumerate(result['runs'], 1):
        out = folder / f'run-{index}'
        report = json.loads((folder / f'capture-{index}.stdout').read_text(encoding='utf-8'))
        restart = json.loads((folder / f'restart-{index}.stdout').read_text(encoding='utf-8'))
        assert json.loads((out / 'capture.json').read_text(encoding='utf-8')) == report
        verify_capture(report)
        assert record['split'] == report['split'] and record['continuous'] == report['continuous']
        assert record['canonical_hex'] == canonical(report['split']).hex()
        assert record['continuous_canonical_hex'] == canonical(report['continuous']).hex()
        assert report['pack'] == record['pack'] and report['pack']['content_hash'] == pack_hash(out / 'pack')
        manifest = tomllib.loads((out / 'pack/manifest.toml').read_text(encoding='utf-8'))
        assert (manifest['id'], manifest['version']) == (report['pack']['id'], report['pack']['version'])
        assert files(out / 'pack') == record['source_files']
        defines, defines_hash = effective_defines(out / 'pack')
        assert defines == record['effective_defines'] and defines_hash == record['effective_defines_hash']
        assert report['split']['config'] == dict(initial_speed=defines['time']['initial_speed'], speed_ms_per_tick=defines['time']['speed_ms_per_tick'])
        assert sha(out / 'saved.ohsave') == record['save_sha256'] != result['original_fixture_sha256']
        assert sha(out / 'paused.ohsave') == record['paused_sha256']
        actual_header, paused_header = header(out / 'saved.ohsave'), header(out / 'paused.ohsave')
        assert actual_header == record['header'] and paused_header == record['paused_header']
        check_header(actual_header, report['split'], report['pack'], defines_hash)
        check_header(paused_header, paused_dto(report), report['pack'], defines_hash)
        assert restart['initial'] == report['split'] and restart['resumed'] == report['continuous']
        for dto in (restart['initial'], restart['resumed']):
            verify_ledger(dto)
        assert restart['initial_hash'] == report['split_hash'] == record['split_hash'] == fnv(canonical(restart['initial']))
        assert restart['resumed_hash'] == report['continuous_hash'] == record['resumed_hash'] == fnv(canonical(restart['resumed']))
        assert hashlib.sha256(canonical(report['split'])).hexdigest() == record['canonical_sha256']
        for stage in ('capture', 'restart', 'cli-resume'):
            receipt = command(folder, result, stage, index); pids.append(receipt['pid'])
            if stage in ('capture', 'restart'):
                assert receipt['pid'] == (report if stage == 'capture' else restart)['pid']
        assert (folder / f'cli-resume-{index}.stdout').read_text(encoding='utf-8').strip() == report['continuous_hash']
        reports.append(report)
    assert len(set(pids)) == 6, 'six separate native processes required'
    assert result['runs'][0] == result['runs'][1], 'repeated save/paused/source/header identity required'
    assert reports[0]['split'] == reports[1]['split'] and reports[0]['continuous'] == reports[1]['continuous']
    return result


def verify_servers(folder, result):
    pids = []
    for force, name in ((False, 'server'), (True, 'server-force')):
        directory = folder / name
        receipt = json.loads((directory / 'result.json').read_text(encoding='utf-8'))
        record = result['runs'][0]
        assert receipt['head'] == receipt['capture_head'] == result['head']
        assert receipt['capture_mode'] == MODE and receipt['dirty'] is False
        assert receipt['server_cwd'] == result['checkout_root']
        assert receipt['force'] is force and receipt['http_status'] == 200
        assert type(receipt['server_exit']) is int and receipt['server_exit'] == 0
        assert type(receipt['query_exit']) is int and receipt['query_exit'] == 0
        assert receipt['save_preserved'] is True and receipt['source_preserved'] is True
        assert receipt['save_sha256'] == record['paused_sha256']
        assert receipt['input_header'] == record['paused_header'] and receipt['input_files'] == record['source_files']
        assert files(directory / 'packs/testland') == record['source_files']
        assert receipt['pack_hash'] == f'{record["pack"]["content_hash"]:016x}'
        for field in ('exe_sha256', 'served_js_sha256', 'built_js_sha256'):
            assert re.fullmatch('[0-9a-f]{64}', receipt[field])
        assert receipt['served_js_sha256'] == receipt['built_js_sha256']
        argv = receipt['server_command']
        assert normalized(argv[0]).split('/')[-1].removesuffix('.exe') == 'oh_server'
        assert argv[1] == '--port' and argv[3] == '--pack-root' and argv[5] == '--load-save'
        assert normalized(argv[6]).endswith('/run-1/paused.ohsave')
        assert normalized(argv[4]).endswith(f'/{name}/packs')
        assert argv[7:] == (['--force'] if force else [])
        assert receipt['query_command'] == ['node', 'crates/oh_server/tests/save_query.cjs', receipt['url']]
        for file in ('server.stdout', 'server.stderr', 'query.stderr'):
            (directory / file).read_bytes()
        startup = (directory / 'server.stdout').read_text(encoding='utf-8')
        assert receipt['url'] in startup and receipt['pack_hash'] in startup
        query = json.loads((directory / 'query.stdout').read_text(encoding='utf-8'))
        assert query == receipt['query'] and query['welcome']['accepted'] is True
        assert query['welcome']['engine_version'] == record['paused_header']['engine_version']
        assert query['welcome']['packs'] == [dict(id=record['pack']['id'], version=record['pack']['version'], hash=receipt['pack_hash'])]
        assert query['welcome']['packs'][0]['hash'] == receipt['pack_hash']
        assert query['snapshot']['state'] == dict(date='2000-03-01', hour=0, tick='48', paused=True, speed=5)
        assert query['query']['supported'] is True and query['query']['world']['tick'] == '48'
        assert len(query['query']['world']['nations']) == 2 and len(query['query']['world']['provinces']) == 6
        assert query['command']['accepted'] is True
        assert query['state']['state']['tick'] == '48' and query['state']['state']['paused'] is True and query['state']['state']['speed'] == 2
        assert type(receipt['server_pid']) is int and receipt['server_pid'] > 0
        assert type(query['pid']) is int and query['pid'] > 0
        assert receipt['server_pid'] != query['pid']
        pids.append(receipt['server_pid'])
    assert len(set(pids)) == 2


def compare(root):
    expected = {f'current-save-{runner}' for runner in RUNNERS}
    assert {p.name for p in root.iterdir()} == expected, 'exactly all three current OS artifacts required'
    results = []
    for runner in RUNNERS:
        folder = root / f'current-save-{runner}'
        result = verify_folder(folder)
        verify_servers(folder, result)
        assert result['platform'] == PLATFORMS[runner]
        results.append(result)
    assert all(r['runs'] == results[0]['runs'] for r in results), 'current state/save/pack/header identity differs across OS'
    print(json.dumps(dict(head=results[0]['head'], evidence_mode=MODE, artifact_count=3, runs=results[0]['runs'])))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='mode', required=True)
    sub.add_parser('capture').add_argument('--out', type=Path, required=True)
    sub.add_parser('compare').add_argument('--root', type=Path, required=True)
    args = parser.parse_args()
    try:
        capture(args.out) if args.mode == 'capture' else compare(args.root)
    except (AssertionError, KeyError, TypeError, ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f'current save determinism: {error}', file=sys.stderr)
        sys.exit(1)
