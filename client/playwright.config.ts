import { resolve } from 'node:path';
import { defineConfig } from '@playwright/test';

const port = Number(process.env.OH_TEST_PORT ?? 4173);
if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('OH_TEST_PORT must be a valid port');
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
  testDir: './e2e',
  use: { baseURL, viewport: { width: 1280, height: 720 }, locale: 'en-US' },
  webServer: {
    command: `"${resolve('../target/debug', process.platform === 'win32' ? 'oh_server.exe' : 'oh_server')}" --port ${port} --pack-root ../data/packs/examples/m0`,
    url: baseURL,
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
