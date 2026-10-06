from pathlib import Path
import datetime,hashlib,json,os,re,socket,subprocess,sys,threading,urllib.request
from checks import SOURCE_HEAD,git_read,tracked_state,require_same_bytes,dist_asset_path
out=Path(__file__).resolve().parent;root=out.parents[1];assert sys.platform=='linux';assert out==root/'target/wp08-dpr-ci2';results=[]
before=tracked_state(root);assert before['head']==SOURCE_HEAD and not before['status'];(out/'product-state-before.json').write_text(json.dumps(before,indent=2)+'\n',encoding='utf-8')
assert before==json.loads((out/'preparation-product-state-after.json').read_text(encoding='utf-8')),'Source changed between preparation and execution, including raw index'
exe=root/'target/debug/oh_server';expected_exe=exe.resolve();exe_bytes=exe.read_bytes();exe_sha=hashlib.sha256(exe_bytes).hexdigest();status=git_read(root,'status','--porcelain').decode();assert not status,status
for mode,port in [('warmup-probe',19471),('probe',19471),('warmup-candidate',19472),('candidate',19472)]:
    with socket.socket() as sock:sock.bind(('127.0.0.1',port))
    dest=out/('linux-'+mode);assert not dest.exists();dest.mkdir();env=os.environ.copy();env.pop('OH_BROWSER_PRODUCTS',None)
    env.update(OH_TEST_PORT=str(port),OH_DPR_PROFILE=str(dest))
    for name in ['OH_MAP_CAPABILITY_EVIDENCE','OH_MAP_RESIZE_EVIDENCE','OH_MAP_PIPELINE_EVIDENCE','OH_MAP_EVIDENCE','OH_MAP_REDIRECT_EVIDENCE','OH_MALFORMED_EVIDENCE','OH_E2E_EVIDENCE','OH_MAP_DPR_EVIDENCE']:env[name]=str(dest/'png'/name.lower())
    info={'mode':mode,'port':port,'exe_sha256':exe_sha,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'tracked_status_before':status,'runtime_identity_confirmed':False};stop=threading.Event()
    def capture():
        base='http://127.0.0.1:'+str(port)
        while not stop.wait(.2):
            html_received=False
            try:
                html=urllib.request.urlopen(base,timeout=1).read();html_received=True;match=re.search(rb'src="(/assets/[^" ]+\.js)"',html);assert match,'Served HTML has no expected asset';asset=match.group(1).decode();data=urllib.request.urlopen(base+asset,timeout=3).read()
                info['served_js']={'path':asset,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()};info['served_html_sha256']=hashlib.sha256(html).hexdigest()
                dist=dist_asset_path(root/'client/dist',asset);assert dist.is_file(),('Missing source dist asset',str(dist));info['served_js']['source_dist_path']=str(dist);info['served_js']['byte_comparison']=require_same_bytes(data,dist.read_bytes(),'served JS vs exact source dist')
                response=subprocess.run(['ss','-ltnp','sport = :'+str(port)],capture_output=True,text=True);info['listener']={'exit':response.returncode,'stdout':response.stdout,'stderr':response.stderr};assert response.returncode==0,response.stderr;pids=set(re.findall(r'pid=(\d+)',response.stdout));assert len(pids)==1,('Require one actual listener PID',pids);pid=int(next(iter(pids)));proc_exe=Path('/proc/'+str(pid)+'/exe');actual=Path(os.readlink(proc_exe)).resolve();assert actual==expected_exe,(actual,expected_exe)
                running_bytes=proc_exe.read_bytes();info['process']={'pid':pid,'exe':str(actual),'proc_exe_path':str(proc_exe),'byte_comparison':require_same_bytes(running_bytes,exe_bytes,'running /proc/PID/exe vs exact source build'),'cmdline':Path('/proc/'+str(pid)+'/cmdline').read_bytes().replace(b'\0',b' ').decode()};info['runtime_identity_confirmed']=True;break
            except AssertionError as error:info['identity_mismatch']=str(error);break
            except Exception as error:
                info['capture_last_error']=str(error)
                if html_received:break
        (dest/'runtime-identity.json').write_text(json.dumps(info,indent=2)+'\n',encoding='utf-8')
    thread=threading.Thread(target=capture);thread.start()
    command=['node','node_modules/@playwright/test/cli.js','test','--config','playwright.config.ts'] if mode.startswith('warmup') else ['node','node_modules/@playwright/test/cli.js','test','--config',str(out/('full-'+mode+'.config.cjs')),'--project=chromium','--project=webkit']
    info['command']=command;info['scope']='Original M0 36-case warmup on unchanged original config/server pack' if mode.startswith('warmup') else 'Full M1 82-case headless config with ignored DPR instrumentation/candidate; same preceding original M0 warmup per arm'
    with (dest/'command.log').open('w',encoding='utf-8') as log:result=subprocess.run(command,cwd=root/'client',env=env,stdout=log,stderr=subprocess.STDOUT)
    info['real_exit_code']=result.returncode;stop.set();thread.join();info['tracked_status_after']=git_read(root,'status','--porcelain').decode()
    if mode.startswith('warmup'):
        log_text=(dest/'command.log').read_text(encoding='utf-8');info['observed_pass_counts']=[int(n) for n in re.findall(r'(\d+) passed \(',log_text)];info['expected_case_count_confirmed']=result.returncode==0 and info['observed_pass_counts']==[36]
    else:
        report=json.loads((dest/'report.json').read_text(encoding='utf-8')) if (dest/'report.json').is_file() else {};info['report_stats']=report.get('stats');stats=report.get('stats',{});info['expected_case_count_confirmed']=result.returncode==0 and stats.get('expected')==82 and all(stats.get(key)==0 for key in ['skipped','unexpected','flaky'])
    arm_state=tracked_state(root);(dest/'product-state-after-arm.json').write_text(json.dumps(arm_state,indent=2)+'\n',encoding='utf-8');info['product_state_unchanged_after_arm']=before==arm_state
    (dest/'runtime-identity.json').write_text(json.dumps(info,indent=2)+'\n',encoding='utf-8')
    with (dest/'summary.log').open('w',encoding='utf-8') as log:summary=subprocess.run(['python',str(out/'summarize_profiles.py'),str(dest)],stdout=log,stderr=subprocess.STDOUT)
    info['summary_exit']=summary.returncode;results.append(info);print(json.dumps(info,indent=2))
    if not info['product_state_unchanged_after_arm']:break
after=tracked_state(root);(out/'product-state-after.json').write_text(json.dumps(after,indent=2)+'\n',encoding='utf-8');unchanged=before==after
(out/'linux-run-result.json').write_text(json.dumps({'results':results,'product_state_unchanged':unchanged},indent=2)+'\n',encoding='utf-8');sys.exit(0 if unchanged and len(results)==4 and all(r['real_exit_code']==0 and r['runtime_identity_confirmed'] and r['expected_case_count_confirmed'] and r['product_state_unchanged_after_arm'] and not r['tracked_status_after'] and r['summary_exit']==0 for r in results) else 1)
