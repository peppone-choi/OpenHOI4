from pathlib import Path
import os,subprocess,json
root=Path.cwd(); dest=root/'target/evidence/WP-05-verify'; sandbox=dest/'stale-workspace'
os.environ['PATH']=r'C:\Users\user\.cargo\bin;'+os.environ['PATH']; os.environ['CARGO_TARGET_DIR']=str(dest/'sandbox-build')
proto=sandbox/'client/src/proto/protocol.ts'; original=(root/'client/src/proto/protocol.ts').read_bytes()
proto.write_bytes(original+b'\n// independent stale injection\n')
cmd=['cargo','test','--manifest-path',str(sandbox/'Cargo.toml'),'-p','oh_proto','--test','contract','req_net_04_generated_types_are_current']
with (dest/'stale-negative-exact.log').open('w',encoding='utf8') as f: a=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
proto.write_bytes(original)
with (dest/'stale-restored-exact.log').open('w',encoding='utf8') as f: b=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
print('exact stale exit',a.returncode,'restored exit',b.returncode)
(dest/'stale-exact-results.json').write_text(json.dumps({'command':cmd,'stale':a.returncode,'restored':b.returncode},indent=2))
