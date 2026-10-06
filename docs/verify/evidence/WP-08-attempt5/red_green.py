import run_checks as m,hashlib,json
from own_runtime import Runtime
changes=[]
for label,root in [('red62',m.OUT/'red62-source'),('current',m.ROOT)]:
 out=m.OUT/(label+'-original-probes');out.mkdir(exist_ok=True)
 for name in ['resize-probe.cjs','gpu-compile-failure.cjs','initialization.cjs']:
  source=m.OUT/'prior-attempt/verify3'/name;original=source.read_text(encoding='utf-8')
  amended=original.replace('../../client/node_modules/@playwright/test',str(root/'client/node_modules/@playwright/test').replace('\\','/')).replace('../../client/e2e-m1/png.ts',(root/'client/e2e-m1/png.ts').as_uri()).replace('127.0.0.1:19428','127.0.0.1:19443').replace('127.0.0.1:19427','127.0.0.1:19443')
  (out/name).write_text(amended,encoding='utf-8');changes.append({'label':label,'name':name,'originalSHA':hashlib.sha256(source.read_bytes()).hexdigest(),'executedSHA':hashlib.sha256((out/name).read_bytes()).hexdigest(),'scope':'only absolute dependency/import paths + own port19443; assertions and timeouts unchanged'})
 (m.OUT/'original-probe-transformations.json').write_text(json.dumps(changes,indent=2))
 with Runtime(root,19443,label):
  for name in ['resize-probe.cjs','gpu-compile-failure.cjs']+(['initialization.cjs'] if label=='current' else []):m.run(label+'-'+name,['node',str(out/name)])
 steps=json.loads((out/'resize-results.json').read_text());green=all(s['rgb']==[40,100,180] and s['gpu']['pixel']==[40,100,180,255] and not s['errors'] for s in steps)
 print(label,'resize visible oracle',green,[(s['label'],s['rgb'],s['gpu']['pixel']) for s in steps],flush=True)
 (out/'resize-oracle.json').write_text(json.dumps({'expected':[40,100,180],'pass':green,'rawOriginalScriptExitNotOracle':True,'steps':steps},indent=2))
