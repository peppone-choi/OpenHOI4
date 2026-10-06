from pathlib import Path
import re,json
r=Path('E:/openhoi/.orchestrator/wt/WP-12-verify');o=r/'target/wp12-verify';d=o/'e2e';d.mkdir(exist_ok=True)
mods={'@playwright/test':(r/'client/node_modules/@playwright/test/index.mjs').as_posix(),'@msgpack/msgpack':(r/'client/node_modules/@msgpack/msgpack/dist.esm/index.mjs').as_posix()}
for p in (r/'client/e2e').glob('*.ts'):
 s=p.read_text(encoding='utf-8')
 for a,b in mods.items():s=s.replace("from '"+a+"'","from '"+b+"'")
 s=s.replace("const evidence = '../docs/worklog/evidence/WP-12';", "const evidence = '../target/wp12-verify/browser';")
 (d/p.name).write_text(s,encoding='utf-8')
s=(r/'client/playwright.config.ts').read_text(encoding='utf-8').replace("from '@playwright/test'","from '"+mods['@playwright/test']+"'").replace("testDir: './e2e'", "testDir: '"+d.as_posix()+"'")
s=s.replace("export default defineConfig({", "export default defineConfig({\n  outputDir: '"+(o/'test-results').as_posix()+"',")
(o/'playwright.config.ts').write_text(s,encoding='utf-8')
print('Copied E2E with output-path and dependency resolution substitutions only')
