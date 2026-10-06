import subprocess,shutil,json,os,pathlib
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify';sandbox=ev/'negative-workspace';paths=subprocess.check_output(['git','ls-files'],text=True).splitlines()
for p in paths:
 if p.startswith('crates/') or p in ['Cargo.toml','Cargo.lock','rust-toolchain.toml','client/src/proto/protocol.ts']:
  to=sandbox/p;to.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(root/p,to)
os.environ['PATH']=r'C:\Users\user\.cargo\bin;'+os.environ['PATH'];os.environ['CARGO_TARGET_DIR']=str(ev/'negative-build')
proto=sandbox/'client/src/proto/protocol.ts';original=proto.read_bytes();results=[]
def run(name,cmd,expect):
 with (ev/(name+'.log')).open('w',encoding='utf8') as f:p=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,timeout=400)
 results.append({'name':name,'command':cmd,'exit':p.returncode,'expected':expect});print(name,p.returncode,flush=True);assert p.returncode==expect
base=['cargo','test','--locked','--manifest-path',str(sandbox/'Cargo.toml'),'-p','oh_proto','--test','contract','req_net_04_generated_types_are_current']
proto.write_bytes(original+b'\n// Independent M0 stale negative injection\n');run('stale-negative',base,101)
assert 'assertion `left == right` failed' in (ev/'stale-negative.log').read_text()
proto.write_bytes(original);run('stale-restored',base,0)
run('generate-copy',['cargo','run','--locked','--manifest-path',str(sandbox/'Cargo.toml'),'-p','oh_proto','--example','generate'],0)
assert proto.read_bytes()==original
run('missing-client-negative',['cargo','check','--locked','--manifest-path',str(sandbox/'Cargo.toml'),'-p','oh_server'],101)
text=(ev/'missing-client-negative.log').read_text();assert 'npm' in text and 'build' in text
(ev/'negative-results.json').write_text(json.dumps(results,indent=2))
