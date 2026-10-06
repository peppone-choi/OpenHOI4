import pathlib,json,hashlib,datetime
r=pathlib.Path('E:/openhoi/.orchestrator/wt/WP-12-verify2');o=r/'target/wp12-verify2'
ns={};exec((o/'run.py').read_text(encoding='utf-8-sig').split("if sys.argv[1]")[0],ns)
before=json.loads((o/'before.json').read_text(encoding='utf-8'));after=ns['snap']('after')
changed=[k for k in before if before[k]!=after[k]]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
result={'unchanged':before==after,'changed_sections':changed,'tracked_count':len(after['files']),'snapshot_before_sha256':sha(o/'before.json'),'snapshot_after_sha256':sha(o/'after.json'),'head':after['head'].strip(),'status':after['status'],'checked_at':datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).isoformat()}
(o/'integrity.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
print(json.dumps(result,indent=2));assert not changed
