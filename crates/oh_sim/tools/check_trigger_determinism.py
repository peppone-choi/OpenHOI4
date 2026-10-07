#!/usr/bin/env python3
"""Actual WP13 capture/repeat/fresh resume and mandatory three-OS comparison.
Independent integer-only Postcard/FNV reference; original checkers stay frozen.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'crates/oh_save/tools'))
from save_reference import varint, signed, string, optional, sequence, date, canonical, fnv, pack_hash, verify_ledger
from check_save_current import effective_defines
RUNNERS = {'ubuntu-latest': 'Linux', 'windows-latest': 'Windows', 'macos-latest': 'Darwin'}
CASES = ('active', 'split', 'ended', 'empty', 'initial', 'paused_condition')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def strict_json(path):
    def unique(items):
        out = {}
        for key, value in items:
            assert key not in out, f'duplicate JSON key {key}'
            out[key] = value
        return out
    return json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=unique)


def git(*args):
    return subprocess.check_output(['git', '--no-optional-locks', *args], cwd=ROOT, text=True, encoding="utf-8").strip()


def identity():
    index = Path(git('rev-parse', '--git-path', 'index'))
    if not index.is_absolute():
        index = ROOT / index
    tracked = git('ls-files').splitlines()
    return {'head': git('rev-parse', 'HEAD'), 'status': git('status', '--porcelain'),
            'raw_index_sha256': sha(index), 'tracked': {p: sha(ROOT / p) for p in tracked},
            'index': git('ls-files', '-s'), 'diff': git('diff', '--binary'),
            'cached': git('diff', '--cached', '--binary')}


def files(folder):
    return {p.relative_to(folder).as_posix(): sha(p) for p in sorted(folder.rglob('*')) if p.is_file()}


def header(path):
    raw = path.read_bytes()
    limits = tomllib.loads((ROOT / 'crates/oh_save/defines.toml').read_text(encoding='utf-8'))['save']
    assert raw[:4] == b'OHSV' and int.from_bytes(raw[4:6], 'little') == 4
    assert 10 < len(raw) <= limits['file_max_bytes']
    length = int.from_bytes(raw[6:10], 'little')
    assert 0 < length <= limits['header_max_bytes'] and 10 + length < len(raw)
    data, at = raw[10:10 + length], 0
    def byte():
        nonlocal at
        value = data[at]
        at += 1
        return value
    def uint(bits=64):
        start, value = at, 0
        for shift in range(0, bits, 7):
            b = byte()
            value |= (b & 127) << shift
            if b < 128:
                assert value < 1 << bits and data[start:at] == varint(value)
                return value
        raise AssertionError('header integer overflow')
    def text():
        nonlocal at
        n = uint()
        assert n <= limits['string_max_bytes'] and at + n <= len(data)
        value = data[at:at+n].decode('utf-8')
        at += n
        return value
    value = {'engine_version': text(), 'format_version': uint(16), 'scenario_id': text()}
    n = uint()
    assert n == 1
    value['packs'] = [{'id': text(), 'version': text(), 'content_hash': uint()} for _ in range(n)]
    value['game_date'] = {'year': uint(32), 'month': byte(), 'day': byte()}
    value.update(tick=uint(), seed=uint(), state_hash=f'{uint():016x}')
    n = uint()
    assert n <= limits['map_entries_max']
    value['player_nations'] = [uint(16) for _ in range(n)]
    saved = uint()
    value['saved_at_utc'] = (saved >> 1) ^ -(saved & 1)
    tag = byte()
    assert tag in (0, 1)
    value['definitions_hash'] = uint() if tag else None
    value['effective_defines_hash'] = uint()
    assert at == len(data) and value['format_version'] == 4
    return value


def ast(value, effect=False):
    assert len(value) == 1
    key, arg = next(iter(value.items()))
    if effect:
        keys = ('scope', 'if', 'add_stability', 'add_mobilization', 'add_political_capital',
                'set_law', 'add_building', 'transfer_state', 'declare_war', 'add_manpower',
                'add_equipment', 'set_flag', 'clear_flag', 'end_scenario')
        tag = varint(keys.index(key))
        if key == 'scope':
            return tag + string(arg['target']) + sequence(arg['effects'], lambda e: ast(e, True))
        if key == 'if':
            return tag + ast(arg['condition']) + sequence(arg['then'], lambda e: ast(e, True)) + sequence(arg['else'], lambda e: ast(e, True))
        if key == 'add_building':
            return tag + varint(arg['state']) + string(arg['building']) + signed(arg['levels'])
        if key == 'transfer_state':
            return tag + varint(arg['state']) + string(arg['nation'])
        if key == 'add_equipment':
            return tag + string(arg['equipment']) + signed(arg['amount'])
        return tag + (signed(arg) if key == 'add_manpower' else string(arg))
    keys = ('all', 'any', 'not', 'date_gte', 'at_war', 'at_war_with', 'nation_is', 'stability',
            'mobilization', 'political_capital', 'has_law', 'controls_province', 'owns_state',
            'has_flag', 'ideology_support', 'chance')
    tag = varint(keys.index(key))
    if key in ('all', 'any'):
        return tag + sequence(arg, ast)
    if key == 'not':
        return tag + ast(arg)
    if key == 'at_war':
        return tag + bytes([int(arg)])
    if key in ('controls_province', 'owns_state'):
        return tag + varint(arg)
    if key in ('stability', 'mobilization', 'political_capital'):
        return tag + compare(arg)
    if key == 'ideology_support':
        return tag + string(arg['ideology']) + compare(arg['value'])
    return tag + string(arg)


def compare(v):
    assert len(v) == 1
    k, a = next(iter(v.items()))
    return varint(('gte', 'lte', 'eq').index(k)) + string(a)


def definition(source):
    def initial(value):
        return sequence(sorted(value.items()), lambda v: string(v[0]) + sequence(v[1], string))
    def programs(value):
        return sequence(sorted(value.items()), lambda v: string(v[0]) + optional(v[1].get('root'), string) + sequence(v[1]['effects'], lambda e: ast(e, True)))
    return (optional(source.get('end_date'), string) + optional(source.get('end_conditions'), ast) +
            optional(source.get('end_root'), string) + optional(source.get('flag_keys'), lambda v: sequence(v, string)) +
            optional(source.get('initial_flags'), initial) + optional(source.get('effect_programs'), programs) +
            optional(source.get('score_weights'), lambda v: b''.join(string(v[k]) for k in ('victory_points', 'industrial_capacity', 'survival', 'faction_victory'))))


def q4(q):
    name, arg = next(iter(q['command'].items()))
    b = varint(q['tick']) + varint(q['nation']) + varint(q['sequence']) + varint(('Pause', 'SetSpeed', 'Move', 'Stop', 'Effects').index(name))
    if name in ('Pause', 'SetSpeed'):
        return b + bytes([int(arg)])
    if name == 'Move':
        return b + varint(arg['unit']) + varint(arg['destination'])
    if name == 'Stop':
        return b + varint(arg['unit'])
    return b + string(arg['program'])


def cause(v):
    if v in ('Date', 'Condition'):
        return varint(('Date', 'Condition').index(v))
    return b'\x02' + string(v['Explicit'])


def trigger(t):
    def end(v):
        return varint(v['tick']) + date(v['date']) + bytes([v['hour']]) + sequence(v['causes'], cause)
    return varint(t['definitions_hash']) + sequence(t['flags'], lambda f: varint(f[0]) + sequence(f[1], string)) + optional(t['ended'], end)


def movement(dto):
    def unit(u):
        return (varint(u['id']) + varint(u['nation']) + varint(u['province']) + signed(u['speed']) +
                sequence(u['allowed'], varint) + sequence(u['corrections'], lambda c: varint(c['from']) + varint(c['to']) + b''.join(signed(f) for f in c['factors'])) +
                sequence(u['route'], lambda l: varint(l['from']) + varint(l['to']) + signed(l['hours'])) + signed(u['elapsed']))
    b = b'\x01' + sequence(dto['base']['base']['units'], unit)
    if dto['strait_present']:
        b += b'\x01' + sequence(dto['base']['straits'], lambda u: varint(u['unit']) + sequence(u['corrections'], lambda c: varint(c['from']) + varint(c['to']) + varint(c['kind']) + b''.join(signed(f) for f in c['factors'])))
    return b


def canonical4(dto):
    base = dto['base']['base']['base']
    s, c = base['state'], base['config']
    prefix = string(s['scenario']) + varint(s['seed']) + varint(s['tick']) + date(s['date']) + bytes([s['hour'], int(s['paused']), s['speed']]) + b''.join(varint(v) for v in c['speed_ms_per_tick']) + bytes([c['initial_speed']])
    assert base['queue'] == dto['base']['base']['queue'] == []
    world = canonical(base)[len(prefix) + 1:]
    return prefix + sequence(dto['queue'], q4) + world + (movement(dto) if dto['movement_present'] else b'') + b'\x01' + trigger(dto['trigger'])


def check_state(report, pack):
    dto = report['dto']
    def numeric_types(value, key=''):
        if isinstance(value, dict):
            for k, v in value.items():
                numeric_types(v, k)
        elif isinstance(value, list):
            for v in value:
                numeric_types(v, key)
        elif type(value) is bool:
            assert key in ('paused', 'movement_present', 'strait_present', 'Pause'), 'boolean masquerades as integer'
        elif isinstance(value, (int, float)):
            assert type(value) is int, 'integer-only state required'
    numeric_types(dto)
    for nation, keys in dto['trigger']['flags']:
        assert type(nation) is int and 0 <= nation <= 65535 and all(type(k) is str for k in keys)
    assert report['format'] == 4
    assert type(report['ended']) is bool and report['ended'] == (dto['trigger']['ended'] is not None)
    encoded = canonical4(dto)
    assert encoded.hex() == report['canonical_hex'] and fnv(encoded) == report['hash']
    source = tomllib.loads((pack / 'scenarios/m1/scenario.toml').read_text(encoding='utf-8'))
    assert int(fnv(definition(source)), 16) == dto['trigger']['definitions_hash']
    assert pack_hash(pack) == report['pack']['content_hash']
    assert effective_defines(pack)[1] == report['effective_defines_hash']
    verify_ledger(dto['base']['base']['base'])
    return report


def native(command, folder):
    folder.mkdir(parents=True, exist_ok=False)
    with (folder / 'stdout').open('wb') as stdout, (folder / 'stderr').open('wb') as stderr:
        p = subprocess.Popen(command, cwd=ROOT, stdout=stdout, stderr=stderr)
        pid, code = p.pid, p.wait()
    receipt = {'command': command, 'cwd': str(ROOT), 'pid': pid, 'exit': code,
               'executable_sha256': sha(Path(command[0]))}
    (folder / 'receipt.json').write_text(json.dumps(receipt, indent=2), encoding='utf-8')
    assert code == 0, (folder / 'stderr').read_text(encoding='utf-8')
    value = strict_json(folder / 'stdout')
    assert value['pid'] == pid
    return value


def pack_for(run, name):
    return run / ('pack' if name in ('active', 'split', 'ended') else f'pack-{name}')


def capture(out):
    out = out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    before = identity()
    (out / 'source-before.json').write_text(json.dumps(before, indent=2), encoding='utf-8')
    build = subprocess.run(['cargo', 'build', '--locked', '-p', 'oh_save', '--example', 'trigger_fixture'], cwd=ROOT, capture_output=True)
    (out / 'build.stdout').write_bytes(build.stdout)
    (out / 'build.stderr').write_bytes(build.stderr)
    (out / 'build.receipt.json').write_text(json.dumps({'command': build.args, 'cwd': str(ROOT), 'exit': build.returncode}), encoding='utf-8')
    assert build.returncode == 0, build.stderr.decode(errors='replace')
    binary = ROOT / ('target/debug/examples/trigger_fixture.exe' if sys.platform == 'win32' else 'target/debug/examples/trigger_fixture')
    for i in (1, 2):
        run = out / f'run-{i}'
        run.mkdir()
        native([str(binary), 'capture', str(run)], out / f'capture-{i}')
        for name in CASES:
            destination = out / f'resume-{i}-{name}'
            destination.mkdir()
            native([str(binary), 'resume', str(pack_for(run, name)), str(run / f'{name}.ohsave'), '0', str(destination)], out / f'native-resume-{i}-{name}')
        destination = out / f'continued-{i}'
        destination.mkdir()
        native([str(binary), 'resume', str(run / 'pack'), str(run / 'split.ohsave'), '100', str(destination)], out / f'native-continued-{i}')
    after = identity()
    (out / 'source-after.json').write_text(json.dumps(after, indent=2), encoding='utf-8')
    assert before == after, 'capture changed source/index'
    result = {'mode': 'trigger-v4', 'platform': platform.system(), 'head': before['head'],
              'dirty': bool(before['status']), 'executable': str(binary), 'exe_sha256': sha(binary)}
    (out / 'result.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    result['files'] = files(out)
    (out / 'manifest.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
    verify_folder(out, require_clean=False)
    print(json.dumps({'platform': result['platform'], 'head': result['head'], 'dirty': result['dirty'], 'cases': CASES}))


def verify_native(folder, expected_exe, arguments, executable):
    r = strict_json(folder / 'receipt.json')
    assert r['exit'] == 0 and type(r['pid']) is int and r['pid'] > 0
    assert r['executable_sha256'] == expected_exe and re.fullmatch('[0-9a-f]{64}', expected_exe)
    assert r['cwd'] and r['command'][1] == arguments
    normalized=lambda p:p.replace('\\','/').rstrip('/')
    assert normalized(r['command'][0])==normalized(executable)
    assert normalized(r['cwd'])==normalized(executable).rsplit('/target/',1)[0]
    assert len(r['command'])==(3 if arguments=='capture' else 6)
    report = strict_json(folder / 'stdout')
    assert report['pid'] == r['pid']
    assert (folder / 'stderr').exists()
    return report, r


def verify_folder(folder, require_clean=True):
    manifest, result = strict_json(folder / 'manifest.json'), strict_json(folder / 'result.json')
    actual = files(folder)
    actual.pop('manifest.json')
    assert actual == manifest['files'], 'original file bytes changed'
    assert {k: v for k, v in manifest.items() if k != 'files'} == result
    assert result['mode'] == 'trigger-v4' and re.fullmatch('[0-9a-f]{40}', result['head'])
    before, after = strict_json(folder / 'source-before.json'), strict_json(folder / 'source-after.json')
    assert before == after and result['head'] == before['head'] and result['dirty'] == bool(before['status'])
    if require_clean:
        assert not result['dirty'] and before['status'] == before['diff'] == before['cached'] == ''
    captures = []
    for i in (1, 2):
        run = folder / f'run-{i}'
        capture, receipt = verify_native(folder / f'capture-{i}', result['exe_sha256'], 'capture', result['executable'])
        assert tuple(sorted(capture['cases'])) == tuple(sorted(CASES))
        for name in CASES:
            case = check_state(capture['cases'][name], pack_for(run, name))
            resume, r = verify_native(folder / f'native-resume-{i}-{name}', result['exe_sha256'], 'resume', result['executable'])
            assert r['pid'] != receipt['pid'], 'fresh process required'
            assert r['command'][-2] == '0'
            assert resume['split'] == resume['result'] == case
            assert (run / f'{name}.ohsave').read_bytes() == (folder / f'resume-{i}-{name}/reencoded.ohsave').read_bytes()
            h = header(run / f'{name}.ohsave')
            base = case['dto']['base']['base']['base']
            assert h['format_version'] == 4 and h['state_hash'] == case['hash']
            assert h['tick'] == base['state']['tick'] and h['game_date'] == base['state']['date']
            assert h['packs'] == [case['pack']] and h['effective_defines_hash'] == case['effective_defines_hash']
        continued, r = verify_native(folder / f'native-continued-{i}', result['exe_sha256'], 'resume', result['executable'])
        assert r['pid'] != receipt['pid'] and r['command'][-2] == '100'
        assert continued['split'] == capture['cases']['split'] and continued['result'] == capture['cases']['ended']
        ended = capture['cases']['ended']['dto']
        assert ended['trigger']['ended']['causes'] == ['Date', 'Condition', {'Explicit': 'a'}, {'Explicit': 'z'}]
        assert len(ended['queue']) == 5 and ended['trigger']['flags'] == [[1, ['x']], [2, ['x', 'y']]]
        assert ended['movement_present'] and ended['strait_present'] and ended['base']['base']['units'][0]['route'] == []
        assert capture['cases']['empty']['dto']['trigger']['flags'] == [[1, []], [2, []]]
        assert capture['cases']['initial']['ended'] and capture['cases']['initial']['dto']['base']['base']['base']['state']['tick'] == 0
        assert not capture['cases']['paused_condition']['ended'] and capture['cases']['paused_condition']['dto']['trigger']['flags'][0][1] == ['x']
        captures.append(capture['cases'])
    assert captures[0] == captures[1], 'repeat full DTO/canonical/definition mismatch'
    for name in CASES:
        assert (folder / f'run-1/{name}.ohsave').read_bytes() == (folder / f'run-2/{name}.ohsave').read_bytes(), 'repeat save bytes'
    return result, captures[0]


def _compare_for_head(root, expected_head, emit=True):
    expected = {f'trigger-{runner}' for runner in RUNNERS}
    assert {p.name for p in root.iterdir()} == expected, 'exact three OS artifacts required'
    heads, states = set(), []
    for runner, system in RUNNERS.items():
        result, cases = verify_folder(root / f'trigger-{runner}')
        assert result['head']==expected_head, 'artifact HEAD differs from current source HEAD'
        assert result['platform'] == system, 'actual platform mismatch'
        heads.add(result['head'])
        states.append(cases)
    assert len(heads) == 1, 'different source HEADs'
    assert states[0] == states[1] == states[2], 'cross-OS full state/identity/canonical/hash mismatch'
    for name in CASES:
        saves = [(root / f'trigger-{runner}/run-1/{name}.ohsave').read_bytes() for runner in RUNNERS]
        assert saves[0] == saves[1] == saves[2], 'cross-OS save bytes mismatch'
    if emit:
        print(f'actual 3OS trigger state/bytes/fresh native resume: {next(iter(heads))}')


def compare_artifacts(root, emit=True):
    # Public consumers cannot substitute an artifact's own HEAD as authority.
    # All Git calls are --no-optional-locks; original index bytes stay read-only.
    before=identity()
    if before['status'] or before['diff'] or before['cached']:
        raise ValueError('current source checkout is dirty; clean exact HEAD required')
    _compare_for_head(root, before['head'], emit=False)
    assert identity()==before, 'current source/index changed during comparison'
    if emit:
        print(f'actual 3OS trigger state/bytes/fresh native resume: {before["head"]}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('capture').add_argument('--out', type=Path, required=True)
    commands.add_parser('compare').add_argument('--root', type=Path, required=True)
    args = parser.parse_args()
    capture(args.out) if args.command == 'capture' else compare_artifacts(args.root)
