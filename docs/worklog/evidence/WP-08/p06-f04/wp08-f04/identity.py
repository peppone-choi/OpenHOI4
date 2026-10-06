from pathlib import Path
import hashlib,json,subprocess
root=Path.cwd().resolve()
files=['client/src/map/renderer.ts','client/src/map/MapView.tsx','client/src/shell.css','client/playwright.m1.config.ts','client/e2e-m1/map-dpr.spec.ts','client/e2e-m1/map-resize.spec.ts','client/e2e-m1/map-pipeline.spec.ts']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
j={'base_product':'9531121ee5677e43436be69364fdef4bfce2fe86','git_head_at_self_tests':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'working_source_sha256':{p:sha(root/p) for p in files},'server_executable':{'path':'target/debug/oh_server.exe','sha256':sha(root/'target/debug/oh_server.exe')},'built_client':{p.relative_to(root).as_posix():sha(p) for p in (root/'client/dist').rglob('*') if p.is_file()},'self_tests':{'m1_full':{'cases':205,'exit':0,'log':'target/wp08-f04/m1-full.log','scope':'DPR cases before fractional extension; product source unchanged'},'m0_full':{'cases':60,'exit':0,'log':'target/wp08-f04/m0-full.log'},'fractional_dpr_final':{'cases':10,'exit':0,'log':'target/wp08-f04/dpr-final.log','scope':'Final DPR tests with fractional buffer exact floor, native PNG, GL readPixels and cleanup authority'}},'scope':'Own Windows managed servers19440/19439. Not independent P05, Linux result, whole M1 gate or new-source preview. Preview19451 continues exact953.'}
j['self_tests']['m1_full']={'cases':205,'exit':0,'log':'target/wp08-f04/m1-final.log','command':'npm --prefix client run test:e2e -- --config playwright.m1.config.ts','scope':'Final fractional DPR tests and M1 default workers1; latest product source'}
(root/'target/wp08-f04/identity.json').write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(j['server_executable']['sha256'])
