from pathlib import Path
import subprocess,sys,json,importlib.util
root=Path('target/evidence/WP-04-verify/gate-independent');root.mkdir(parents=True,exist_ok=True)
tool=Path('crates/oh_sim/tools/check_sim_determinism.py')
records=[]
for case in ['equal','missing_os','missing_file','extra_os','invalid','repeat_mismatch','os_mismatch']:
 tree=root/case;tree.mkdir(exist_ok=True)
 for os in ['ubuntu-latest','windows-latest','macos-latest']:
  if case=='missing_os' and os=='macos-latest':continue
  folder=tree/f'sim-hash-{os}';folder.mkdir(exist_ok=True)
  for file in ['sim-hash.txt','run-1.txt','run-2.txt']:
   if case=='missing_file' and os=='macos-latest' and file=='run-2.txt':continue
   v='ff921fd8148e699d\n'
   if os=='macos-latest':
    if case=='invalid' and file=='sim-hash.txt':v='FF921FD8148E699D\n'
    if case=='repeat_mismatch' and file=='run-2.txt':v='0000000000000000\n'
    if case=='os_mismatch':v='0000000000000000\n'
   (folder/file).write_text(v,encoding='utf8')
 if case=='extra_os':(tree/'sim-hash-extra').mkdir(exist_ok=True)
 command=[sys.executable,str(tool),'compare','--root',str(tree)]
 p=subprocess.run(command,capture_output=True,text=True)
 expected=0 if case=='equal' else 1
 assert p.returncode==expected,(case,p.returncode,p.stdout,p.stderr)
 records.append(dict(case=case,command=command,exit=p.returncode,stdout=p.stdout,stderr=p.stderr,expected=expected))
 print(case,': exit=',p.returncode,'expected=',expected)
# Execute an actual failing child process through capture, rather than mocking run.
spec=importlib.util.spec_from_file_location('gate',tool);g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
out=root/'capture_failure';out.mkdir(exist_ok=True);marker=out/'sim-hash.txt';marker.write_text('ff921fd8148e699d\n')
g.COMMAND=(sys.executable,'-c','import sys;sys.exit(73)')
try:g.capture(out)
except subprocess.CalledProcessError as e:
 assert e.returncode==73
 assert not marker.exists()
 records.append(dict(case='actual_child_failure',command=list(g.COMMAND),exit=e.returncode,marker_exists=marker.exists()))
else:raise AssertionError('capture hid process failure')
print('actual failing child exit=73 and stale success marker removed: PASS')
(root/'results.json').write_text(json.dumps(records,indent=2),encoding='utf8')
