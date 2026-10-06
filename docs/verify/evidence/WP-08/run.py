import os, sys, json, subprocess, hashlib, pathlib, time
ROOT=pathlib.Path(__file__).resolve().parents[2]
OUT=ROOT/'target/wp08-verify2'
ENV=os.environ.copy()
ENV['PATH']=str(pathlib.Path.home()/'.cargo/bin')+os.pathsep+ENV['PATH']
ENV.update(OH_BROWSER_PRODUCTS='1',OH_TEST_PORT='19414',OH_E2E_EVIDENCE='../target/wp08-verify2/e2e',OH_MALFORMED_EVIDENCE='../target/wp08-verify2/malformed',OH_MAP_EVIDENCE='../target/wp08-verify2/map',OH_MAP_REDIRECT_EVIDENCE='../target/wp08-verify2/redirect',PYTHONUTF8='1')
CARGO=str(pathlib.Path.home()/'.cargo/bin/cargo.exe')
NPM=['C:/Program Files/nodejs/node.exe','C:/Program Files/nodejs/node_modules/npm/bin/npm-cli.js']
def git(*args):
 return subprocess.check_output(['git',*args],cwd=ROOT).decode('utf-8')
def snapshot(name):
 paths=git('ls-files','-z').split('\0')[:-1]
 data={'head':git('rev-parse','HEAD').strip(),'index':git('ls-files','--stage','-z'),'status':git('status','--porcelain=v1'),'diff':git('diff','--binary'),'cached_diff':git('diff','--cached','--binary'),'files':[[p,hashlib.sha256((ROOT/p).read_bytes()).hexdigest()] for p in paths]}
 (OUT/(name+'.json')).write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
 print(json.dumps({'snapshot':name,'head':data['head'],'tracked':len(paths),'status':data['status']}),flush=True)
 return data
def run(name,args,cwd=ROOT):
 start=time.monotonic()
 with (OUT/(name+'.log')).open('w',encoding='utf-8') as f:
  f.write(json.dumps({'argv':args,'cwd':str(cwd),'environment':{k:v for k,v in ENV.items() if k.startswith('OH_')}})+'\n');f.flush()
  p=subprocess.run(args,cwd=cwd,env=ENV,stdout=f,stderr=subprocess.STDOUT)
  f.write('\nexit='+str(p.returncode)+'\n')
 result={'name':name,'argv':args,'exit':p.returncode,'seconds':round(time.monotonic()-start,2)}
 with (OUT/'commands.jsonl').open('a',encoding='utf-8') as f:f.write(json.dumps(result)+'\n')
 print(json.dumps(result),flush=True)
 if p.returncode:print((OUT/(name+'.log')).read_text(encoding='utf-8')[-6000:],flush=True)
 return p.returncode
if __name__=='__main__':
 mode=sys.argv[1]
 if mode=='before':snapshot('before')
 elif mode=='client':
  for name,args in [('npm-ci',NPM+['--prefix','client','ci']),('client-unit',NPM+['--prefix','client','test']),('client-build',NPM+['--prefix','client','run','build']),('server-build',[CARGO,'build','-p','oh_server'])]:
   if run(name,args):sys.exit(1)
 elif mode=='rust':
  for name,args in [('fmt',[CARGO,'fmt','--check']),('clippy',[CARGO,'clippy','--workspace','--','-D','warnings']),('rust-tests',[CARGO,'test','--workspace']),('proto-generate',[CARGO,'run','-p','oh_proto','--example','generate']),('proto-diff',['git','diff','--exit-code','--','client/src/proto'])]:run(name,args)
 elif mode=='e2e':
  run('m1-e2e',NPM+['--prefix','client','run','test:e2e','--','--config','playwright.m1.config.ts','--workers','3','--output','../target/wp08-verify2/m1-results'])
  run('m0-e2e',NPM+['--prefix','client','run','test:e2e','--','--workers','3','--output','../target/wp08-verify2/m0-results'])
 elif mode=='checks':
  for name,args in [('typecheck',NPM+['--prefix','client','run','typecheck']),('npm-licenses',NPM+['--prefix','client','run','licenses']),('assets',[sys.executable,'tools/check_assets.py','--release']),('docs',[sys.executable,'tools/check_docs.py']),('architecture',[sys.executable,'tools/check_architecture.py']),('diff-check',['git','diff','--check']),('deny',[CARGO,'deny','check','licenses'])]:run(name,args)
 elif mode=='hashes':
  for scenario,extra in [('m1',['--pack','data/packs/testland']),('testland',[])]:
   for i in (1,2):run(('m1' if scenario=='m1' else 'm0')+'-hash-'+str(i),[CARGO,'run','-p','oh_cli','--','run','--scenario',scenario,*extra,'--days','365','--seed','1','--hash-out'])
  run('capture-m1',[sys.executable,'crates/oh_sim/tools/check_sim_determinism.py','capture','--m1','--out','target/wp08-verify2/m1-capture'])
 elif mode=='after':
  after=snapshot('after');before=json.loads((OUT/'before.json').read_text(encoding='utf-8'));same=after==before
  (OUT/'integrity.json').write_text(json.dumps({'same':same,'fields':{k:before[k]==after[k] for k in before},'tracked':len(after['files'])},indent=2),encoding='utf-8');print('integrity='+str(same));sys.exit(0 if same else 4)
