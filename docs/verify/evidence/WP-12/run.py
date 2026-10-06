import subprocess,pathlib,hashlib,json,sys,os,time
ROOT=pathlib.Path('E:/openhoi/.orchestrator/wt/WP-12-verify2')
OUT=ROOT/'target/wp12-verify2'
os.environ['PATH']='C:/Users/user/.cargo/bin;'+os.environ['PATH']
os.environ.update(OH_BROWSER_PRODUCTS='1',OH_TEST_PORT='19412',OH_E2E_EVIDENCE='../target/wp12-verify2/browser',OH_MALFORMED_EVIDENCE='../target/wp12-verify2/malformed')
def snap(name):
 def git(*a):return subprocess.check_output(['git','-C',str(ROOT),*a]).decode()
 paths=git('ls-files').splitlines()
 data={'head':git('rev-parse','HEAD'),'status':git('status','--porcelain=v1'),'index':git('ls-files','--stage'),'diff':git('diff'),'cached':git('diff','--cached'),'files':{p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in paths}}
 (OUT/(name+'.json')).write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
 print(name,data['head'].strip(),len(paths),repr(data['status']),flush=True)
 return data
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
elif sys.argv[1]=='suite':
 npm=['node','C:/Program Files/nodejs/node_modules/npm/bin/npm-cli.js','--prefix','client']
 cargo=['C:/Users/user/.cargo/bin/cargo.exe']
 tasks=[('npm-ci',npm+['ci']),('client-tests',npm+['test']),('client-build',npm+['run','build']),('server-build',cargo+['build','-p','oh_server']),('fmt',cargo+['fmt','--check']),('clippy',cargo+['clippy','--workspace','--','-D','warnings']),('rust-tests',cargo+['test','--workspace']),('hash-1',cargo+['run','-p','oh_cli','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']),('hash-2',cargo+['run','-p','oh_cli','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']),('typecheck',npm+['run','typecheck']),('e2e',npm+['run','test:e2e']),('npm-licenses',npm+['run','licenses']),('cargo-deny',cargo+['deny','check','licenses','bans','advisories','sources']),('assets',['python','tools/check_assets.py']),('assets-release',['python','tools/check_assets.py','--release']),('docs',['python','tools/check_docs.py']),('architecture',['python','tools/check_architecture.py']),('policy-fixtures',['python','-m','unittest','discover','-s','tools/tests','-p','test_wp01.py','-v']),('lifecycle',['python','crates/oh_server/tests/lifecycle.py'])]
 for label,cmd in tasks:run(label,cmd)
else:sys.exit(run(sys.argv[1],sys.argv[2:]))
