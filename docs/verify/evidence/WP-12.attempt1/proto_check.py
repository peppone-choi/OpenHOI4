import pathlib,subprocess,os,json
r=pathlib.Path('E:/openhoi/.orchestrator/wt/WP-12-verify');o=r/'target/wp12-verify';p=o/'proto-gen';(p/'src').mkdir(parents=True,exist_ok=True)
(p/'Cargo.toml').write_text('[package]\nname="wp12_verify_proto"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\noh_proto={path="'+(r/'crates/oh_proto').as_posix()+'"}\n')
(p/'src/main.rs').write_text('fn main(){print!("{}",oh_proto::typescript());}\n')
e=os.environ.copy();e['CARGO_TARGET_DIR']=str(r/'target')
a=subprocess.run(['C:/Users/user/.cargo/bin/cargo.exe','run','--offline','--manifest-path',str(p/'Cargo.toml')],cwd=r,env=e,capture_output=True)
(o/'generated-protocol.ts').write_bytes(a.stdout);print(a.stderr.decode());print('Generation exit',a.returncode)
idx=subprocess.check_output(['git','-C',str(r),'show',':client/src/proto/protocol.ts'])
print('Generated equals tracked Git LF blob',a.stdout==idx)
assert a.returncode==0 and a.stdout==idx
b=subprocess.run(['git','-C',str(r),'diff','--exit-code','--','client/src/proto/protocol.ts']);print('protocol diff exit',b.returncode);assert b.returncode==0
