import subprocess,os,pathlib,json,shutil
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify';os.environ['PATH']=r'C:\Users\user\.cargo\bin;C:\Program Files\nodejs;'+os.environ['PATH'];os.environ['OH_BROWSER_PRODUCTS']='1'
cmd=['npm.cmd','run','test:e2e','--','--output',str(ev/'playwright-results')]
with (ev/'e2e.log').open('w',encoding='utf8') as f:p=subprocess.run(cmd,cwd=root/'client',stdout=f,stderr=subprocess.STDOUT,timeout=240)
print('5 browser e2e exit',p.returncode,flush=True);print((ev/'e2e.log').read_text()[-2800:],flush=True)
shutil.copytree(root/'target/wp05',ev/'browser',dirs_exist_ok=True)
source=(root/'docs/verify/evidence/WP-05/exact_display.cjs').read_text().replace("target/evidence/WP-05-verify","target/evidence/M0-gate-verify")
(ev/'exact_display.cjs').write_text(source,encoding='utf8')
with (ev/'exact-display.log').open('w',encoding='utf8') as f:q=subprocess.run(['node',str(ev/'exact_display.cjs')],stdout=f,stderr=subprocess.STDOUT,timeout=150)
print('exact server display exit',q.returncode,flush=True);print((ev/'exact-display.log').read_text(),flush=True)
(ev/'browser-results.json').write_text(json.dumps({'e2e_command':cmd,'e2e_exit':p.returncode,'exact_exit':q.returncode},indent=2))
