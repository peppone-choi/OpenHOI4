from pathlib import Path
import json,re
r=Path(r'E:/openhoi/.orchestrator/wt/WP-09-verify/target/wp09-verify')
a=json.loads((r/'before.json').read_text(encoding='utf8'));b=json.loads((r/'interim.json').read_text(encoding='utf8'));print('same',a==b,'head',b['head'],'count',b['count'],'tracked_sha',b['tracked_list_sha256'],'index_sha',b['index_sha256']);print('changes',[k for k in a if a[k]!=b[k]])
print('Rust tests',sum(map(int,re.findall(r'test result: ok\. (\d+) passed', (r/'rustfinal-2.log').read_text(encoding='utf8')))))
p=r/'adversarial/results.json';d=json.loads(p.read_text(encoding='utf8'));print('Browser rows',len(d['rows']));print('last row',d['rows'][-1]['product'],d['rows'][-1]['name'])

