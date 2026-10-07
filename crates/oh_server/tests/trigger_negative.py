"""Real capability and old-None/local-Some/force rejection without source loss."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
from pack_startup_native import serve

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'crates/oh_sim/tools'))
from check_trigger_determinism import files, sha


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    out = parser.parse_args().out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    cli = ROOT / ('target/debug/oh_cli.exe' if sys.platform == 'win32' else 'target/debug/oh_cli')
    def run(command, name, expected, cause=None):
        with (out / f'{name}.stdout').open('wb') as stdout, (out / f'{name}.stderr').open('wb') as stderr:
            p = subprocess.Popen(command, cwd=ROOT, stdout=stdout, stderr=stderr)
            pid, code = p.pid, p.wait()
        (out / f'{name}.receipt.json').write_text(json.dumps({'command': command, 'cwd': str(ROOT), 'pid': pid, 'exit': code, 'exe_sha256': sha(cli)}, indent=2), encoding='utf-8')
        assert code == expected
        if cause:
            assert cause in (out / f'{name}.stderr').read_text(encoding='utf-8')
    packs = out / 'packs'
    pack = packs / 'testland'
    shutil.copytree(ROOT / 'data/packs/testland', pack)
    save = out / 'original-v1.ohsave'
    run([str(cli), 'run', '--pack', str(pack), '--scenario', 'm1', '--ticks', '0', '--seed', '7', '--save-out', str(save)], 'make-v1', 0)
    saved = sha(save)
    scenario = pack / 'scenarios/m1/scenario.toml'
    scenario.write_text((ROOT / 'crates/oh_save/tests/fixtures/trigger/scenario.toml').read_text(encoding='utf-8').replace('2000-02-28', '2000-01-01'), encoding='utf-8')
    before = files(pack)
    for force in (False, True):
        name = 'force' if force else 'normal'
        command = [str(cli), 'resume', '--load', str(save), '--pack', str(pack), '--ticks', '0', '--save-out', str(out / f'{name}.ohsave')]
        if force:
            command += ['--force']
        run(command, f'cli-mode-{name}', 1, 'TriggerModeMismatch')
        assert not (out / f'{name}.ohsave').exists()
        receipt = serve(packs, out, f'server-mode-{name}', save, force, False, 'TriggerModeMismatch')
        assert receipt['passed'] and receipt['native_exit'] == 1 and receipt['http_status'] is None and receipt['ws_status'] is None
        assert sha(save) == saved and files(pack) == before
    source = scenario.read_text(encoding='utf-8')
    scenario.write_text(source.replace('end_conditions = { date_gte = "2000-03-01" }', 'end_root = "NTH"\nend_conditions = { at_war = false }'), encoding='utf-8')
    before = files(pack)
    run([str(cli), 'validate', '--deny-warnings', str(pack)], 'cli-capability', 1, 'host capability')
    run([str(cli), 'run', '--pack', str(pack), '--scenario', 'm1', '--ticks', '1', '--seed', '7'], 'cli-runtime-capability', 1, 'host capability')
    receipt = serve(packs, out, 'server-capability', None, False, False, 'host capability')
    assert receipt['passed'] and receipt['native_exit'] == 1 and receipt['http_status'] is None and receipt['ws_status'] is None
    assert sha(save) == saved and files(pack) == before
    print('actual CLI/server normal+force mode mismatch and unavailable at_war rejected; original save/pack preserved')


if __name__ == '__main__':
    main()
