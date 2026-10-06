import subprocess,pathlib,hashlib,json,os,sys,time
sys.stdout.reconfigure(encoding='utf8'); os.environ['PYTHONIOENCODING']='utf8'
ROOT=pathlib.Path(r'E:/openhoi/.orchestrator/wt/WP-09-verify'); os.chdir(ROOT)
OUT=ROOT/'target/wp09-verify'; OUT.mkdir(exist_ok=True)
os.environ['PATH']=r'C:/Users/user/.cargo/bin;'+os.environ['PATH']
def git(*args): return subprocess.check_output(['git','-C',str(ROOT),*args])
def snapshot(name):
 paths=git('ls-files','-z').decode().split('\0')[:-1]
 data={'head':git('rev-parse','HEAD').decode().strip(),'count':len(paths),'tracked_list_sha256':hashlib.sha256(git('ls-files','-z')).hexdigest(),'files':{p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in paths},'status':git('status','--porcelain=v1').decode(),'diff':git('diff','--binary').decode(),'staged':git('diff','--cached','--binary').decode(),'index_sha256':hashlib.sha256(pathlib.Path(git('rev-parse','--git-path','index').decode().strip()).read_bytes()).hexdigest()}
 (OUT/(name+'.json')).write_text(json.dumps(data,indent=2),encoding='utf8'); return data
if sys.argv[1]=='snapshot': snapshot(sys.argv[2]);sys.exit()
groups={
'client':[['npm.cmd','--prefix','client','ci'],['npm.cmd','--prefix','client','test'],['npm.cmd','--prefix','client','run','typecheck'],['npm.cmd','--prefix','client','run','build'],['cargo','build','-p','oh_server']],
'rust':[['cargo','fmt','--check'],['cargo','clippy','--workspace','--all-targets','--','-D','warnings'],['cargo','test','--workspace']],
'checks':[['python','tools/check_docs.py'],['python','tools/check_assets.py'],['python','tools/check_architecture.py'],['npm.cmd','--prefix','client','run','licenses'],['python','-m','unittest','discover','-s','crates/oh_core/tests','-p','test_determinism_workflow.py','-v'],['python','-m','unittest','discover','-s','crates/oh_sim/tests','-p','test_sim_determinism.py','-v']],
'hashes':[['cargo','run','-p','oh_cli','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']]*2+[['cargo','run','-p','oh_cli','--','run','--pack','data/packs/testland','--scenario','m1','--days','365','--seed','1','--hash-out']]*2+[['python','crates/oh_sim/tools/check_sim_determinism.py','capture','--out','target/wp09-verify/capture-m0'],['python','crates/oh_sim/tools/check_sim_determinism.py','capture','--m1','--out','target/wp09-verify/capture-m1']],
'e2e':[['npm.cmd','--prefix','client','run','test:e2e','--','--config','playwright.m1.config.ts'],['npm.cmd','--prefix','client','run','test:e2e']],
}
groups['rustfinal']=groups['rust']; groups['clientfinish']=[groups['client'][-1]]
group=sys.argv[1]
if group=='e2e': os.environ.update(OH_BROWSER_PRODUCTS='1',OH_TEST_PORT='19412',OH_E2E_EVIDENCE='../target/wp09-verify/e2e',OH_MALFORMED_EVIDENCE='../target/wp09-verify/malformed')
results=[]
for i,command in enumerate(groups[group]):
 print('START',group,i,subprocess.list2cmdline(command),flush=True); started=time.time()
 logfile=OUT/f'{group}-{i}.log'
 with logfile.open('wb') as log:
  p=subprocess.run(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
 row={'command':command,'exit':p.returncode,'seconds':round(time.time()-started,2),'log':str(logfile)};results.append(row)
 (OUT/(group+'.json')).write_text(json.dumps(results,indent=2),encoding='utf8')
 print('END',json.dumps(row),flush=True);print(logfile.read_text(encoding='utf8',errors='replace')[-1800:],flush=True)

