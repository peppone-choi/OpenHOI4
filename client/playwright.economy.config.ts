import { resolve } from 'node:path';
import { cpSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { defineConfig } from '@playwright/test';

const port=Number(process.env.OH_TEST_PORT??19415);
if(!Number.isInteger(port)||port<1||port>65534)throw new Error('OH_TEST_PORT and its next port must be valid');
// Existing server contract: self-contained M2 is staged as <host>/testland.
// No dependency on the separate production-content PR or a protocol extension.
const host=mkdtempSync(resolve(tmpdir(),'openhoi-o3-economy-host-'));
cpSync(resolve('../data/packs/testland_m2'),resolve(host,'testland'),{recursive:true});
const legacy=mkdtempSync(resolve(tmpdir(),'openhoi-o3-economy-legacy-'));
cpSync(resolve('../data/packs/testland'),resolve(legacy,'testland'),{recursive:true});
const server=process.env.OH_SERVER_EXECUTABLE??resolve('../target/debug',process.platform==='win32'?'oh_server.exe':'oh_server');
const chromium={browserName:'chromium' as const,launchOptions:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:undefined};
export default defineConfig({
  testDir:'./e2e-m1',testMatch:'economy.spec.ts',workers:1,
  use:{baseURL:`http://127.0.0.1:${port}`,viewport:{width:1280,height:900},locale:'en-US'},
  webServer:[
    {command:`"${server}" --port ${port} --pack-root "${host}" --scenario m2_initial`,url:`http://127.0.0.1:${port}`,reuseExistingServer:false},
    {command:`"${server}" --port ${port+1} --pack-root "${legacy}" --scenario m1`,url:`http://127.0.0.1:${port+1}`,reuseExistingServer:false},
  ],
  projects:[
    {name:'chromium',grepInvert:/actual frozen M1/,use:chromium},
    {name:'chromium-m1',grep:/actual frozen M1/,use:{...chromium,baseURL:`http://127.0.0.1:${port+1}`}},
  ],
});
