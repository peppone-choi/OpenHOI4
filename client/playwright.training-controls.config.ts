import {defineConfig} from '@playwright/test';
export default defineConfig({
  testDir:'./e2e-m1',testMatch:'training-controls.spec.ts',workers:1,
  outputDir:'../target/wp16/training-controls-browser',
  use:{viewport:{width:1280,height:900},locale:'en-US'},
  projects:[{name:'chromium',use:{browserName:'chromium',launchOptions:{...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:{}),args:['--renderer-process-limit=1','--num-raster-threads=1']}}}],
});
