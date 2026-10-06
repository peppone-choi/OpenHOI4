import os, sys, subprocess, pathlib, hashlib, json, datetime, socket, shutil
ROOT=pathlib.Path(__file__).resolve().parents[2]
OUT=ROOT/'target/wp08-verify5'
env=os.environ.copy(); env['PATH']=r'C:\Users\user\.cargo\bin;'+env['PATH'];env['PYTHONUTF8']='1'
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT).decode('utf-8').strip()
def snap(name):
 files=git('ls-files','-z').split('\0'); files=[f for f in files if f]
 s={'time':now(),'root':str(ROOT),'head':git('rev-parse','HEAD'),'indexSHA':hashlib.sha256(pathlib.Path(git('rev-parse','--git-path','index')).read_bytes()).hexdigest(),'tracked':{f:hashlib.sha256((ROOT/f).read_bytes()).hexdigest() for f in files},'status':git('status','--porcelain=v1'),'staged':git('diff','--cached','--binary'),'unstaged':git('diff','--binary')}
 (OUT/(name+'.json')).write_text(json.dumps(s,ensure_ascii=False,indent=2),encoding='utf-8'); print(name,len(files),s['head'],repr(s['status']),flush=True)
def run(name,args,cwd=ROOT,extra=None):
 e=env.copy();e.update(extra or {});start=now()
 with (OUT/(name+'.log')).open('w',encoding='utf-8') as f:
  f.write(json.dumps({'args':args,'cwd':str(cwd),'start':start},ensure_ascii=False)+'\n');f.flush()
  actual=[shutil.which(args[0],path=e['PATH']) or args[0],*args[1:]]
  p=subprocess.run(actual,cwd=cwd,env=e,stdout=f,stderr=subprocess.STDOUT)
 r={'name':name,'args':args,'cwd':str(cwd),'start':start,'end':now(),'exit':p.returncode}
 with (OUT/'commands.jsonl').open('a',encoding='utf-8') as f:f.write(json.dumps(r)+'\n')
 print(json.dumps(r),flush=True);return p.returncode
if __name__=='__main__':
 phase=sys.argv[1]
 if phase=='bootstrap':
  snap('before');run('environment',['powershell','-NoProfile','-Command','node --version; npm --version; cargo --version; rustc --version; python --version; netstat -ano -p tcp | Select-String LISTENING; Get-Process | Where-Object {$_.ProcessName -match "oh_server"} | Select-Object Id,Path'])
  run('npm-ci',['npm.cmd','--prefix','client','ci'])
  run('npm-test',['npm.cmd','--prefix','client','test'])
  run('client-build',['npm.cmd','--prefix','client','run','build'])
  run('server-build',['cargo','build','-p','oh_server','--locked'])
 elif phase=='checks':
  for name,args in [('fmt',['cargo','fmt','--check']),('clippy',['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings']),('workspace-test',['cargo','test','--workspace','--locked']),('proto',['cargo','run','-p','oh_proto','--locked','--example','generate']),('proto-diff',['git','diff','--exit-code','--','client/src/proto']),('typecheck',['npm.cmd','--prefix','client','run','typecheck']),('npm-licenses',['npm.cmd','--prefix','client','run','licenses']),('cargo-deny',['cargo','deny','check','licenses','bans','advisories','sources']),('assets',['python','tools/check_assets.py','--release']),('docs',['python','tools/check_docs.py']),('architecture',['python','tools/check_architecture.py']),('policy-fixtures',['python','-m','unittest','discover','-s','tools/tests','-p','test_wp01.py','-v']),('lifecycle',['python','crates/oh_server/tests/lifecycle.py'])]:run(name,args)
 elif phase=='e2e':
  e={'OH_TEST_PORT':'19443','OH_BROWSER_PRODUCTS':'1'}
  for key,label in [('OH_MAP_EVIDENCE','map'),('OH_MAP_REDIRECT_EVIDENCE','redirect'),('OH_MAP_CAPABILITY_EVIDENCE','capability'),('OH_MAP_RESIZE_EVIDENCE','resize'),('OH_MAP_PIPELINE_EVIDENCE','pipeline'),('OH_E2E_EVIDENCE','panels'),('OH_MALFORMED_EVIDENCE','malformed')]:e[key]=str(OUT/label)
  run('m1-e2e',['node',str(ROOT/'client/node_modules/@playwright/test/cli.js'),'test','--config','playwright.m1.config.ts','--workers','1','--reporter','line','--output',str(OUT/'m1-results')],ROOT/'client',e)
  e['OH_TEST_PORT']='19444';e['OH_E2E_EVIDENCE']=str(OUT/'m0');e['OH_MALFORMED_EVIDENCE']=str(OUT/'m0-malformed')
  run('m0-e2e',['node',str(ROOT/'client/node_modules/@playwright/test/cli.js'),'test','--config','playwright.config.ts','--workers','1','--reporter','line','--output',str(OUT/'m0-results')],ROOT/'client',e)
 elif phase=='after':snap('after')
