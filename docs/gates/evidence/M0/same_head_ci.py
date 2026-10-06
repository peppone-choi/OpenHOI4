import pathlib, subprocess, json, hashlib, os
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify';dest=ev/'same-head';dest.mkdir(exist_ok=True);expected='d6ae2c8f176d6e8fdc0bc1a102ec2eb74ccac756';results=[]
os.environ['PATH']=r'C:\Users\user\.cargo\bin;'+os.environ['PATH']
for family,run,count in [('general','37448564387',7),('core','37448564385',4),('sim','37448564362',4)]:
 def query(name,endpoint):
  p=subprocess.run(['gh','api',endpoint],capture_output=True,check=True);(dest/f'{name}-{run}.json').write_bytes(p.stdout);return json.loads(p.stdout)
 meta=query('run',f'repos/peppone-choi/OpenHOI4/actions/runs/{run}');jobs=query('jobs',f'repos/peppone-choi/OpenHOI4/actions/runs/{run}/jobs?per_page=100')['jobs'];arts=query('artifacts',f'repos/peppone-choi/OpenHOI4/actions/runs/{run}/artifacts?per_page=100')['artifacts']
 assert meta['head_sha']==expected and meta['head_branch']=='main' and meta['status']=='completed' and meta['conclusion']=='success';assert len(jobs)==count and all(j['conclusion']=='success' for j in jobs)
 with (dest/f'run-{run}.log').open('wb') as f:p=subprocess.run(['gh','run','view',run,'--repo','peppone-choi/OpenHOI4','--log'],stdout=f,stderr=subprocess.PIPE,check=True)
 out=dest/family;p=subprocess.run(['gh','run','download',run,'--repo','peppone-choi/OpenHOI4','--dir',str(out)],capture_output=True,check=True)
 assert len(arts)==(1 if family=='general' else 3)
 for a in arts:
  assert a['workflow_run']['head_sha']==expected and not a['expired']
  zipfile=dest/(a['name']+'.zip')
  with zipfile.open('wb') as f:subprocess.run(['gh','api',f"repos/peppone-choi/OpenHOI4/actions/artifacts/{a['id']}/zip"],stdout=f,stderr=subprocess.PIPE,check=True)
  digest='sha256:'+hashlib.sha256(zipfile.read_bytes()).hexdigest();assert digest==a['digest']
 result={'family':family,'run':run,'head_sha':meta['head_sha'],'head_branch':meta['head_branch'],'status':meta['status'],'conclusion':meta['conclusion'],'jobs':[(j['id'],j['name'],j['conclusion']) for j in jobs],'artifacts':[{'id':a['id'],'name':a['name'],'digest':a['digest'],'digest_verified':True} for a in arts]}
 results.append(result);print(family,run,'same HEAD PASS',count,'jobs; artifacts',[(a['id'],a['name']) for a in arts],flush=True)
 if family in ['core','sim']:
  checker='crates/oh_core/tools/check_determinism.py' if family=='core' else 'crates/oh_sim/tools/check_sim_determinism.py'
  with (dest/f'{family}-compare.log').open('wb') as f:p=subprocess.run(['python',checker,'compare','--root',str(out)],stdout=f,stderr=subprocess.STDOUT)
  assert p.returncode==0;print((dest/f'{family}-compare.log').read_text(),flush=True)
  expected_hash='0dd81b8754bcc3f9' if family=='core' else 'ff921fd8148e699d'
  for text in out.rglob('*.txt'):assert text.read_text().strip()==expected_hash
(dest/'same-head-ci-summary.json').write_text(json.dumps(results,indent=2),encoding='utf8')
