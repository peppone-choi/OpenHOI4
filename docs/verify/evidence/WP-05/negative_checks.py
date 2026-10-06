import subprocess,shutil,json,os
from pathlib import Path
root=Path.cwd(); dest=root/'target/evidence/WP-05-verify'; sandbox=dest/'stale-workspace'
paths=subprocess.check_output(['git','ls-files'],text=True).splitlines()
for p in paths:
 if p.startswith('crates/') or p in ['Cargo.toml','Cargo.lock','rust-toolchain.toml'] or p=='client/src/proto/protocol.ts':
  to=sandbox/p; to.parent.mkdir(parents=True,exist_ok=True); shutil.copy2(root/p,to)
proto=sandbox/'client/src/proto/protocol.ts'
proto.write_text(proto.read_text(encoding='utf8')+'\n// independent stale injection\n',encoding='utf8')
os.environ['PATH']=r'C:\Users\user\.cargo\bin;'+os.environ['PATH']
os.environ['CARGO_TARGET_DIR']=str(dest/'sandbox-build')
cmd=['cargo','test','--manifest-path',str(sandbox/'Cargo.toml'),'-p','oh_proto','--test','contract','req_net_04_generated_types_are_current']
with (dest/'stale-negative.log').open('w',encoding='utf8') as f: p=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
print('stale-negative exit='+str(p.returncode),flush=True)
proto.write_bytes((root/'client/src/proto/protocol.ts').read_bytes())
with (dest/'stale-restored.log').open('w',encoding='utf8') as f: q=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
print('stale-restored exit='+str(q.returncode),flush=True)
cmd2=['cargo','check','--manifest-path',str(sandbox/'Cargo.toml'),'-p','oh_server']
with (dest/'missing-client-negative.log').open('w',encoding='utf8') as f: r=subprocess.run(cmd2,stdout=f,stderr=subprocess.STDOUT)
print('missing-client-negative exit='+str(r.returncode),flush=True)
(dest/'negative-results.json').write_text(json.dumps({'stale':p.returncode,'restored':q.returncode,'missing-client':r.returncode},indent=2))
