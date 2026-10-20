import { resolve } from 'node:path';
import { defineConfig } from '@playwright/test';
const port=19429;
export default defineConfig({
 testDir:'./e2e-m1',testMatch:'world-scale-map.spec.ts',workers:1,retries:0,
 timeout:15*60*1000,expect:{timeout:60*1000},
 outputDir:'../target/world-scale/test-results',
 reporter:[['list'],['html',{outputFolder:'../target/world-scale/playwright-report',open:'never'}],['json',{outputFile:'../target/world-scale/playwright.json'}]],
 use:{baseURL:`http://127.0.0.1:${port}`,viewport:{width:1280,height:720},locale:'en-US',deviceScaleFactor:1,screenshot:'only-on-failure',trace:'retain-on-failure'},
 projects:[{name:'chromium',use:{browserName:'chromium'}}],
 webServer:{command:`"${resolve('../target/debug/oh_server')}" --port ${port} --pack-root ../data/packs > ../target/world-scale/server.log 2>&1`,url:`http://127.0.0.1:${port}`,reuseExistingServer:false,gracefulShutdown:{signal:'SIGTERM',timeout:10000}},
});
