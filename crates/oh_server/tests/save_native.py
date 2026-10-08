"""Actual exclusive native --load-save host, served JS identity and wire query.
Run after npm ci/build, cargo build -p oh_server and save capture.
"""
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
import re
from lifecycle import ctrl_c
import economy_native

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT / 'crates/oh_save/tools'))
from save_reference import pack_hash
from check_save_current import files, header, verify_folder

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--force', action='store_true')
    args = parser.parse_args()
    out, source = args.out.resolve(), args.capture.resolve()
    capture = verify_folder(source, require_clean=False)
    out.mkdir(parents=True, exist_ok=True)
    packs = out / 'packs'
    shutil.copytree(source / 'run-1/pack', packs / 'testland', dirs_exist_ok=True)
    save = source / 'run-1/paused.ohsave'
    before = files(packs / 'testland')
    save_before = sha(save)
    binary = ROOT / ('target/debug/oh_server.exe' if sys.platform == 'win32' else 'target/debug/oh_server')
    with socket.socket() as probe:
        probe.bind(('127.0.0.1', 0))
        port = probe.getsockname()[1]
    command = [str(binary), '--port', str(port), '--pack-root', str(packs), '--load-save', str(save)]
    if args.force:
        command.append('--force')
    options = {}
    if sys.platform == 'win32':
        startup = subprocess.STARTUPINFO()
        startup.dwFlags = subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = 0
        options = {'creationflags':subprocess.CREATE_NEW_CONSOLE,'startupinfo':startup}
    log, errors = (out / 'server.stdout').open('wb'), (out / 'server.stderr').open('wb')
    process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=errors, **options)
    result = None
    try:
        url = f'http://127.0.0.1:{port}/'
        deadline = time.monotonic() + 15
        while True:
            try:
                with urllib.request.urlopen(url, timeout=1) as response:
                    assert response.status == 200
                    html = response.read()
                break
            except OSError:
                assert process.poll() is None, 'own server exited during startup'
                if time.monotonic() >= deadline:
                    raise
                time.sleep(.05)
        script = re.search(rb'<script[^>]*src="([^"]+)"',html).group(1).decode()
        js = urllib.request.urlopen(url.rstrip('/')+script,timeout=5).read()
        dist = ROOT / 'client/dist' / script.lstrip('/')
        assert js == dist.read_bytes(), 'served JS must match this checkout build'
        query_cmd = ['node','crates/oh_server/tests/save_query.cjs',url]
        query = subprocess.run(query_cmd,cwd=ROOT,text=True,capture_output=True,timeout=20)
        (out / 'query.stdout').write_text(query.stdout,encoding='utf-8')
        (out / 'query.stderr').write_text(query.stderr,encoding='utf-8')
        assert query.returncode == 0, query.stderr
        response = json.loads(query.stdout)
        assert response['welcome']['packs'][0]['hash'] == f'{pack_hash(packs / "testland"):016x}'
        startup_text = (out / 'server.stdout').read_text(encoding='utf-8')
        assert url in startup_text and response['welcome']['packs'][0]['hash'] in startup_text
        result = {'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                  'url':url,'server_pid':process.pid,'server_command':command,'exe_sha256':sha(binary),
                  'js_path':script,'served_js_sha256':hashlib.sha256(js).hexdigest(),
                  'save_sha256':sha(save),'pack_hash':response['welcome']['packs'][0]['hash'],
                  'query_command':query_cmd,'query_exit':query.returncode,'query':response}
        result.update(capture_mode=capture['evidence_mode'], capture_head=capture['head'],
                      server_cwd=str(ROOT),
                      dirty=bool(subprocess.check_output(['git','--no-optional-locks','status','--porcelain'],cwd=ROOT,text=True)),
                      force=args.force, http_status=200, input_header=header(save), input_files=before,
                      built_js_sha256=sha(dist), save_preserved=sha(save)==save_before,
                      source_preserved=files(packs / 'testland')==before)
        assert result['head'] == capture['head']
        assert result['save_preserved'] and result['source_preserved']
    finally:
        if process.poll() is None:
            ctrl_c(process)
        code = process.wait(timeout=10)
        log.close()
        errors.close()
        if result is not None:
            result['server_exit'] = code
            assert code == 0
            assert files(packs / 'testland') == before and sha(save) == save_before
            (out / 'result.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
            print(json.dumps({key:result[key] for key in ('head','url','server_pid','server_exit','exe_sha256','served_js_sha256','pack_hash')}))

    economy_native.capture(source/'economy-v5',out/'economy-v5',args.force)

if __name__ == '__main__':
    main()
