from pathlib import Path
import json,hashlib
r=Path(r'E:/openhoi/.orchestrator/wt/WP-09-verify/target/wp09-verify')
a=json.loads((r/'before.json').read_text(encoding='utf8'));b=json.loads((r/'after.json').read_text(encoding='utf8'));assert a==b
rows=json.loads((r/'adversarial/results.json').read_text(encoding='utf8'))['rows'];selection=json.loads((r/'selection/results.json').read_text(encoding='utf8'))['rows'];assert len(rows)==105 and len(selection)==5
summary={'unchanged':True,'head':b['head'],'tracked_count':b['count'],'tracked_list_sha256':b['tracked_list_sha256'],'index_sha256':b['index_sha256'],'before_snapshot_sha256':hashlib.sha256((r/'before.json').read_bytes()).hexdigest(),'after_snapshot_sha256':hashlib.sha256((r/'after.json').read_bytes()).hexdigest(),'pageerrors':sum(len(x['errors']) for x in rows+selection),'external_requests':sum(len(x['externalRequests']) for x in rows+selection),'browser_cases':len(rows)+len(selection),'shape_disconnect_cases':sum(x['kind']=='reject' for x in rows),'safe_display_cases':sum(x['kind']=='safe' for x in rows)}
(r/'summary.json').write_text(json.dumps(summary,indent=2),encoding='utf8');print(json.dumps(summary,indent=2))
