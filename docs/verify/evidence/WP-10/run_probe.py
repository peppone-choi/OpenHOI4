import pathlib,subprocess,os,json
root=pathlib.Path(r'E:/openhoi/.orchestrator/wt/WP-10-verify'); out=root/'target/verify/WP-10'
os.environ['PATH']=r'C:/Users/user/.cargo/bin;'+os.environ['PATH'];os.environ['PYTHONUTF8']='1';os.environ['CARGO_TARGET_DIR']=str(root/'target')
cmd=['cargo','run','--offline','--manifest-path','target/verify/WP-10/probe/Cargo.toml']
with (out/'probe.log').open('w',encoding='utf8') as f:
 f.write('command: '+subprocess.list2cmdline(cmd)+'\n');f.flush();p=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT);f.write('\nexit: '+str(p.returncode)+'\n')
print('probe exit',p.returncode)
