import os,sys,json,hashlib,subprocess,pathlib,time
sys.stdout.reconfigure(errors='backslashreplace')
ROOT=pathlib.Path(__file__).resolve().parents[2]
OUT=ROOT/'target/wp11-independent'
os.environ['GIT_OPTIONAL_LOCKS']='0'
os.environ['PATH']='C:/Users/user/.cargo/bin;'+os.environ['PATH']
os.environ['TEMP']=os.environ['TMP']=str(OUT/'tmp')
(OUT/'tmp').mkdir(exist_ok=True)
BASE=json.load(open('E:/openhoi/.orchestrator/evidence/WP11.verify3.before.json',encoding='utf-8'))
def git(*args):
 return subprocess.check_output(['git','--no-optional-locks','-C',str(ROOT),*args],env=os.environ).decode('utf-8')
def snapshot(label):
 idx=pathlib.Path(git('rev-parse','--git-path','index').strip());idx=idx if idx.is_absolute() else ROOT/idx
 raw=idx.read_bytes();(OUT/(label+'.index')).write_bytes(raw)
 paths=git('ls-files','-z').split('\0')
 files=[[p,hashlib.sha256((ROOT/p).read_bytes()).hexdigest()] for p in paths if p]
 d=dict(worktree=str(ROOT),head=git('rev-parse','HEAD').strip(),raw_index_sha256=hashlib.sha256(raw).hexdigest(),semantic_index=git('ls-files','--stage','-z'),tracked_list=paths,status=git('status','--porcelain=v1','--untracked-files=all'),diff=git('diff','--binary'),cached_diff=git('diff','--cached','--binary'),files=files)
 (OUT/(label+'.json')).write_text(json.dumps(d),encoding='utf-8')
 changes={k: [BASE[k],d[k]] for k in ['head','raw_index_sha256','status','diff','cached_diff'] if BASE[k]!=d[k]}
 changedfiles=[p for p,h in files if dict(BASE['files']).get(p)!=h]
 result=dict(label=label,raw=d['raw_index_sha256'],files=len(files),changes=changes,changedfiles=changedfiles,semantic_equal=d['semantic_index']==BASE['semantic_index'],list_equal=d['tracked_list']==BASE['tracked_list'],raw_bytes_equal=raw==pathlib.Path('E:/openhoi/.orchestrator/evidence/WP11.verify3.before.index').read_bytes())
 (OUT/(label+'.summary.json')).write_text(json.dumps(result),encoding='utf-8');print(json.dumps(result),flush=True)
 return result
if __name__=='__main__':
 label=sys.argv[1]
 snapshot(label+'-before')
 if len(sys.argv)>2:
  args=sys.argv[2:];cwd=ROOT
  if args[0]=='CLIENT':cwd=ROOT/'client';args=args[1:]
  if args[0] in ['E2EM0','E2EM1']:
   suite='m0' if args[0]=='E2EM0' else 'm1';cwd=ROOT/'client';args=args[1:]
   os.environ.update(json.load(open(OUT/'Effective-output-paths.json',encoding='utf-8'))['envs'][suite])
   subprocess.run(['node','-e','console.log(JSON.stringify({cwd:process.cwd(),env:Object.fromEntries(Object.entries(process.env).filter(([k])=>k.startsWith("OH_"))),cli:require("node:path").resolve("node_modules/@playwright/test/cli.js")}))'],cwd=cwd,env=os.environ,stdout=open(OUT/(suite+'-actual-cli-env.json'),'wb'),check=True)
  args=['npx.cmd' if a=='npx' else a for a in args]
  if args[0]=='npm':args[0]='npm.cmd'
  t=time.time()
  with open(OUT/(label+'.log'),'wb') as f:r=subprocess.run(args,cwd=cwd,env=os.environ,stdout=f,stderr=subprocess.STDOUT)
  rec=dict(label=label,args=args,cwd=str(cwd),exit=r.returncode,seconds=time.time()-t)
  with open(OUT/'commands.jsonl','a',encoding='utf-8') as f:f.write(json.dumps(rec)+'\n')
  print(json.dumps(rec),flush=True)
  print((OUT/(label+'.log')).read_text(encoding='utf-8',errors='replace')[-5000:],flush=True)
 snapshot(label+'-after')
