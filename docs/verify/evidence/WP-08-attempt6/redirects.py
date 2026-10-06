import pathlib,json,hashlib,socket
import run as r
from runtime import Runtime,ROOT,OUT
src=OUT/'prior-attempt/verify5-extra/redirect-matrix';dst=OUT/'redirect-matrix';dst.mkdir(exist_ok=True);changes=[]
for p in src.glob('*.ts'):
 before=p.read_text(encoding='utf8');after=before.replace('E:/openhoi/.orchestrator/wt/WP-08-verify5',ROOT.as_posix()).replace('127.0.0.1:19444','127.0.0.1:19454').replace("foreign.listen(0, '127.0.0.1')","foreign.listen(19458, '127.0.0.1')").replace("proxy.listen(0, '127.0.0.1')","proxy.listen(19457, '127.0.0.1')")
 (dst/p.name).write_text(after,encoding='utf8');changes.append({'source':str(p),'dest':str(dst/p.name),'sourceSHA256':hashlib.sha256(p.read_bytes()).hexdigest(),'executedSHA256':hashlib.sha256(after.encode()).hexdigest(),'changes':'absolute own dependency root / upstream19454 / source19457 / foreign19458 only; no assertions modified'})
(OUT/'redirect-transformations.json').write_text(json.dumps(changes,indent=2))
for port in [19454,19457,19458]:
 with socket.socket() as s:s.bind(('127.0.0.1',port))
with Runtime(ROOT,19454,'redirect-matrix'):
 r.run('redirect-matrix',['node',str(ROOT/'client/node_modules/@playwright/test/cli.js'),'test','--config',str(dst/'config.ts'),'--workers','1','--reporter','line,json','--output',str(dst/'test-results')],dst,{'OH_MAP_REDIRECT_EVIDENCE':str(dst/'evidence'),'PLAYWRIGHT_JSON_OUTPUT_FILE':str(dst/'report.json')})
