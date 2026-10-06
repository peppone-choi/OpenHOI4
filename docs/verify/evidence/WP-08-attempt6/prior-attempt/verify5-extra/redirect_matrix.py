import run_checks as m,json
from own_runtime import Runtime
extra=m.OUT/'redirect-matrix';extra.mkdir(exist_ok=True);root=m.ROOT.as_posix()
proxy=(m.ROOT/'client/e2e-m1/mapRedirectFixture.ts').read_text(encoding='utf-8');test=(m.ROOT/'client/e2e-m1/map-redirect.spec.ts').read_text(encoding='utf-8')
for status in [301,303,307,308]:
 (extra/f'proxy{status}.ts').write_text(proxy.replace('302',str(status)),encoding='utf-8')
 t=test.replace("'@playwright/test'",repr(root+'/client/node_modules/@playwright/test')).replace("'@msgpack/msgpack'",repr(root+'/client/node_modules/@msgpack/msgpack')).replace("'./mapRedirectFixture'",repr('./proxy'+str(status))).replace('302',str(status)).replace('P06 ',f'P06 HTTP{status} ')
 (extra/f'redirect{status}.spec.ts').write_text(t,encoding='utf-8')
config="import {defineConfig} from "+repr(root+'/client/node_modules/@playwright/test')+";export default defineConfig({testDir:'.',testMatch:'*.spec.ts',use:{baseURL:'http://127.0.0.1:19444',viewport:{width:1280,height:720},locale:'en-US'},projects:[{name:'chrome',use:{browserName:'chromium',channel:'chrome'}},{name:'edge',use:{browserName:'chromium',channel:'msedge'}},{name:'chromium',use:{browserName:'chromium'}},{name:'firefox',use:{browserName:'firefox'}},{name:'webkit',use:{browserName:'webkit'}}]});"
(extra/'config.ts').write_text(config)
with Runtime(m.ROOT,19444,'redirect-matrix'):
 m.run('redirect-matrix',['node',str(m.ROOT/'client/node_modules/@playwright/test/cli.js'),'test','--config',str(extra/'config.ts'),'--workers','1','--reporter','line','--output',str(extra/'test-results')],extra,{'OH_MAP_REDIRECT_EVIDENCE':str(extra/'evidence')})
