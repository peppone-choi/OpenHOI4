from pathlib import Path
import datetime,hashlib,json,os,re,subprocess,sys,threading,urllib.request
from checks import SOURCE_HEAD,git_read,tracked_state,require_same_bytes,dist_asset_path

out=Path(__file__).resolve().parent;root=out.parents[1]
assert sys.platform=='linux' and out==root/'target/wp08-dpr-ci2'

def save(path,data):
    path.write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')

def port_observation(port):
    observations={}
    for label,command in [('listeners',['ss','-Hltnp','sport = :'+str(port)]),('tcp_all',['ss','-tanp','( sport = :'+str(port)+' or dport = :'+str(port)+' )'])]:
        result=subprocess.run(command,capture_output=True,text=True)
        observations[label]={'command':command,'exit':result.returncode,'stdout':result.stdout,'stderr':result.stderr}
    return observations

def require_no_listener(observations):
    assert all(v['exit']==0 for v in observations.values()),'Port observation failed'
    assert not observations['listeners']['stdout'].strip(),'Active listener: fail, never reuse or kill it'

def capture(info,stop,port,expected_exe,exe_bytes):
    base='http://127.0.0.1:'+str(port)
    while not stop.wait(.2):
        html_received=False
        try:
            html=urllib.request.urlopen(base,timeout=1).read();html_received=True
            match=re.search(rb'src="(/assets/[^" ]+\.js)"',html);assert match,'Missing served asset'
            asset=match.group(1).decode();data=urllib.request.urlopen(base+asset,timeout=3).read()
            dist=dist_asset_path(root/'client/dist',asset);assert dist.is_file(),'Missing source dist asset'
            info['served_js']={'path':asset,'source_dist_path':str(dist),'byte_comparison':require_same_bytes(data,dist.read_bytes(),'served JS vs source dist')}
            info['served_html_sha256']=hashlib.sha256(html).hexdigest()
            listener=port_observation(port);info['listener_at_identity']=listener
            assert all(v['exit']==0 for v in listener.values())
            pids=set(re.findall(r'pid=(\d+)',listener['listeners']['stdout']));assert len(pids)==1,pids
            pid=int(next(iter(pids)));proc=Path('/proc/'+str(pid)+'/exe');actual=Path(os.readlink(proc)).resolve()
            assert actual==expected_exe,(actual,expected_exe)
            info['process']={'pid':pid,'exe':str(actual),'cmdline':Path('/proc/'+str(pid)+'/cmdline').read_bytes().replace(b'\0',b' ').decode(),'byte_comparison':require_same_bytes(proc.read_bytes(),exe_bytes,'running exe vs source build')}
            info['runtime_identity_confirmed']=True;return
        except AssertionError as error:info['identity_error']=str(error);return
        except Exception as error:
            info['capture_last_error']=str(error)
            if html_received:return

def guarded_capture(info,*args):
    try:capture(info,*args)
    except BaseException as failure:
        info['observer_thread_error']=repr(failure);info['runtime_identity_confirmed']=False

def own_process_after(info):
    process=info.get('process')
    if not process:return {'captured':False,'cleanup_verified':False}
    pid=process['pid'];proc=Path('/proc/'+str(pid));record={'captured':True,'pid':pid,'pid_exists':proc.exists()}
    if not proc.exists():record['cleanup_verified']=True;return record
    try:
        record['exe']=os.readlink(proc/'exe');record['cmdline']=(proc/'cmdline').read_bytes().replace(b'\0',b' ').decode()
        record['same_process_identity']=record['exe']==process['exe'] and record['cmdline']==process['cmdline']
    except OSError as error:record['read_error']=str(error)
    record['cleanup_verified']=False
    return record

