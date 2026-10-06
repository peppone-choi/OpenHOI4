import json
from pathlib import Path
base=Path('target/evidence/WP-04-verify')
a=json.loads((base/'before.json').read_text(encoding='utf-8-sig'));b=json.loads((base/'after.json').read_text(encoding='utf-8-sig'))
assert a==b,(a,b)
assert b['head']=='88d730a386c6af4913705b209cc5ce6f2dfd050c'
assert not b['status'] and not b['diff'] and not b['cached']
ci=json.loads((base/'branch-ci.json').read_text(encoding='utf-8-sig'))
assert ci['headSha']==b['head'] and ci['status']=='completed' and ci['conclusion']=='success'
assert all(j['status']=='completed' and j['conclusion']=='success' for j in ci['jobs'])
print('HEAD/tracked list/status/diff/cached diff unchanged; clean detached worktree: PASS')
print('General branch CI exact SHA, all jobs success: PASS')
for j in ci['jobs']:print(j['name'],j['conclusion'])
