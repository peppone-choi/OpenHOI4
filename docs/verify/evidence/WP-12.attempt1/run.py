import subprocess, pathlib, hashlib, json, sys, os, time
ROOT=pathlib.Path('E:/openhoi/.orchestrator/wt/WP-12-verify')
OUT=ROOT/'target/wp12-verify'
os.environ['PATH']='C:/Users/user/.cargo/bin;'+os.environ['PATH']
def snap(name):
 def git(*a): return subprocess.check_output(['git', '-C',str(ROOT),*a]).decode()
 paths=git('ls-files').splitlines()
 data={'head':git('rev-parse','HEAD'),'status':git('status','--porcelain=v1'),'index':git('ls-files','--stage'),'diff':git('diff'),'cached':git('diff','--cached'),'files':{p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in paths}}
 (OUT/(name+'.json')).write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
 print(name, data['head'].strip(),len(paths),'status='+repr(data['status']))
def run(label,cmd):
 start=time.time()
 with (OUT/(label+'.log')).open('w',encoding='utf-8') as f:
  f.write(json.dumps(cmd)+'\n');f.flush()
  p=subprocess.run(cmd,cwd=ROOT,env=os.environ,stdout=f,stderr=subprocess.STDOUT)
  f.write('\nEXIT='+str(p.returncode)+'\n')
 print(label,'EXIT',p.returncode,'seconds',round(time.time()-start,1),flush=True)
 with (OUT/'results.jsonl').open('a',encoding='utf-8') as f:f.write(json.dumps({'label':label,'command':cmd,'exit':p.returncode})+'\n')
 return p.returncode
if sys.argv[1]=='snap':snap(sys.argv[2])
else:run(sys.argv[1],sys.argv[2:])