before=None;results=[];error=None;exit_code=1
planned=[('warmup-probe',19471),('probe',19472),('warmup-candidate',19473),('candidate',19474)]
try:
    before=tracked_state(root);save(out/'product-state-before.json',before)
    assert before['head']==SOURCE_HEAD and not before['status']
    assert before==json.loads((out/'preparation-product-state-after.json').read_text(encoding='utf-8'))
    fixture_inputs=json.loads((out/'fixture-inputs.json').read_text(encoding='utf-8'));assert fixture_inputs['source_head']==SOURCE_HEAD
    assert {Path(x['path']).name for x in fixture_inputs['files']}=={'ledger-ko.html','ledger-en.html'}
    for entry in fixture_inputs['files']:
        path=Path(entry['path']).resolve();assert path.is_relative_to((root/'target/wp12').resolve())
        data=path.read_bytes();assert len(data)==entry['bytes'] and hashlib.sha256(data).hexdigest()==entry['sha256']
    expected_exe=(root/'target/debug/oh_server').resolve();exe_bytes=expected_exe.read_bytes()
    # Fresh ports per segment; checking previous port/process cleanup remains mandatory.
    for mode,port in planned:
        dest=out/('linux-'+mode);assert not dest.exists();dest.mkdir()
        info={'mode':mode,'port':port,'source_head':SOURCE_HEAD,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'runtime_identity_confirmed':False,'test_started':False}
        stop=threading.Event();thread=None;segment_error=None
        try:
            info['port_before']=port_observation(port);save(dest/'port-before.json',info['port_before']);require_no_listener(info['port_before'])
            env=os.environ.copy();env.pop('OH_BROWSER_PRODUCTS',None);env.update(OH_TEST_PORT=str(port),OH_DPR_PROFILE=str(dest))
            for key in ['OH_MAP_CAPABILITY_EVIDENCE','OH_MAP_RESIZE_EVIDENCE','OH_MAP_PIPELINE_EVIDENCE','OH_MAP_EVIDENCE','OH_MAP_REDIRECT_EVIDENCE','OH_MALFORMED_EVIDENCE','OH_E2E_EVIDENCE','OH_MAP_DPR_EVIDENCE']:env[key]=str(dest/'png'/key.lower())
            command=['node','node_modules/@playwright/test/cli.js','test','--config','playwright.config.ts'] if mode.startswith('warmup') else ['node','node_modules/@playwright/test/cli.js','test','--config',str(out/('full-'+mode+'.config.cjs')),'--project=chromium','--project=webkit']
            info['command']=command;thread=threading.Thread(target=guarded_capture,args=(info,stop,port,expected_exe,exe_bytes));thread.start();info['test_started']=True
            with (dest/'command.log').open('w',encoding='utf-8') as log:result=subprocess.run(command,cwd=root/'client',env=env,stdout=log,stderr=subprocess.STDOUT)
            info['real_exit_code']=result.returncode
            if mode.startswith('warmup'):
                observed=[int(n) for n in re.findall(r'(\d+) passed \(',(dest/'command.log').read_text(encoding='utf-8'))];info['observed_pass_counts']=observed;info['expected_case_count_confirmed']=result.returncode==0 and observed==[36];info['suite_coverage_confirmed']=info['expected_case_count_confirmed']
            else:
                stats=json.loads((dest/'report.json').read_text(encoding='utf-8')).get('stats',{});info['report_stats']=stats
                info['expected_case_count_confirmed']=result.returncode==0 and stats.get('expected')==82 and all(stats.get(k)==0 for k in ['skipped','unexpected','flaky'])
                info['suite_coverage_confirmed']=stats.get('expected',0)+stats.get('unexpected',0)==82 and all(stats.get(k)==0 for k in ['skipped','flaky'])
            with (dest/'summary.log').open('w',encoding='utf-8') as log:summary=subprocess.run(['python',str(out/'summarize_profiles.py'),str(dest)],stdout=log,stderr=subprocess.STDOUT)
            info['summary_exit']=summary.returncode
        except BaseException as failure:segment_error=repr(failure);info['segment_error']=segment_error
        finally:
            stop.set()
            if thread and thread.ident is not None:
                try:thread.join()
                except BaseException as failure:
                    info['observer_finalization_error']=repr(failure);info['runtime_identity_confirmed']=False
            try:
                info['port_after']=port_observation(port);save(dest/'port-after.json',info['port_after']);require_no_listener(info['port_after']);info['port_listen_closed']=True
            except BaseException as failure:info['cleanup_error']=repr(failure);info['port_listen_closed']=False
            info['own_process_after']=own_process_after(info)
            try:
                arm_after=tracked_state(root);save(dest/'product-state-after-arm.json',arm_after);info['product_state_unchanged_after_arm']=before==arm_after
            except BaseException as failure:info['state_after_error']=repr(failure);info['product_state_unchanged_after_arm']=False
            snapshot=dict(info);save(dest/'runtime-identity.json',snapshot);results.append(snapshot)
            save(out/'linux-run-result.json',{'results':results,'pending_final_source_check':True})
        # A valid full M1 assertion failure is comparison data: keep the already
        # planned next arm once, not a retry. Failed warmup/preparation/ownership/
        # cleanup/source or incomplete coverage aborts the remaining plan.
        if segment_error or not info.get('suite_coverage_confirmed') or not info['runtime_identity_confirmed'] or not info['product_state_unchanged_after_arm'] or not info.get('port_listen_closed') or not info['own_process_after']['cleanup_verified'] or info.get('summary_exit')!=0:break
except BaseException as failure:error=repr(failure)
finally:
    after=None;after_error=None
    try:after=tracked_state(root);save(out/'product-state-after.json',after)
    except BaseException as failure:after_error=repr(failure)
    unchanged=before is not None and before==after
    complete=unchanged and not error and len(results)==4 and all(r.get('real_exit_code')==0 and r.get('runtime_identity_confirmed') and r.get('expected_case_count_confirmed') and r.get('product_state_unchanged_after_arm') and r.get('port_listen_closed') and r.get('own_process_after',{}).get('cleanup_verified') and r.get('summary_exit')==0 for r in results)
    exit_code=0 if complete else 1
    remaining=[mode for mode,port in planned if mode not in {r['mode'] for r in results}]
    save(out/'linux-run-result.json',{'results':results,'product_state_unchanged':unchanged,'global_error':error,'source_after_error':after_error,'all_four_segments_finished':len(results)==4,'remaining_segments_unexecuted':remaining,'exit_code':exit_code})
sys.exit(exit_code)
