import subprocess,json,time,os
from pathlib import Path
root=Path.cwd(); dest=root/'target/evidence/WP-05-verify'
os.environ['PATH']=r'C:\Users\user\.cargo\bin;'+os.environ['PATH']
commands=[('client-build',['npm.cmd','--prefix','client','run','build']),('fmt',['cargo','fmt','--check']),('clippy',['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings']),('test-workspace',['cargo','test','--workspace','--locked']),('client-test',['npm.cmd','--prefix','client','test']),('generate',['cargo','run','-p','oh_proto','--locked','--example','generate']),('generate-diff',['git','diff','--exit-code','--','client/src/proto']),('server-build',['cargo','build','-p','oh_server','--locked']),('hash-365-1',['cargo','run','-p','oh_cli','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']),('hash-365-2',['cargo','run','-p','oh_cli','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']),('hash-1000',['cargo','run','-p','oh_cli','--','run','--scenario','testland','--ticks','1000','--seed','1','--hash-out']),('architecture',['python','tools/check_architecture.py']),('docs',['python','tools/check_docs.py']),('assets',['python','tools/check_assets.py','--release']),('lifecycle',['python','crates/oh_server/tests/lifecycle.py']),('lifecycle-default',['python','crates/oh_server/tests/lifecycle.py','--default']),('lifecycle-open',['python','crates/oh_server/tests/lifecycle.py','--open'])]
results=[]
for name,command in commands:
 print('START '+name,flush=True); t=time.time()
 with (dest/(name+'.log')).open('w',encoding='utf-8') as f:
  p=subprocess.run(command,stdout=f,stderr=subprocess.STDOUT)
 results.append(dict(name=name,command=command,exit=p.returncode,seconds=round(time.time()-t,2)))
 (dest/'results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
 print(f'END {name} exit={p.returncode}',flush=True)
