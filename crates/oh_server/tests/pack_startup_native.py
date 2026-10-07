"""WP-24 P06: actual native startup/restore, HTTP/WS and child termination receipts."""
import argparse
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('lifecycle', Path(__file__).with_name('lifecycle.py'))
lifecycle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lifecycle)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def invoke(args, out, name):
    r = subprocess.run(args, cwd=ROOT, capture_output=True)
    (out/(name+'.stdout')).write_bytes(r.stdout)
    (out/(name+'.stderr')).write_bytes(r.stderr)
    (out/(name+'.command.json')).write_text(json.dumps(dict(command=args,cwd=str(ROOT),exit=r.returncode),indent=2)+'\n')
    return r


def serve(pack_root, out, name, save=None, force=False, expect_online=False, cause=None):
    with socket.socket() as s:
        s.bind(('127.0.0.1',0));port=s.getsockname()[1]
    exe=ROOT/'target/debug'/('oh_server.exe' if sys.platform=='win32' else 'oh_server')
    args=[str(exe),'--port',str(port),'--pack-root',str(pack_root)]
    if save: args+=['--load-save',str(save)]
    if force: args+=['--force']
    kwargs={}
    if sys.platform=='win32':
        info=subprocess.STARTUPINFO();info.dwFlags=subprocess.STARTF_USESHOWWINDOW;info.wShowWindow=0
        kwargs=dict(creationflags=subprocess.CREATE_NEW_CONSOLE,startupinfo=info)
    process=subprocess.Popen(args,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,**kwargs)
    http=None;ws_status=None;closed=False;ctrl_c=False;error=None
    try:
        deadline=time.monotonic()+15
        while process.poll() is None and time.monotonic()<deadline:
            try:
                with urllib.request.urlopen(f'http://127.0.0.1:{port}/',timeout=.3) as response:http=response.status
                break
            except OSError:time.sleep(.03)
        if http:
            with socket.create_connection(('127.0.0.1',port),timeout=3) as sock:
                key=base64.b64encode(os.urandom(16)).decode()
                sock.sendall(f'GET /ws HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n'.encode())
                headers=b''
                while not headers.endswith(b'\r\n\r\n'):headers+=lifecycle.recv_exact(sock,1)
                ws_status=headers.split(b'\r\n')[0].decode()
                lifecycle.ctrl_c(process);ctrl_c=True
                while lifecycle.frame(sock)[0]!=8:pass
                closed=True
        output,errors=process.communicate(timeout=10)
        # A rejected input must exit itself. A served control is stopped normally.
        ok=(http==200 and ws_status=='HTTP/1.1 101 Switching Protocols' and closed and process.returncode==0) if expect_online else (http is None and not ctrl_c and process.returncode==1 and b'startup error:' in errors and (cause is None or cause.encode() in errors))
    except Exception as exc:
        error=repr(exc);ok=False
        if process.poll() is None:
            lifecycle.ctrl_c(process);ctrl_c=True
        output,errors=process.communicate(timeout=10)
    receipt=dict(name=name,exe=str(exe),exe_sha256=sha(exe),cwd=str(ROOT),args=args,pid=process.pid,port=port,http_status=http,ws_status=ws_status,ws_close=closed,ctrl_c_sent=ctrl_c,native_exit=process.returncode,expect_online=expect_online,expected_cause=cause,passed=ok,error=error)
    (out/(name+'.stdout')).write_bytes(output);(out/(name+'.stderr')).write_bytes(errors)
    (out/(name+'.receipt.json')).write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out',required=True,type=Path)
    parser.add_argument('--red',action='store_true')
    args=parser.parse_args();out=args.out.resolve();out.mkdir(parents=True,exist_ok=True)
    binary=ROOT/'target/debug'/('oh_cli.exe' if sys.platform=='win32' else 'oh_cli')
    cases=[('normal',None,None,None),
           ('dependency','manifest.toml','depends = []',"depends = [{ id = 'missing', version = '*' }]"),
           ('ftl','localisation/en/pack.ftl',None,'broken = {\n')]
    if not args.red:
        cases += [('engine','manifest.toml','engine = ">=0.1, <0.2"','engine = ">=0.2"'),
                  ('version','manifest.toml','version = "0.1.0"','version = "invalid"'),
                  ('conflict','manifest.toml','conflicts = []',"conflicts = ['testland']"),
                  ('schema','scenarios/m1/scenario.toml','map = "testland"','unknown = "testland"'),
                  ('reference','scenarios/m1/nations/NTH.toml','capital = 10','capital = 65535'),
                  ('defines','scenarios/m1/defines.toml',None,'invalid [ syntax\n'),
                  ('missing-network','scenarios/m1/defines.toml','max_message_bytes = 65536','')]
    receipts=[]
    causes=dict(dependency='missing dependency',ftl='invalid Fluent',engine='incompatible',version='invalid SemVer',conflict='self conflicts',schema='unknown field',reference='capital',defines='defines.toml',**{'missing-network':'max_message_bytes'})
    for name,file,old,new in cases:
        root=out/name;shutil.copytree(ROOT/'data/packs/testland',root/'testland')
        save=root/'start.ohsave'
        r=invoke([str(binary),'run','--pack',str(root/'testland'),'--scenario','m1','--ticks','0','--seed','1','--save-out',str(save)],out,name+'-make-save')
        assert r.returncode==0,r.stderr
        before=sha(save)
        if file:
            path=root/'testland'/file
            if old is None:path.write_text(new,encoding='utf-8',newline='\n')
            else:
                text=path.read_text(encoding='utf-8');assert old in text;path.write_text(text.replace(old,new),encoding='utf-8',newline='\n')
        modes=[('startup',None,False)] if args.red else [('startup',None,False),('restore',save,False),('restore-force',save,True)]
        for mode,load,force in modes:receipts.append(serve(root,out,name+'-'+mode,load,force,name=='normal',causes.get(name)))
        assert sha(save)==before
    if not args.red:
        legacy=out/'m0';shutil.copytree(ROOT/'data/packs/examples/m0/testland',legacy/'testland')
        save=out/'m0.ohsave'
        r=invoke([str(binary),'run','--scenario','testland','--ticks','1','--seed','7','--save-out',str(save)],out,'m0-make-save');assert r.returncode==0,r.stderr
        before=sha(save)
        for mode,load,force in [('startup',None,False),('restore',save,False),('restore-force',save,True)]:
            receipt=serve(legacy,out,'m0-'+mode,load,force,True);receipts.append(receipt)
            assert (out/('m0-'+mode+'.stderr')).read_bytes().count(b'metadata unavailable')==2
        assert sha(save)==before
        for mutation in ['one-byte','extra','missing','bad-ftl','plain-frozen']:
            parent=out/('m0-invalid-'+mutation)
            source=ROOT/'crates/oh_save/tests/fixtures/m1-pack-v1' if mutation=='plain-frozen' else ROOT/'data/packs/examples/m0/testland'
            shutil.copytree(source,parent/'testland')
            pack=parent/'testland'
            if mutation=='one-byte':
                p=pack/'defines.toml';raw=p.read_bytes();assert b'Original' in raw;p.write_bytes(raw.replace(b'Original',b'original',1))
            elif mutation=='extra':(pack/'extra.txt').write_text('extra')
            elif mutation=='missing':(pack/'defines.toml').unlink()
            elif mutation=='bad-ftl':
                (pack/'localisation/en').mkdir(parents=True);(pack/'localisation/en/broken.ftl').write_text('broken = {\n')
            for mode,load,force in [('startup',None,False),('restore',save,False),('restore-force',save,True)]:
                receipts.append(serve(parent,out,mutation+'-'+mode,load,force,False))
    (out/'results.json').write_text(json.dumps(receipts,indent=2)+'\n')
    print(json.dumps(dict(total=len(receipts),passed=sum(r['passed'] for r in receipts))))
    return int(any(not r['passed'] for r in receipts))


if __name__=='__main__':raise SystemExit(main())
