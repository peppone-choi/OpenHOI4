import { resolve } from 'node:path';
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  use: { baseURL: 'http://127.0.0.1:4173', viewport: { width: 1280, height: 720 }, locale: 'en-US' },
  webServer: {
    command: `"${resolve('../target/debug', process.platform === 'win32' ? 'oh_server.exe' : 'oh_server')}" --port 4173 --pack-root ../data/packs/examples/m0`,
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: false,
  },
  projects: [
    ...(process.env.OH_BROWSER_PRODUCTS === '1' ? [
      { name: 'chrome', use: { browserName: 'chromium' as const, channel: 'chrome' as const } },
      { name: 'edge', use: { browserName: 'chromium' as const, channel: 'msedge' as const } },
    ] : []),
    { name: 'chromium', use: { browserName: 'chromium' } },
    { name: 'firefox', use: { browserName: 'firefox' } },
    { name: 'webkit', use: { browserName: 'webkit' } },
  ],
});
