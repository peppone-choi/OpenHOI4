import pathlib,json,subprocess,hashlib,re,datetime
root=pathlib.Path(r'E:/openhoi/.orchestrator/wt/WP-10-verify');out=root/'target/verify/WP-10'
def git(*args):return subprocess.check_output(['git',*args],cwd=root)
files=git('ls-files','-z').decode().split('\0')[:-1]
state={'head':git('rev-parse','HEAD').decode().strip(),'status':git('status','--porcelain=v1').decode(),'diff':git('diff').decode(),'index_diff':git('diff','--cached').decode(),'files':{p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in files},'index':git('ls-files','--stage').decode()}
(out/'after.json').write_text(json.dumps(state,ensure_ascii=False,indent=2),encoding='utf8')
before=json.loads((out/'before.json').read_text(encoding='utf8'))
keys=['head','status','diff','index_diff','files','index']
checks={k:before[k]==state[k] for k in keys}
checks['tracked_count']=len(files);checks['status_empty']=state['status']=='';checks['expected_head']=state['head']=='44cc3ea9a38d5c0450ad782635d59cbc88f57ce9'
checks['time_kst']=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).isoformat()
(out/'integrity.json').write_text(json.dumps(checks,ensure_ascii=False,indent=2),encoding='utf8')
assert all(checks[k] for k in keys) and checks['status_empty'] and checks['expected_head']
summary={'initial_checks':json.loads((out/'results.json').read_text()),'retry_deny_exit':0 if 'exit: 0' in (out/'deny-retry.log').read_text(encoding='utf-8-sig') else None,'probe_exit':0 if 'INDEPENDENT PROBE PASS' in (out/'probe.log').read_text() else None,'integrity':checks,'workspace_passed':sum(map(int,re.findall(r'test result: ok\. (\d+) passed;', (out/'workspace-tests.log').read_text())))}
(out/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2),encoding='utf8')
print(json.dumps(checks,ensure_ascii=False));print('workspace test cases=',summary['workspace_passed'])
