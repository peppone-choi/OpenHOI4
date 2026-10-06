import pathlib,hashlib,json
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'target/wp08-verify6';prior=OUT/'prior-attempt';changes=[]
def copy(src,dst,replacements):
 original=src.read_text(encoding='utf-8-sig');amended=original
 for a,b in replacements:amended=amended.replace(a,b)
 dst.parent.mkdir(parents=True,exist_ok=True);dst.write_text(amended,encoding='utf8')
 changes.append({'source':str(src),'executed':str(dst),'originalSHA':hashlib.sha256(src.read_bytes()).hexdigest(),'executedSHA':hashlib.sha256(dst.read_bytes()).hexdigest(),'replacements':replacements})
for label,root,port in [('red62',OUT/'red62-source',19455),('red953',OUT/'red953-source',19456),('current',ROOT,19453)]:
 dest=OUT/(label+'-probes')
 for name in ['resize-probe.cjs','gpu-compile-failure.cjs','initialization.cjs']:
  if label=='red953':continue
  copy(prior/'verify3-originals'/name,dest/name,[('../../client/node_modules/@playwright/test',str(root/'client/node_modules/@playwright/test').replace('\\','/')),('../../client/e2e-m1/png.ts',(root/'client/e2e-m1/png.ts').as_uri()),('127.0.0.1:19428','127.0.0.1:'+str(port)),('127.0.0.1:19427','127.0.0.1:'+str(port))])
 for name in ['ready-probe.cjs','live-pipeline.cjs','atomic.cjs']:
  if label=='red953' or (label=='red62' and name!='ready-probe.cjs'):continue
  copy(prior/'verify5-extra'/name,dest/name,[("path.resolve(__dirname,'../..')",repr(root.as_posix())),("require('node:path').resolve(__dirname,'../..')",repr(root.as_posix())),('127.0.0.1:19444','127.0.0.1:'+str(port)),('127.0.0.1:19443','127.0.0.1:'+str(port))])
 if label in ['red953','current']:
  copy(prior/'lifecycle.cjs',dest/'lifecycle.cjs',[("path.resolve(__dirname,'../..')",repr(root.as_posix())),('127.0.0.1:19443','127.0.0.1:'+str(port))])
(OUT/'probe-transformations.json').write_text(json.dumps(changes,indent=2),encoding='utf8');print('Prepared',len(changes),'scripts; path/port only transformations')
