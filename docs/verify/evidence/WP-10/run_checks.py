import subprocess, pathlib, os, json, hashlib, datetime
root=pathlib.Path(r'E:/openhoi/.orchestrator/wt/WP-10-verify')
out=root/'target/verify/WP-10'
os.environ['PATH']=r'C:/Users/user/.cargo/bin;'+os.environ['PATH']
os.environ['PYTHONUTF8']='1'
def git(*args): return subprocess.check_output(['git',*args],cwd=root)
files=git('ls-files','-z').decode().split('\0')[:-1]
state={'head':git('rev-parse','HEAD').decode().strip(),'status':git('status','--porcelain=v1').decode(),'diff':git('diff').decode(),'index_diff':git('diff','--cached').decode(),'files':{p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in files},'index':git('ls-files','--stage').decode()}
(out/'before.json').write_text(json.dumps(state,ensure_ascii=False,indent=2),encoding='utf8')
commands=[('npm-ci',['npm.cmd','--prefix','client','ci']),('client-build',['npm.cmd','--prefix','client','run','build']),('fmt',['cargo','fmt','--check']),('clippy',['cargo','clippy','--workspace','--locked','--','-D','warnings']),('workspace-tests',['cargo','test','--workspace','--locked']),('sim-all-targets',['cargo','test','-p','oh_sim','--all-targets','--locked']),('hash1',['cargo','run','-p','oh_cli','--locked','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']),('hash2',['cargo','run','-p','oh_cli','--locked','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']),('architecture',['python','tools/check_architecture.py']),('docs',['python','tools/check_docs.py']),('assets',['python','tools/check_assets.py','--release']),('licenses',['cargo','deny','check','licenses','bans','advisories','sources']),('npm-licenses',['npm.cmd','--prefix','client','run','licenses']),('diff-check',['git','diff','--check']),('policy-tests',['python','-m','unittest','discover','-s','tools/tests','-p','test_wp01.py','-v'])]
results=[]
for name,cmd in commands:
 print('START '+name,flush=True)
 with (out/(name+'.log')).open('w',encoding='utf8') as f:
  f.write('command: '+subprocess.list2cmdline(cmd)+'\n'); f.flush()
  p=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT)
  f.write('\nexit: '+str(p.returncode)+'\n')
 results.append({'name':name,'cmd':cmd,'exit':p.returncode})
 (out/'results.json').write_text(json.dumps(results,indent=2),encoding='utf8')
 print('END '+name+' exit='+str(p.returncode),flush=True)
