import subprocess, os, json, pathlib, time
root=pathlib.Path.cwd(); ev=root/'target/evidence/M0-gate-verify'
os.environ['PATH']=r'C:\Users\user\.cargo\bin;C:\Program Files\nodejs;'+os.environ['PATH']
commands=[('fmt',['cargo','fmt','--check']),('clippy',['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings']),('workspace-tests',['cargo','test','--workspace','--locked']),('server-build',['cargo','build','-p','oh_server','--locked']),('client-tests',['npm.cmd','--prefix','client','test']),('hash-1000-1',['cargo','run','-p','oh_cli','--locked','--','run','--scenario','testland','--ticks','1000','--seed','1','--hash-out']),('hash-1000-2',['cargo','run','-p','oh_cli','--locked','--','run','--scenario','testland','--ticks','1000','--seed','1','--hash-out']),('hash-365',['cargo','run','-p','oh_cli','--locked','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']),('architecture',['python','tools/check_architecture.py']),('docs',['python','tools/check_docs.py']),('assets',['python','tools/check_assets.py','--release']),('violation-fixtures',['python','-m','unittest','discover','-s','tools/tests','-p','test_wp01.py','-v']),('npm-licenses',['npm.cmd','--prefix','client','run','licenses']),('rust-deny',['cargo','deny','check','licenses','bans','advisories','sources']),('core-workflow-tests',['python','-m','unittest','discover','-s','crates/oh_core/tests','-p','test_determinism_workflow.py','-v']),('sim-workflow-tests',['python','-m','unittest','discover','-s','crates/oh_sim/tests','-p','test_sim_determinism.py','-v']),('lifecycle',['python','crates/oh_server/tests/lifecycle.py']),('lifecycle-default',['python','crates/oh_server/tests/lifecycle.py','--default']),('lifecycle-open',['python','crates/oh_server/tests/lifecycle.py','--open'])]
results=[]
for name,cmd in commands:
 t=time.time()
 with (ev/(name+'.log')).open('w',encoding='utf-8') as f:
  try: p=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,timeout=500); code=p.returncode
  except Exception as e: f.write(str(e));code=-1
 results.append(dict(name=name,command=cmd,exit=code,seconds=round(time.time()-t,2)))
 (ev/'local-results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
 print(name,code,flush=True)
 if code: print((ev/(name+'.log')).read_text(encoding='utf-8')[-2500:],flush=True)
