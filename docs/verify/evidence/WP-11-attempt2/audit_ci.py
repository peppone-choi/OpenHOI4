import pathlib,json,subprocess,hashlib,zipfile,io
r=pathlib.Path.cwd();o=r/'target/wp11-verify2';p=o/'ui_query.cjs';s=p.read_text().replace("'310'","'23'").replace("snap.state.speed,4","snap.state.speed,5").replace("ledger.final_value,'1.5'","ledger.final_value,'1.5000000002'").replace("ledger.entries.length,2","ledger.entries.length,3").replace("ledger.entries[1].source_key,'probe.mul'","ledger.entries[1].source_key,'probe.bit'");(o/'ui_query_saved.cjs').write_text(s)
(o/'ui-query-saved-run.json').write_text(json.dumps([['own-ui-wire-saved-session',['node','target/wp11-verify2/ui_query_saved.cjs','http://127.0.0.1:3800/']]]))
# Direct API full jobs and original run logs, exact source HEAD.
repo='repos/peppone-choi/OpenHOI4'
for ident in [37536767684,37536767681,37536767724,37536767698]:
 for endpoint in ['', '/jobs?per_page=100','/logs']:
  p=subprocess.run(['gh','api',repo+'/actions/runs/'+str(ident)+endpoint],cwd=r,capture_output=True);assert p.returncode==0,p.stderr
  name=str(ident)+('jobs.json' if endpoint.startswith('/jobs') else 'logs.zip' if endpoint=='/logs' else 'run.json');(o/'ci'/name).write_bytes(p.stdout)
  if not endpoint:
   doc=json.loads(p.stdout);assert doc['head_sha']=='750c2732b00c164ddc252fa26c4984ababd64f35' and doc['conclusion']=='success'
logz=zipfile.ZipFile(o/'ci/37536767684logs.zip');matches={n:logz.read(n).decode('utf-8',errors='replace') for n in logz.namelist() if n.endswith('.txt')}
for osname in ['ubuntu-latest','windows-latest','macos-latest']:
 text='\n'.join(t for n,t in matches.items() if osname in n)
 for case in ['req_sav_01_flush_fault_cleans_temp_and_preserves_export','req_sav_01_sync_fault_cleans_temp_and_preserves_export','req_sav_01_persist_fault_cleans_temp_and_preserves_export','req_sav_01_directory_sync_warning_keeps_committed_new_file']:assert case in text and '... ok' in text
# Inspect source original red/green bundle and digest every member.
d=r/'docs/worklog/evidence/WP-11/p06';m=json.loads((d/'manifest.json').read_text());b=(d/m['archive']).read_bytes();assert hashlib.sha256(b).hexdigest()==m['sha256'];z=zipfile.ZipFile(io.BytesIO(b));assert all(hashlib.sha256(z.read(name)).hexdigest()==sha for name,sha in m['files'].items());assert z.read('red.exit').decode().strip()=='101';assert 'must reach precommit error branch' in z.read('red.log').decode();assert z.read('green-file-tests.exit').decode().strip()=='0';(o/'redgreen-checked.json').write_text(json.dumps({'zip_sha':m['sha256'],'members':len(m['files']),'red_exit':101,'green_exit':0},indent=2))
# Preserve current main's distinct API evidence, not proof of product750c integration.
p=subprocess.run(['gh','api',repo+'/actions/runs?head_sha=3b99b1b4be8eed3e3a3f99e3fd6bfa0f5ce341a5&per_page=30'],cwd=r,capture_output=True);assert p.returncode==0;(o/'ci/main3b99-runs.json').write_bytes(p.stdout)
print('exact 750c all four source CI successes, original logs, per-OS fault tests, red/green member digests verified; main3b99 separately preserved')
