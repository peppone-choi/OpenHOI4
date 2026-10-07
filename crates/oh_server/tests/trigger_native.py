"""Exclusive native end/tick0/paused-active query and normal shutdown evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import time
import urllib.request
from lifecycle import ctrl_c

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'crates/oh_sim/tools'))
from check_trigger_determinism import files, sha, pack_for, strict_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    source, out = args.capture.resolve(), args.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    capture = strict_json(source / 'capture-1/stdout')
    # JavaScript cannot preserve large u64 from JSON.parse; stringify the hash explicitly.
    expected = json.loads(json.dumps(capture))
    for case in expected['cases'].values():
        case['dto']['trigger']['definitions_hash'] = str(case['dto']['trigger']['definitions_hash'])
    (out / 'expected.json').write_text(json.dumps(expected), encoding='utf-8')
    binary = ROOT / ('target/debug/oh_server.exe' if sys.platform == 'win32' else 'target/debug/oh_server')
    for name, startup in (('ended', False), ('initial', False), ('paused_condition', False), ('initial', True)):
        folder = out / (name + ('-startup' if startup else '-restore'))
        folder.mkdir()
        packs = folder / 'packs'
        pack = packs / 'testland'
        shutil.copytree(pack_for(source / 'run-1', name), pack)
        save = source / f'run-1/{name}.ohsave'
        before, saved = files(pack), sha(save)
        with socket.socket() as probe:
            probe.bind(('127.0.0.1', 0))
            port = probe.getsockname()[1]
        url = f'http://127.0.0.1:{port}/'
        command = [str(binary), '--port', str(port), '--pack-root', str(packs)]
        if not startup:
            command += ['--load-save', str(save)]
        options = {}
        if sys.platform == 'win32':
            info = subprocess.STARTUPINFO()
            info.dwFlags = subprocess.STARTF_USESHOWWINDOW
            info.wShowWindow = 0
            options = {'creationflags': subprocess.CREATE_NEW_CONSOLE, 'startupinfo': info}
        with (folder / 'server.stdout').open('wb') as stdout, (folder / 'server.stderr').open('wb') as stderr:
            p = subprocess.Popen(command, cwd=ROOT, stdout=stdout, stderr=stderr, **options)
            result = {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'cmd': command, 'cwd': str(ROOT), 'pid': p.pid, 'exe_sha256': sha(binary)}
            try:
                deadline = time.monotonic() + 15
                while True:
                    try:
                        with urllib.request.urlopen(url, timeout=1) as response:
                            assert response.status == 200
                            break
                    except OSError:
                        assert p.poll() is None
                        assert time.monotonic() < deadline
                        time.sleep(.05)
                node = ['node', 'crates/oh_server/tests/trigger_query.cjs', url, str(out / 'expected.json'), name]
                query = subprocess.run(node, cwd=ROOT, capture_output=True, timeout=20)
                (folder / 'query.stdout').write_bytes(query.stdout)
                (folder / 'query.stderr').write_bytes(query.stderr)
                result.update(query_cmd=node, query_cwd=str(ROOT), query_exit=query.returncode, http_status=200)
                assert query.returncode == 0, query.stderr.decode(errors='replace')
                assert p.poll() is None, 'query must remain available after ending'
            finally:
                if p.poll() is None:
                    ctrl_c(p)
                code = p.wait(timeout=10)
                result.update(exit=code, save_preserved=sha(save) == saved, pack_preserved=files(pack) == before)
                (folder / 'result.json').write_text(json.dumps(result, indent=2), encoding='utf-8')
                assert code == 0 and result['save_preserved'] and result['pack_preserved']
    print('actual ended/tick0/paused-condition restore + tick0 startup: HTTP200, authoritative query, normal native0')


if __name__ == '__main__':
    main()
