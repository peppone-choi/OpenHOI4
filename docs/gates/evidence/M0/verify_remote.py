import json, pathlib, hashlib, subprocess
root=pathlib.Path.cwd(); ev=root/'target/evidence/M0-gate-verify'; result={}
for run in ['37447307839','37447307865','37447307966']:
 meta=json.loads((ev/f'ci-{run}.json').read_text(encoding='utf-8-sig')); jobs=json.loads((ev/f'jobs-{run}.json').read_text(encoding='utf-8-sig'))['jobs']; artifacts=json.loads((ev/f'artifacts-{run}.json').read_text(encoding='utf-8-sig'))['artifacts']
 assert meta['head_sha']=='f12b30345c7255bdceb4d0e6b32405f5c48dd5b6'; assert meta['head_branch']=='main'; assert meta['conclusion']=='success'; assert all(j['conclusion']=='success' for j in jobs)
 assert all(a['workflow_run']['head_sha']==meta['head_sha'] and not a['expired'] for a in artifacts)
 result[run]={'head_sha':meta['head_sha'],'branch':meta['head_branch'],'jobs':[(j['id'],j['name'],j['conclusion']) for j in jobs],'artifacts':[(a['id'],a['name'],a['digest']) for a in artifacts]}
 print(run,result[run])
comparisons=[]
for remote,stored in [('remote-core','main-core'),('remote-sim','main-sim'),('remote-client/wp05-protocol-browser-evidence','main-client')]:
 rp=ev/remote; sp=root/'docs/verify/evidence/WP-05'/stored
 for p in sp.rglob('*'):
  if not p.is_file(): continue
  rel=p.relative_to(sp); actual=rp/rel
  assert actual.is_file(),str(actual)
  h=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
  assert h(p)==h(actual),str(rel)
  comparisons.append({'path':str(rel),'sha256':h(actual)})
result['byte_comparisons']=comparisons
for family in ['remote-core','remote-sim']:
 for p in (ev/family).rglob('*.txt'): print(p.relative_to(ev),p.read_text().strip())
(ev/'remote-verification.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
print('remote stored byte comparison',len(comparisons),'PASS')
