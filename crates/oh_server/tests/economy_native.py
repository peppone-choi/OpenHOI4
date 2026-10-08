"""Exclusive actual native v5 saved host and authoritative economic wire query."""
import hashlib,json,re,shutil,socket,subprocess,sys,time,tomllib,urllib.request
from pathlib import Path
from lifecycle import ctrl_c
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'crates/oh_save/tools'))
from check_save_current import files
from economy_current import header
from economy_wire_reference import verify_query

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def capture(source,out,force=False):
    source=Path(source).resolve();out=Path(out).resolve();out.mkdir(parents=True,exist_ok=False)
    evidence=json.loads((source/'result.json').read_text());record=evidence['runs'][0]
    packs=out/'packs';shutil.copytree(source/'run-1/pack',packs/'testland');save=source/'run-1/paused.ohsave';before=files(packs/'testland');saved_sha=sha(save)
    binary=ROOT/('target/debug/oh_server.exe' if sys.platform=='win32' else 'target/debug/oh_server')
    with socket.socket() as probe:probe.bind(('127.0.0.1',0));port=probe.getsockname()[1]
    argv=[str(binary),'--port',str(port),'--pack-root',str(packs),'--scenario','wp14_native','--load-save',str(save)]+(['--force'] if force else [])
    options={}
    if sys.platform=='win32':
        startup=subprocess.STARTUPINFO();startup.dwFlags=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0;options={'creationflags':subprocess.CREATE_NEW_CONSOLE,'startupinfo':startup}
    log=(out/'server.stdout').open('wb');errors=(out/'server.stderr').open('wb');process=subprocess.Popen(argv,cwd=ROOT,stdout=log,stderr=errors,**options);result=None
    try:
        url=f'http://127.0.0.1:{port}/';deadline=time.monotonic()+15
        while True:
            try:
                with urllib.request.urlopen(url,timeout=1) as response:assert response.status==200;html=response.read()
                break
            except OSError:
                assert process.poll() is None,'economic server exited during startup'
                if time.monotonic()>=deadline:raise
                time.sleep(.05)
        script=re.search(rb'<script[^>]*src="([^"]+)"',html).group(1).decode();js=urllib.request.urlopen(url.rstrip('/')+script,timeout=5).read();dist=ROOT/'client/dist'/script.lstrip('/');assert js==dist.read_bytes()
        command=['node','crates/oh_server/tests/economy_query.cjs',url];query=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=20)
        (out/'query.stdout').write_text(query.stdout,encoding='utf-8');(out/'query.stderr').write_text(query.stderr,encoding='utf-8');assert query.returncode==0,query.stderr
        decoded=json.loads(query.stdout);definitions=tomllib.loads((packs/'testland/common/economy/wp14_native.toml').read_text(encoding='utf-8'));verify_query(decoded,record,definitions)
        result=dict(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),dirty=bool(subprocess.check_output(['git','--no-optional-locks','status','--porcelain'],cwd=ROOT,text=True)),capture_mode=evidence['mode'],capture_head=evidence['head'],force=force,server_pid=process.pid,server_cwd=str(ROOT),server_command=argv,url=url,exe_sha256=sha(binary),js_path=script,served_js_sha256=hashlib.sha256(js).hexdigest(),built_js_sha256=sha(dist),http_status=200,query_exit=query.returncode,query_command=command,query=decoded,save_sha256=saved_sha,input_header=header(save),input_files=before,pack_hash=f"{record['pack']['content_hash']:016x}",save_preserved=sha(save)==saved_sha,source_preserved=files(packs/'testland')==before)
        assert result['head']==evidence['head'] and result['save_preserved'] and result['source_preserved']
    finally:
        if process.poll() is None:ctrl_c(process)
        code=process.wait(timeout=10);log.close();errors.close()
        if result is not None:
            result['server_exit']=code;assert code==0;assert sha(save)==saved_sha and files(packs/'testland')==before
            (out/'result.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
    return result
if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser();parser.add_argument('--capture',type=Path,required=True);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--force',action='store_true');args=parser.parse_args();capture(args.capture,args.out,args.force)
