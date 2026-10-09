"""Separate-process V7 host and actual authoritative military query/commands."""
import argparse, hashlib, json, re, socket, subprocess, time, urllib.request
from pathlib import Path
from lifecycle import ctrl_c
ROOT=Path(__file__).resolve().parents[3]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def capture(fixture,out,binary):
 fixture=Path(fixture).resolve();out=Path(out).resolve();out.mkdir(parents=True,exist_ok=False)
 expected=json.loads((fixture/'expected.json').read_text());pack=Path(expected['pack']);save=fixture/'ready.ohsave'
 inputs={str(p.relative_to(pack)):sha(p) for p in pack.rglob('*') if p.is_file()};saved_sha=sha(save)
 with socket.socket() as probe:probe.bind(('127.0.0.1',0));port=probe.getsockname()[1]
 argv=[str(Path(binary).resolve()),'--port',str(port),'--pack-root',str(pack.parent),'--scenario','m1','--load-save',str(save)]
 stdout=(out/'server.stdout').open('wb');stderr=(out/'server.stderr').open('wb');process=subprocess.Popen(argv,cwd=ROOT,stdout=stdout,stderr=stderr);result=None
 try:
  url=f'http://127.0.0.1:{port}/';deadline=time.monotonic()+15
  while True:
   try:
    with urllib.request.urlopen(url,timeout=1) as response:assert response.status==200;html=response.read()
    break
   except OSError:
    assert process.poll() is None,'military server exited during startup'
    if time.monotonic()>=deadline:raise
    time.sleep(.05)
  script=re.search(rb'<script[^>]*src="([^"]+)"',html).group(1).decode();served=urllib.request.urlopen(url.rstrip('/')+script,timeout=5).read();built=ROOT/'client/dist'/script.lstrip('/');assert served==built.read_bytes()
  command=['node','crates/oh_server/tests/military_query.cjs',url,str(fixture/'expected.json')]
  query=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
  (out/'query.stdout').write_text(query.stdout,encoding='utf8');(out/'query.stderr').write_text(query.stderr,encoding='utf8');assert query.returncode==0,query.stderr
  result=dict(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),server_command=argv,server_pid=process.pid,exe_sha256=sha(Path(binary)),fixture=expected,input_files=inputs,save_sha256=saved_sha,served_js_sha256=hashlib.sha256(served).hexdigest(),built_js_sha256=sha(built),query_command=command,query_exit=query.returncode,query=json.loads(query.stdout))
 finally:
  if process.poll() is None:ctrl_c(process)
  code=process.wait(timeout=10);stdout.close();stderr.close()
  if result is not None:
   result['server_exit']=code;assert code==0;assert sha(save)==saved_sha;assert {str(p.relative_to(pack)):sha(p) for p in pack.rglob('*') if p.is_file()}==inputs
   (out/'result.json').write_text(json.dumps(result,indent=2),encoding='utf8')
 return result
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--fixture',type=Path,required=True);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--binary',type=Path,required=True);args=parser.parse_args();capture(args.fixture,args.out,args.binary)
