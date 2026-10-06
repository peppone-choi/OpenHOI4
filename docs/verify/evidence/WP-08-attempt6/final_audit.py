import pathlib,json,hashlib,subprocess,os,datetime
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'target/wp08-verify6'
def read(path):return json.loads(path.read_text(encoding='utf-8-sig'))
a=read(OUT/'before.json');b=read(OUT/'after.json');same={k:a[k]==b[k] for k in a if k!='time'};assert all(same.values())
result={'head':b['head'],'trackedCount':b['tracked_count'],'beforeAfter':same,'rawIndexSHA256':b['raw_index_sha256'],'semanticIndexSHA256':b['semantic_index_sha256'],'trackedListSHA256':b['tracked_list_sha256'],'trackedFileSHAs':'before.json / after.json','ownPortsListen':[line for line in subprocess.check_output(['netstat','-ano','-p','tcp'],text=True).splitlines() if any(f':{p} ' in line for p in [19453,19454,19455,19456,19457,19458]) and 'LISTENING' in line],'runtime':[{'label':p.stem,**{k:read(p).get(k) for k in ['pid','port','exeSHA256','actual_cwd','start','end','exit','portReleased']}} for p in OUT.glob('*-runtime.json') if 'pid' in read(p)],'originals':{},'cases':{},'fixtureLimitations':[]}
assert not result['ownPortsListen']
for path in [OUT/'m1-report.json',OUT/'m0-report.json',OUT/'redirect-matrix/report.json']:
 r=read(path);assert r['stats']['unexpected']==0 and r['stats']['skipped']==0 and r['stats']['flaky']==0;result['cases'][str(path.relative_to(OUT))]=r['stats']
for name in ['native-lifecycle-results.json','live-pipeline-results.json','atomic-results.json','initialization-results.json','gpu-compile-results.json']:
 r=read(OUT/'current-probes'/name);assert all(x['pass'] for x in r);result['cases'][name]=len(r)
r=read(OUT/'compile-dpr-results.json');assert all(x['pass'] for x in r);result['cases']['compile-dpr']=len(r)
for directory in [OUT/'prior-attempt/verify3-originals',OUT/'prior-attempt']:
 r=read(directory/'SHA256SUMS.json');existing={p:sha for p,sha in r.items() if (directory/p).is_file()};bad=[p for p,sha in existing.items() if hashlib.sha256((directory/p).read_bytes()).hexdigest()!=sha];assert not bad;result['originals'][str(directory.relative_to(OUT))]={'manifestEntries':len(r),'presentChecked':len(existing),'missingFromProvidedSubset':len(r)-len(existing),'mismatches':bad}
result['fixtureLimitations']=[{'fixture':'first policy fixtures','exit':1,'scope':'31 passed/1 failure: Cargo standalone fixture placed below product workspace; missing diagnostic because metadata rejected parent membership. Same original32 assertions PASS after [workspace] in generated isolated fixture only; no product settings change.'},{'fixture':'current original lifecycle','exit':1,'scope':'original preserved; preferred chrome/edge + Firefox/WebKit six PASS; four GL cases fail after locator screenshot restores configuredDPR1. Native screenshot supplemental10 PASS. Original failed evidence not replaced.'},{'fixture':'original current250ms probe','exit':0,'scope':'no pixel assertions in original; raw0/clear during epoch transition retained; separate5s-ready same-canvas/camera/box oracle4 PASS.'},{'fixture':'native-life preflight attempt','scope':'19453 occupied by own still-running atomic suite, bind10048 detected before process/HTTP/browser; did not navigate. Retried after owner suite completed/released.'},{'fixture':'IAB version inspection','scope':'raw CDP Browser.getVersion unsupported and readonly page scope lacks navigator; tool inspection errors, no app JS error. Exact IAB version unavailable; native five product versions independently recorded.'}]
result['endUTC']=datetime.datetime.now(datetime.timezone.utc).isoformat();(OUT/'final-audit.json').write_text(json.dumps(result,indent=2),encoding='utf8')
files={}
for root,dirs,names in os.walk(OUT):
 dirs[:]=[d for d in dirs if d not in ['node_modules','__pycache__'] and not (d=='target' and pathlib.Path(root).name in ['red62-source','red953-source'])]
 for name in names:
  p=pathlib.Path(root)/name
  if name=='EVIDENCE_SHA256SUMS.json':continue
  files[p.relative_to(OUT).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
(OUT/'EVIDENCE_SHA256SUMS.json').write_text(json.dumps(files,indent=2),encoding='utf8');print('PASS integrity, evidence files',len(files));print(json.dumps({k:v for k,v in result.items() if k not in ['runtime','fixtureLimitations']},indent=2))
