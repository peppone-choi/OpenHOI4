from pathlib import Path
import hashlib,json,subprocess,datetime
root=Path.cwd(); dest=root/'target/evidence/WP-05-verify'; source=Path(r'E:/openhoi/.orchestrator/wt/WP-05/target/wp05')
files={p.name:{'size':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in source.iterdir() if p.is_file()}
(dest/'implementation-evidence-index.json').write_text(json.dumps({'source':str(source),'files':files},indent=2),encoding='utf8')
checks={}
for name,args in [('head',['git','rev-parse','HEAD']),('status',['git','status','--porcelain=v1']),('diff',['git','diff','--exit-code']),('staged-diff',['git','diff','--cached','--exit-code'])]:
 p=subprocess.run(args,capture_output=True,text=True); checks[name]={'exit':p.returncode,'output':p.stdout,'stderr':p.stderr}
before=json.loads((dest/'tracked-before.json').read_text(encoding='utf-8-sig')); after=json.loads((dest/'tracked-after.json').read_text(encoding='utf-8-sig'))
checks.update({'trackedCount':len(before),'trackedListAndHashesEqual':before==after,'cwd':str(root),'checkedAtKST':datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).isoformat()})
(dest/'integrity-final.json').write_text(json.dumps(checks,indent=2),encoding='utf8');print(json.dumps(checks,indent=2))
original=json.loads((source/'final-results.json').read_text(encoding='utf-8-sig')); print('Original final records',len(original),'all exits zero',all(x['exit']==0 for x in original))
