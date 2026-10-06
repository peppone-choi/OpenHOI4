import pathlib,subprocess,json,re,hashlib
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify';out={}
for wp,sha,paths in [('02','eb15633',['crates/oh_core/tests']),('03','70de319',['crates/oh_data/tests/loader.rs','crates/oh_data/tests/fixtures']),('04','88d730a',['crates/oh_sim/tests','crates/oh_cli/tests','tests/golden'])]:
 p=subprocess.run(['git','diff','--exit-code',sha,'HEAD','--',*paths],capture_output=True,text=True);out[wp]={'exit':p.returncode,'diff':p.stdout};assert p.returncode==0,(wp,p.stdout)
source_scan={}
for folder in ['crates/oh_core/src','crates/oh_sim/src','crates/oh_ai/src']:
 for p in (root/folder).rglob('*.rs'):
  hits=[(i,l) for i,l in enumerate(p.read_text(encoding='utf8').splitlines(),1) if re.search(r'\b(f32|f64|HashMap|thread_rng|SystemTime|Instant)\b|std::fs|std::io|tokio|rayon',l)]
  if hits:source_scan[str(p.relative_to(root))]=hits
out['source_scan']=source_scan
for p in ['selftest-original.err.txt','selftest-repeat.err.txt','WP-00.impl.jsonl']:
 text=(root/'docs/worklog/evidence/WP-06'/p).read_text(encoding='utf-8-sig');out[p]={'HTTP400':'400' in text,'model_rejection':'not supported' in text,'turn_failed':'turn.failed' in text}
out['selftest_exit']=(root/'docs/worklog/evidence/WP-06/selftest-repeat.exit.txt').read_text(encoding='utf-8-sig').strip();out['WP00_exit']=(root/'docs/worklog/evidence/WP-06/WP-00.impl.exit.txt').read_text(encoding='utf-8-sig').strip()
trace=json.loads((root/'docs/worklog/evidence/WP-06/app-call-trace.json').read_text(encoding='utf-8-sig'));out['app_trace_file_sha256']=hashlib.sha256((root/'docs/worklog/evidence/WP-06/app-call-trace.json').read_bytes()).hexdigest();out['app_trace_type']=type(trace).__name__
logs=[]
for wp in range(1,7):
 folder=root/f'docs/worklog/evidence/WP-{wp:02}'
 for p in folder.glob('*'):
  if p.is_file() and any(s in p.name for s in ['red','initial','first-implementation','second-implementation']):
   logs.append({'file':str(p.relative_to(root)),'size':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
out['failure_evidence_index']=logs
(ev/'preservation-and-policy-audit.json').write_text(json.dumps(out,ensure_ascii=False,indent=2),encoding='utf8');print(json.dumps({k:v for k,v in out.items() if k!='failure_evidence_index'},ensure_ascii=False,indent=2));print('initial failure log files',len(logs))
