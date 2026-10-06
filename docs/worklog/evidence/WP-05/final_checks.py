from pathlib import Path
import os, subprocess, json, shutil, hashlib
root=Path.cwd(); out=root/'target/wp05'; out.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env['PATH']='C:/Users/user/.cargo/bin;'+env['PATH'];env['PYTHONIOENCODING']='utf-8';env['OH_BROWSER_PRODUCTS']='1'
npm=shutil.which('npm.cmd') or shutil.which('npm')
checks=[
 ('npm-ci-final',[npm,'--prefix','client','ci']),
 ('client-build-final',[npm,'--prefix','client','run','build']),
 ('fmt-final',['cargo','fmt','--check']),
 ('clippy-final',['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings']),
 ('test-workspace-final',['cargo','test','--workspace','--locked']),
 ('typecheck-final',[npm,'--prefix','client','run','typecheck']),
 ('test-client-final',[npm,'--prefix','client','test']),
 ('server-build-final',['cargo','build','-p','oh_server','--locked']),
 ('lifecycle-final',['python','crates/oh_server/tests/lifecycle.py']),
 ('lifecycle-default-final',['python','crates/oh_server/tests/lifecycle.py','--default']),
 ('e2e-final',[npm,'--prefix','client','run','test:e2e']),
 ('hash-first',['cargo','run','-p','oh_cli','--','run','--scenario','testland','--ticks','1000','--seed','1','--hash-out']),
 ('hash-second',['cargo','run','-p','oh_cli','--','run','--scenario','testland','--ticks','1000','--seed','1','--hash-out']),
 ('cli-days',['cargo','run','-p','oh_cli','--','run','--scenario','testland','--days','365','--seed','1','--hash-out']),
 ('architecture-final',['python','tools/check_architecture.py']),
 ('docs-final',['python','tools/check_docs.py']),
 ('assets-final',['python','tools/check_assets.py','--release']),
 ('npm-licenses-final',[npm,'--prefix','client','run','licenses']),
 ('rust-deny-final',['cargo','deny','check','licenses','bans','advisories','sources']),
 ('wp01-fixtures-final',['python','-m','unittest','discover','-s','tools/tests','-p','test_wp01.py','-v']),
]
results=[]
for name, command in checks:
 result=subprocess.run(command,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 (out/(name+'.log')).write_bytes(result.stdout)
 results.append({'name':name,'command':command,'exit':result.returncode})
 (out/'final-results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
 print(name, 'exit='+str(result.returncode), flush=True)
 if result.returncode:print(result.stdout.decode('utf-8',errors='replace')[-2200:],flush=True)
before=hashlib.sha256((root/'client/src/proto/protocol.ts').read_bytes()).hexdigest()
result=subprocess.run(['cargo','run','-p','oh_proto','--locked','--example','generate'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
after=hashlib.sha256((root/'client/src/proto/protocol.ts').read_bytes()).hexdigest()
(out/'generate-final.log').write_bytes(result.stdout+f'\nsha256 before={before} after={after}\n'.encode())
assert result.returncode==0 and before==after
print('protocol regenerate unchanged',before,flush=True)
assert (out/'hash-first.log').read_text(encoding='utf-8').splitlines()[-1] == (out/'hash-second.log').read_text(encoding='utf-8').splitlines()[-1]
assert all(row['exit']==0 for row in results), 'Some final checks failed; inspect final-results.json'
