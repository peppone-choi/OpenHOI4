from pathlib import Path
import json
base=Path('target/evidence/WP-04-verify')
head='88d730a386c6af4913705b209cc5ce6f2dfd050c'
run=json.loads((base/'remote-run.json').read_text(encoding='utf-8-sig'))
art=json.loads((base/'remote-artifacts.json').read_text(encoding='utf-8-sig'))
assert run['headSha']==head and run['conclusion']=='success' and run['status']=='completed'
assert len(run['jobs'])==4 and all(j['conclusion']=='success' for j in run['jobs'])
assert art['total_count']==3
rows=[]
for a in art['artifacts']:
 assert a['workflow_run']['head_sha']==head and a['workflow_run']['id']==37441857380 and not a['expired']
 for name in ['run-1.txt','run-2.txt','sim-hash.txt']:
  p=base/'remote-sim'/a['name']/name
  raw=p.read_bytes()
  assert raw.decode().strip()=='ff921fd8148e699d'
  rows.append(dict(artifact=a['name'],artifact_id=a['id'],file=name,raw=repr(raw),head=head))
print(json.dumps(rows,indent=2))
