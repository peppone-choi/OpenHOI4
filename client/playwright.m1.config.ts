import { resolve } from 'node:path';
import { defineConfig } from '@playwright/test';
import base from './playwright.config';
const port=Number(process.env.OH_TEST_PORT??19412);
if(!Number.isInteger(port)||port<1||port>65535)throw new Error('OH_TEST_PORT must be a valid port');
// Native GPU render fixtures share runner resources. Keep their unchanged
// 30s budgets isolated; see ADR-0803 and the full/isolated Linux phase trace.
export default defineConfig({...base,workers:1,use:{...base.use,baseURL:`http://127.0.0.1:${port}`},testDir:'./e2e-m1',testMatch:['national.spec.ts','malformed-world.spec.ts','map.spec.ts','map-redirect.spec.ts','map-capability.spec.ts','map-resize.spec.ts','map-pipeline.spec.ts','map-dpr.spec.ts'],webServer:{command:`"${resolve('../target/debug',process.platform==='win32'?'oh_server.exe':'oh_server')}" --port ${port} --pack-root ../data/packs`,url:`http://127.0.0.1:${port}`,reuseExistingServer:false}});
