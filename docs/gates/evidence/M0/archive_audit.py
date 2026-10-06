import pathlib,json,subprocess,hashlib
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify';dest=ev/'remote-zips';dest.mkdir(exist_ok=True);results=[]
for run in ['37447307839','37447307865','37447307966']:
 meta=json.loads((ev/f'ci-{run}.json').read_text(encoding='utf-8-sig'));jobs=json.loads((ev/f'jobs-{run}.json').read_text(encoding='utf-8-sig'))['jobs'];assert meta['head_branch']=='main' and meta['head_sha']=='f12b30345c7255bdceb4d0e6b32405f5c48dd5b6' and meta['conclusion']=='success';assert all(j['conclusion']=='success' for j in jobs)
 artifacts=json.loads((ev/f'artifacts-{run}.json').read_text(encoding='utf-8-sig'))['artifacts']
 for a in artifacts:
  assert a['workflow_run']['head_sha']==meta['head_sha'] and not a['expired']
  out=dest/(a['name']+'.zip')
  with out.open('wb') as f:p=subprocess.run(['gh','api',f"repos/peppone-choi/OpenHOI4/actions/artifacts/{a['id']}/zip"],stdout=f,stderr=subprocess.PIPE,timeout=60)
  assert p.returncode==0,p.stderr.decode()
  digest='sha256:'+hashlib.sha256(out.read_bytes()).hexdigest();assert digest==a['digest'],(a['name'],digest,a['digest'])
  results.append({'run':run,'head_sha':meta['head_sha'],'id':a['id'],'name':a['name'],'digest':digest,'digest_matches':True});print(a['name'],'API archive SHA256 PASS',flush=True)
(ev/'remote-archive-verification.json').write_text(json.dumps(results,indent=2),encoding='utf8')
