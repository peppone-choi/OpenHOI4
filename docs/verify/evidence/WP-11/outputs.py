from run import *
import re,socket
keys=['OH_E2E_EVIDENCE','OH_MALFORMED_EVIDENCE','OH_MAP_EVIDENCE','OH_MAP_REDIRECT_EVIDENCE','OH_MAP_CAPABILITY_EVIDENCE','OH_MAP_RESIZE_EVIDENCE','OH_MAP_PIPELINE_EVIDENCE','OH_MAP_DPR_EVIDENCE']
tracked=[(ROOT/p).resolve() for p in BASE['tracked_list'] if p]
def guard(p):
 p=pathlib.Path(p).resolve()
 assert not any(p==t or p in t.parents for t in tracked),str(p)
 r=subprocess.run(['git','--no-optional-locks','-C',str(ROOT),'check-ignore',str(p).replace('\\','/')+'/'],capture_output=True,env=os.environ)
 assert r.returncode==0,(p,r.returncode,r.stderr)
 return dict(absolute=str(p),tracked_overlap=False,check_ignore_exit=r.returncode,ignore_output=r.stdout.decode().strip())
if __name__=='__main__':
 rows=[];producers=[];envs={}
 for suite in ['m0','m1']:
  e=os.environ.copy();e['OH_BROWSER_PRODUCTS']='1'
  s=socket.socket();s.bind(('127.0.0.1',0));e['OH_TEST_PORT']=str(s.getsockname()[1]);s.close()
  for k in keys:e[k]=str((OUT/'e2e'/suite/k).resolve())
  node=subprocess.check_output(['node','-e','console.log(JSON.stringify({cwd:process.cwd(),env:Object.fromEntries(Object.entries(process.env).filter(([k])=>k.startsWith("OH_"))),resolved:Object.fromEntries(Object.entries(process.env).filter(([k])=>k.endsWith("_EVIDENCE")).map(([k,v])=>[k,require("node:path").resolve(v)]))}))'],cwd=ROOT/'client',env=e)
  (OUT/(suite+'-node-env.json')).write_bytes(node);envs[suite]={k:e[k] for k in keys+['OH_BROWSER_PRODUCTS','OH_TEST_PORT']}
  for folder in [ROOT/'client/e2e',ROOT/'client/e2e-m1',ROOT/'client/src']:
   for p in folder.rglob('*.ts*'):
    text=p.read_text(encoding='utf-8')
    for n,line in enumerate(text.splitlines(),1):
     if re.search(r'writeFile|mkdir|screenshot|toHaveScreenshot|toMatchSnapshot',line):producers.append(dict(source=str(p.relative_to(ROOT)),line=n,text=line))
    for k,default in re.findall(r"process\.env\.(OH_\w+_EVIDENCE)\s*\?\?\s*'([^']+)'",text):
     rows.append(dict(suite=suite,producer=str(p.relative_to(ROOT)),env=k,default=default,actual=e[k],cwd=str(ROOT/'client'),**guard(e[k])))
  for p in ['target/wp05','target/wp09','target/wp12','client/test-results','client/playwright-report','client/dist','client/node_modules','target/wp11-independent','target/wp11-independent/generated','target/wp11-independent/harness','target/wp11-independent/tmp','target/wp11-independent/native','target/wp11-independent/server','target/debug']:
   rows.append(dict(suite=suite,env='fixed',default=p,actual=str(ROOT/p),cwd=str(ROOT/'client'),**guard(ROOT/p)))
 try:guard(ROOT/'docs/worklog/evidence/WP-12');raise AssertionError('negative not rejected')
 except AssertionError as error:negative=str(error)
 (OUT/'Effective-output-paths.json').write_text(json.dumps(dict(rows=rows,producer_calls=producers,negative_rejected=negative,envs=envs),indent=2),encoding='utf-8')
 (OUT/'Effective-output-paths.md').write_text('|suite|producer/env|default|actual absolute|cwd|ignored|\n|---|---|---|---|---|---|\n'+'\n'.join('|'+ '|'.join([r['suite'],r.get('producer',r['env'])+'/'+r['env'],r['default'],r['absolute'],r['cwd'],'exit0; no tracked overlap'])+'|' for r in rows)+'\nNegative tracked directory rejected: '+negative,encoding='utf-8')
 print(json.dumps(dict(rows=len(rows),producer_calls=len(producers),negative_rejected=negative,envs=envs)))
