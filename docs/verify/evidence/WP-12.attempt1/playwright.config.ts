import { resolve } from 'node:path';
import { defineConfig } from 'E:/openhoi/.orchestrator/wt/WP-12-verify/client/node_modules/@playwright/test/index.mjs';

const port = Number(process.env.OH_TEST_PORT ?? 4173);
if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('OH_TEST_PORT must be a valid port');
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
  outputDir: 'E:/openhoi/.orchestrator/wt/WP-12-verify/target/wp12-verify/test-results',
  testDir: 'E:/openhoi/.orchestrator/wt/WP-12-verify/target/wp12-verify/e2e',
  use: { baseURL, viewport: { width: 1280, height: 720 }, locale: 'en-US' },
  webServer: {
    cwd: 'E:/openhoi/.orchestrator/wt/WP-12-verify/client',
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
