import pathlib,os,sys,subprocess,json,time,datetime,threading,re,urllib.request,hashlib,socket,shutil
from process_cwd import process_cwd
ROOT=pathlib.Path(__file__).resolve().parents[2]; OUT=ROOT/'target/wp08-verify6'
def stamp():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def digest(b):return hashlib.sha256(b).hexdigest()
env=os.environ.copy();env['PATH']=str(pathlib.Path.home()/'.cargo/bin')+os.pathsep+env['PATH'];env['TMP']=env['TEMP']=str(OUT/'temp');(OUT/'temp').mkdir(exist_ok=True)
mode=sys.argv[1] if len(sys.argv)>1 else 'aux';ledger=OUT/(mode+'-commands.json');records=json.loads(ledger.read_text(encoding='utf8')) if ledger.exists() else []
def run(label,args,cwd=ROOT,extra=None,port=None):
 e=env.copy();e.update(extra or {});r={'label':label,'argv':args,'cwd':str(cwd),'start':stamp(),'env':extra};stop=threading.Event()
 args=[shutil.which(args[0],path=e['PATH']) or args[0],*args[1:]]
 if port:
  with socket.socket() as s:s.bind(('127.0.0.1',port))
  r['port_preflight']='free bind; netstat listener checked before HTTP'
 with (OUT/(label+'.log')).open('w',encoding='utf8') as log:
  p=subprocess.Popen(args,cwd=cwd,env=e,stdout=log,stderr=subprocess.STDOUT);r['launcher_pid']=p.pid
  if port:
   def monitor():
    while not stop.is_set():
     listing=subprocess.check_output(['netstat','-ano','-p','tcp'],text=True)
     found=re.findall(r'\sTCP\s+127\.0\.0\.1:'+str(port)+r'\s+\S+\s+LISTENING\s+(\d+)',listing)
     if found:
      pid=int(found[0]);q=json.loads(subprocess.check_output(['powershell.exe','-NoProfile','-Command',f'Get-CimInstance Win32_Process -Filter "ProcessId={pid}" | Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine,CreationDate | ConvertTo-Json'],text=True))
      expected=ROOT/'target/debug/oh_server.exe'
      assert pathlib.Path(q['ExecutablePath']).resolve()==expected.resolve(),q
      q.update({'observation':stamp(),'exe_sha256':digest(expected.read_bytes()),'source_head':subprocess.check_output(['git','--no-optional-locks','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'cwd_contract':str(cwd),'actual_cwd':process_cwd(pid)})
      assert pathlib.Path(q['actual_cwd']).resolve()==cwd.resolve()
      req=urllib.request.urlopen(f'http://127.0.0.1:{port}/');html=req.read();q['HTTP_status']=req.status;q['html_sha256']=digest(html)
      asset=re.search(rb'src="([^"]+\.js)"',html).group(1).decode();b=urllib.request.urlopen(f'http://127.0.0.1:{port}'+asset).read();local=(ROOT/'client/dist'/asset.lstrip('/')).read_bytes();assert b==local
      q['servedJS']={'path':asset,'bytes':len(b),'sha256':digest(b),'equals_own_dist':True}
      try:q['metadata']=json.load(urllib.request.urlopen(f'http://127.0.0.1:{port}/maps/testland/metadata'))
      except Exception as x:q['metadata_error']=str(x)
      r['runtime']=q;(OUT/(label+'-runtime.json')).write_text(json.dumps(r,indent=2),encoding='utf8');return
     stop.wait(.25)
   t=threading.Thread(target=monitor);t.start()
  r['exit']=p.wait();stop.set()
  if port:
   t.join();r['port_postflight']=[line for line in subprocess.check_output(['netstat','-ano','-p','tcp'],text=True).splitlines() if re.search(r':'+str(port)+r'\s',line) and 'LISTENING' in line]
  r['end']=stamp();records.append(r);(OUT/(mode+'-commands.json')).write_text(json.dumps(records,indent=2),encoding='utf8');print(label,r['exit'],flush=True)
 return r['exit']
if mode in ['m1','m0']:
 port=19453 if mode=='m1' else 19454
 extra={'OH_TEST_PORT':str(port),'OH_BROWSER_PRODUCTS':'1','PLAYWRIGHT_JSON_OUTPUT_FILE':str(OUT/(mode+'-report.json'))}
 for k,v in [('OH_E2E_EVIDENCE','e2e'),('OH_MALFORMED_EVIDENCE','malformed'),('OH_MAP_EVIDENCE','map'),('OH_MAP_REDIRECT_EVIDENCE','redirect'),('OH_MAP_CAPABILITY_EVIDENCE','capability'),('OH_MAP_RESIZE_EVIDENCE','resize'),('OH_MAP_PIPELINE_EVIDENCE','pipeline'),('OH_MAP_DPR_EVIDENCE','dpr')]:extra[k]=str(OUT/mode/v)
 args=['node',str(ROOT/'client/node_modules/@playwright/test/cli.js'),'test','--reporter=line,json','--output',str(OUT/mode/'test-results')]
 if mode=='m1':args+=['--config','playwright.m1.config.ts']
 sys.exit(run(mode,args,ROOT/'client',extra,port))
if mode=='checks':
 for label,args in [('fmt',['cargo','fmt','--check']),('clippy',['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings']),('workspace-test',['cargo','test','--workspace','--locked']),('typecheck',['npm.cmd','--prefix','client','run','typecheck']),('npm-licenses',['npm.cmd','--prefix','client','run','licenses']),('cargo-deny',['cargo','deny','check','licenses','bans','advisories','sources']),('assets',['python','tools/check_assets.py','--release']),('docs',['python','tools/check_docs.py']),('architecture',['python','tools/check_architecture.py']),('policy-fixtures',['python','-m','unittest','discover','-s','tools/tests','-p','test_wp01.py','-v'])]:run(label,args)
if mode=='oldbuild':
 for name in ['red62','red953']:
  cwd=OUT/(name+'-source')
  for label,args in [('npm-ci',['npm.cmd','--prefix','client','ci']),('client-build',['npm.cmd','--prefix','client','run','build']),('server-build',['cargo','build','-p','oh_server','--locked'])]:run(name+'-'+label,args,cwd)
if mode=='hashes':
 for n in [1,2]:
  run('m0-hash'+str(n),['cargo','run','-p','oh_cli','--locked','--','run','--scenario','testland','--days','365','--seed','1','--hash-out'])
  run('m1-hash'+str(n),['cargo','run','-p','oh_cli','--locked','--','run','--pack','data/packs/testland','--scenario','m1','--days','365','--seed','1','--hash-out'])
 run('m1-capture',['python','crates/oh_sim/tools/check_sim_determinism.py','capture','--m1','--out',str(OUT/'m1-sim-determinism')])
