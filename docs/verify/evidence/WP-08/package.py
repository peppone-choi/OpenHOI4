import pathlib,hashlib,json,zipfile,socket
root=pathlib.Path(__file__).resolve().parent
(root/'bootstrap-and-helper-notes.json').write_text(json.dumps({
 'initialDelegationWasErrorOutput':True,
 'beforeCorrectedDelegation':{'cwd':'E:/openhoi','actions':['read Get-Location / git status','read AGENTS/HANDOFF/plans/prompts/app-threads/worktree list/log','read source thread history','read verify2 draft and prepared prior-attempt files'],'mutations':[],'newSessions':[],'processActions':[]},
 'correctedDelegationAllProductActionsCwd':'E:/openhoi/.orchestrator/wt/WP-08-verify2',
 'initialToolDiscoveries':['cargo not on PATH: used installed C:/Users/user/.cargo/bin explicitly and process-local PATH only','tools/check_sim_determinism.py absent: found actual crates/oh_sim/tools/check_sim_determinism.py','m1-capture/hash-1.txt/hash-2.txt absent: actual files run-1.txt/run-2.txt/sim-hash.txt','data/packs/testland/map / maps/provinces.csv absent: found maps/testland/provinces.csv'],
 'auditFirstExit1':'P06 original CJS files use mixed ending: only last LF must restore CRLF. Prior raw copies exactly match manifest, normalized content equal. audit-final verifies exact known byte transform.',
 'httpHelperFirstExit1':'http.py shadowed stdlib http.client; renamed ignored helper http_oracle.py, assertions unchanged; final exit0.',
 'oldNativeExpectedExit1':'same unchanged final source fixture against git archive exact f58 commit: six rejected-boundary assertions fail; four cross-origin traces all foreign GET1; initial metadata/index and reload index canvas1; metadata reload still alerted but foreign GET1.',
 'manualServerShutdown':{'port':19415,'session':88360,'action':'Ctrl+C own temporary verification server','execWrapperExit':1,'stdout':'Shutting down OpenHOI.\nOpenHOI stopped normally.\n','notClaimedServerNativeExit0':True,'independentLifecycleTest':'actual native exit0 verified separately by lifecycle.py'},
 'primarySources':[{'url':'https://fetch.spec.whatwg.org/#http-redirect-fetch','checkedDate':'2026-10-07','finding':'redirect error rejects redirect as network error'},{'url':'https://developer.mozilla.org/en-US/docs/Web/API/Request/mode','checkedDate':'2026-10-07','finding':'same-origin requests error on another origin'},{'url':'https://threejs.org/docs/pages/WebGPURenderer.html','checkedDate':'2026-10-07','finding':'preferred WebGPU and WebGL2 fallback; installed pinned code also inspected'}]
},ensure_ascii=False,indent=2),encoding='utf-8')
sock=socket.socket();portOpen=sock.connect_ex(('127.0.0.1',19415))==0;sock.close()
(root/'server-cleanup.json').write_text(json.dumps({'temporaryVerificationPort':19415,'stillOpen':portOpen,'preview19418Touched':False}),encoding='utf-8')
assert not portOpen
selected=[]
for p in root.rglob('*'):
 if not p.is_file():continue
 rel=p.relative_to(root).as_posix()
 if rel.startswith(('f58-source/','f58-build/','__pycache__/','missing-pack/')):continue
 if rel in ('SHA256SUMS.json','raw-evidence.zip','archive-summary.json'):continue
 selected.append(p)
selected.append(root/'f58-build/debug/oh_server.exe')
manifest={p.relative_to(root).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(selected)}
(root/'SHA256SUMS.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2),encoding='utf-8')
with zipfile.ZipFile(root/'raw-evidence.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as z:
 for p in selected+[root/'SHA256SUMS.json']:z.write(p,p.relative_to(root).as_posix())
summary={'files':len(manifest),'zipSHA256':hashlib.sha256((root/'raw-evidence.zip').read_bytes()).hexdigest(),'zipBytes':(root/'raw-evidence.zip').stat().st_size,'excluded':'expanded historical source/dependency/build trees and missing-PNG input copy; exact git archive and historical server exe included'}
(root/'archive-summary.json').write_text(json.dumps(summary,indent=2),encoding='utf-8');print(json.dumps(summary,indent=2))
