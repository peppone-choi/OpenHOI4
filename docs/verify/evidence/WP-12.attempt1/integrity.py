import json,pathlib,hashlib
r=pathlib.Path('E:/openhoi/.orchestrator/wt/WP-12-verify/target/wp12-verify');b=json.loads((r/'before.json').read_text(encoding='utf-8'));a=json.loads((r/'after.json').read_text(encoding='utf-8'))
changed=[p for p in b['files'] if b['files'][p]!=a['files'].get(p)]
d={k:a[k]==b[k] for k in ['head','status','index','diff','cached','files']};d.update({'tracked_count_before':len(b['files']),'tracked_count_after':len(a['files']),'changed_paths':changed,'beforeSHA':hashlib.sha256((r/'before.json').read_bytes()).hexdigest(),'afterSHA':hashlib.sha256((r/'after.json').read_bytes()).hexdigest()});(r/'integrity.json').write_text(json.dumps(d,indent=2));print(json.dumps(d,indent=2));assert all(d[k] for k in ['head','status','index','diff','cached','files'])
