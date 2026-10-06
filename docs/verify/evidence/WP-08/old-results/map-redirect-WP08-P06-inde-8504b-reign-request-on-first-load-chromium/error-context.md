# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: map-redirect.spec.ts >> WP08 P06 index CORS redirect sends no foreign request on first load
- Location: e2e-m1\map-redirect.spec.ts:15:3

# Error details

```
Error: expect(locator).toHaveText(expected) failed

Locator: getByRole('alert')
Expected: "Map data could not be loaded. Check the bundled scenario files."
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveText" getByRole('alert') with timeout 5000ms
  - waiting for getByRole('alert')

```

```yaml
- main:
  - heading "OpenHOI4" [level=1]
  - paragraph: Local scenario
  - text: Language
  - combobox "Language":
    - option "English" [selected]
    - option "한국어"
  - status: Connected
  - region "Time controls":
    - term: Date
    - definition: 2000-01-01
    - term: Hour
    - definition: "8"
    - term: Tick
    - definition: "8"
    - term: Speed
    - definition: "1"
    - button "Pause"
    - button "Speed 1" [pressed]: "1"
    - button "Speed 2": "2"
    - button "Speed 3": "3"
    - button "Speed 4": "4"
    - button "Speed 5": "5"
  - region "Province map":
    - navigation "Map modes":
      - button "Ownership" [pressed]
      - button "Control"
      - button "Terrain"
      - button "States"
      - button "Reset view"
    - text: "Province:"
    - strong: —
    - text: Click to select · Drag to pan · Scroll to zoom
    - button "Clear selection" [disabled]
  - complementary:
    - region "Country":
      - heading "Country" [level=2]
      - navigation:
        - button "Northern Test Nation" [pressed]
        - button "Southern Test Nation"
      - heading "Northern Test Nation" [level=3]
      - term: Tag
      - definition: NTH
      - term: Capital province
      - definition: "10"
      - term: Government
      - definition: Test Republic
      - heading "Ideology support (ratio)" [level=4]
      - term: Civic
      - definition: "0.75"
      - term: Local
      - definition: "0.25"
    - region "State":
      - heading "State" [level=2]
      - navigation:
        - button "Northern Test State" [pressed]
        - button "Southern Test State"
      - heading "Northern Test State" [level=3]
      - term: Owner
      - definition: Northern Test Nation
      - term: Population
      - definition: "1000"
      - term: Steel
      - definition: "2"
      - term: Industry
      - definition: "1"
      - group:
        - text: Infrastructure
        - strong: "1"
      - table:
        - rowgroup:
          - row "Province Owner Controller":
            - columnheader "Province"
            - columnheader "Owner"
            - columnheader "Controller"
        - rowgroup:
          - row "10 Northern Test Nation Northern Test Nation":
            - cell "10"
            - cell "Northern Test Nation"
            - cell "Northern Test Nation"
          - row "20 Northern Test Nation Southern Test Nation":
            - cell "20"
            - cell "Northern Test Nation"
            - cell "Southern Test Nation"
  - link "Noto Sans KR · SIL Open Font License 1.1":
    - /url: /fonts/OFL.txt
```

# Test source

```ts
  1   | import { expect, test } from '@playwright/test';
  2   | import { createServer } from 'node:http';
  3   | import { once } from 'node:events';
  4   | import { mkdirSync, writeFileSync } from 'node:fs';
  5   | import { createHash } from 'node:crypto';
  6   | import { encode, decode } from '@msgpack/msgpack';
  7   | import type { ClientMessage, ServerMessage } from '../src/proto/protocol';
  8   | import { mapRedirectFixture } from './mapRedirectFixture';
  9   |
  10  | const evidence = process.env.OH_MAP_REDIRECT_EVIDENCE ?? '../target/wp08/p06-redirect/browser';
  11  | mkdirSync(evidence, { recursive: true });
  12  |
  13  | for (const resource of ['metadata', 'index'] as const) {
  14  |  for (const reload of [false, true]) {
  15  |   test(`WP08 P06 ${resource} CORS redirect sends no foreign request ${reload ? 'after valid selection' : 'on first load'}`, async ({ page, context, request }, info) => {
  16  |    const metaResponse = await request.get('/maps/testland/metadata');
  17  |    const originalMeta = await metaResponse.json();
  18  |    const upstream = new URL(metaResponse.url()).origin;
  19  |    const bytes = await (await request.get(`/maps/testland/index.bin?pack=${originalMeta.pack_hash}`)).body();
  20  |    const newHash = originalMeta.pack_hash === '0000000000000000' ? '0000000000000001' : '0000000000000000';
  21  |    let hash = originalMeta.pack_hash;
  22  |    const foreignHits: { method: string; url: string | undefined; origin: string | undefined }[] = [];
  23  |    const foreign = createServer((req, res) => {
  24  |     foreignHits.push({ method: req.method ?? '', url: req.url, origin: req.headers.origin });
  25  |     res.writeHead(200, {
  26  |      'content-type': resource === 'metadata' ? 'application/json' : 'application/octet-stream',
  27  |      'access-control-allow-origin': '*',
  28  |      'access-control-expose-headers': 'x-pack-hash',
  29  |      'x-pack-hash': hash,
  30  |     });
  31  |     res.end(resource === 'metadata' ? JSON.stringify({ ...originalMeta, pack_hash: hash }) : bytes);
  32  |    });
  33  |    foreign.listen(0, '127.0.0.1'); await once(foreign, 'listening');
  34  |    const address = foreign.address(); if (!address || typeof address === 'string') throw new Error('foreign fixture address');
  35  |    const foreignURL = `http://127.0.0.1:${address.port}/payload`;
  36  |    const requests: string[] = [], errors: string[] = [], redirects: { url: string; status: number; location: string | undefined }[] = [];
  37  |    const failures: { url: string; error: string | undefined }[] = [];
  38  |    page.on('request', r => requests.push(r.url())); page.on('pageerror', e => errors.push(e.message));
  39  |    page.on('response', r => { if (r.status() === 302) redirects.push({ url: r.url(), status: r.status(), location: r.headers().location }); });
  40  |    page.on('requestfailed', r => failures.push({ url: r.url(), error: r.failure()?.errorText }));
  41  |    let shouldRedirect = !reload, sendIdentity = false, injectedIdentity = false;
  42  |    const source = await mapRedirectFixture(upstream, { resource, location: () => foreignURL, redirect: () => shouldRedirect,
  43  |     metadata: () => reload && resource === 'index' && injectedIdentity ? { ...originalMeta, pack_hash: hash } : undefined,
  44  |    });
  45  |    let controlEvidence: unknown;
  46  |    type UIState = { date: string | null; hour: string | null; tick: string | null; selected: string | null; worldTick: string | null; state: string | null; country: string | null; ledger: string | null };
  47  |    let before: UIState | undefined, preserved: UIState | undefined, cameraBefore: string | null = null, rejectedRequestStartIndex = 0;
  48  |    const readUI = async (): Promise<UIState> => ({ date: await page.getByTestId('date').textContent(), hour: await page.getByTestId('hour').textContent(), tick: await page.getByTestId('tick').textContent(), selected: await page.getByTestId('selected-province').textContent(), worldTick: await page.getByTestId('national-panels').getAttribute('data-world-tick'), state: await page.getByTestId('state-panel').locator('h3').textContent(), country: await page.getByTestId('country-panel').locator('h3').textContent(), ledger: await page.getByTestId('state-panel').locator('.ledger-value strong').textContent() });
  49  |    try {
  50  |     // Positive control proves a native browser can consume this actual CORS 200
  51  |     // response through a real 302. The game page is a separate negative control.
  52  |     const control = await context.newPage();
  53  |     const controlRedirects: { status: number; location: string | undefined }[] = [];
  54  |     control.on('response', r => { if (r.status() === 302) controlRedirects.push({ status: r.status(), location: r.headers().location }); });
  55  |     await control.goto(`${source.origin}/fonts/OFL.txt`);
  56  |     const redirectProbe = await request.get(`${source.origin}/map-fetch-control`, { maxRedirects: 0 });
  57  |     expect(redirectProbe.status()).toBe(302); expect(redirectProbe.headers().location).toBe(foreignURL);
  58  |     controlEvidence = await control.evaluate(async () => {
  59  |      const response = await fetch('/map-fetch-control', { mode: 'cors', redirect: 'follow' });
  60  |      return { status: response.status, url: response.url, redirected: response.redirected, cors: response.type, packHash: response.headers.get('x-pack-hash'), body: Array.from(new Uint8Array(await response.arrayBuffer())) };
  61  |     });
  62  |     const controlResult = controlEvidence as { status: number; url: string; redirected: boolean; cors: string; packHash: string; body: number[] };
  63  |     expect(controlResult).toMatchObject({ status: 200, url: foreignURL, redirected: true, cors: 'cors', packHash: hash });
  64  |     expect(Buffer.from(controlResult.body)).toEqual(resource === 'metadata' ? Buffer.from(JSON.stringify(originalMeta)) : bytes);
  65  |     expect(foreignHits).toHaveLength(1);
  66  |     expect(source.sent.filter(r => r.url === '/map-fetch-control')).toHaveLength(2);
  67  |     controlEvidence = { ...controlResult, body: undefined, controlRedirects, redirectProbe: { status: redirectProbe.status(), location: redirectProbe.headers().location }, bodyLength: controlResult.body.length, bodySha256: createHash('sha256').update(Buffer.from(controlResult.body)).digest('hex') };
  68  |     await control.close(); foreignHits.length = 0; source.sent.length = 0;
  69  |
  70  |     let welcome: Extract<ServerMessage, { type: 'Welcome' }> | undefined;
  71  |     if (reload) await page.routeWebSocket('**/ws', route => {
  72  |      const server = route.connectToServer();
  73  |      server.onMessage(data => { const m = decode(data as Buffer) as ServerMessage; if (m.type === 'Welcome') welcome = m; route.send(data); });
  74  |      route.onMessage(data => {
  75  |       const m = decode(data as Buffer) as ClientMessage;
  76  |       // This test-only identity reload exercises the existing MapView effect.
  77  |       // Authority values and the original world remain unmodified.
  78  |       if (injectedIdentity && m.type === 'Join') return;
  79  |       if (sendIdentity && m.type === 'Query') {
  80  |        sendIdentity = false; injectedIdentity = true; hash = newHash;
  81  |        route.send(Buffer.from(encode({ ...welcome!, packs: welcome!.packs.map(p => ({ ...p, hash })) })));
  82  |       }
  83  |       server.send(data);
  84  |      });
  85  |     });
  86  |     await page.goto(`${source.origin}/?forceWebGL=1`);
  87  |     const map = page.getByTestId('province-map');
  88  |     if (reload) {
  89  |      await expect(map).toHaveAttribute('data-frames', /^[1-9]\d*$/);
  90  |      await page.getByTestId('pause').click(); await expect(page.getByTestId('pause')).toHaveText('Resume');
  91  |      const box = (await map.boundingBox())!, camera = JSON.parse((await map.getAttribute('data-camera'))!);
  92  |      await map.click({ position: { x: (1 - camera.x) / camera.width * box.width + box.width / 2, y: (3 - camera.y) / camera.height * box.height + box.height / 2 } });
  93  |      await expect(page.getByTestId('selected-province')).toHaveText('30');
  94  |      await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('Southern Test State');
  95  |      before = await readUI(); cameraBefore = await map.getAttribute('data-camera');
  96  |      rejectedRequestStartIndex = requests.length;
  97  |      shouldRedirect = true; sendIdentity = true;
  98  |      await page.getByTestId('state-panel').getByRole('button', { name: 'Southern Test State', exact: true }).click();
  99  |     }
> 100 |     await expect(page.getByRole('alert')).toHaveText('Map data could not be loaded. Check the bundled scenario files.');
      |                                           ^ Error: expect(locator).toHaveText(expected) failed
  101 |     await expect(map.locator('canvas')).toHaveCount(0);
  102 |     expect(foreignHits).toEqual([]);
  103 |     expect(requests.every(url => new URL(url).origin === new URL(page.url()).origin)).toBe(true);
  104 |     // Engines report redirect:error through different event channels. The
  105 |     // actual HTTP 302, one source GET, zero foreign GETs and UI are invariant.
  106 |     expect(source.sent).toHaveLength(1); expect(source.sent[0]).toMatchObject({ status: 302, location: foreignURL });
  107 |     expect(requests.slice(rejectedRequestStartIndex).filter(url => url === source.origin + source.sent[0].url)).toHaveLength(1);
  108 |     await expect(page.getByTestId('connection')).toHaveText('Connected');
  109 |     if (before) {
  110 |      await expect(page.getByTestId('tick')).toHaveText(before.tick!);
  111 |      await expect(page.getByTestId('selected-province')).toHaveText('30');
  112 |      await expect(page.getByTestId('state-panel').locator('h3')).toHaveText(before.state!);
  113 |      await expect(page.getByTestId('country-panel').locator('h3')).toHaveText(before.country!);
  114 |      await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');
  115 |      preserved = await readUI(); expect(preserved).toEqual(before);
  116 |     } else {
  117 |      await expect(page.getByTestId('selected-province')).toHaveText('—');
  118 |      await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('Northern Test State');
  119 |      await expect(page.getByTestId('country-panel').locator('h3')).toHaveText('Northern Test Nation');
  120 |     }
  121 |     await page.getByRole('combobox').selectOption('ko');
  122 |     await expect(page.getByRole('alert')).toHaveText('지도 데이터를 불러올 수 없습니다. 동봉된 시나리오 파일을 확인하세요.');
  123 |     // WebKit reports this exact native blocked-fetch diagnostic as pageerror
  124 |     // even when fetch is caught; no other JavaScript error is accepted.
  125 |     const blockedFetch = `${source.origin.replace(/^http:\//, '')}${source.sent[0].url} due to access control checks.`;
  126 |     expect(errors.filter(e => !(info.project.use.browserName === 'webkit' && e === blockedFetch))).toEqual([]);
  127 |    } finally {
  128 |     try {
  129 |      writeFileSync(`${evidence}/${info.project.name}-${resource}-${reload}.json`, JSON.stringify({ resource, reload, controlEvidence, sourceOrigin: source.origin, foreignURL, foreignHits, requests, rejectedRequestStartIndex, actualSourceRedirects: source.sent, failures, responseEvents: redirects, errors, before, preserved, cameraBefore, rendererOutcome: 'initial load creates no renderer; identity reload disposes previous renderer, keeps authority UI/selection only', alerts: await page.getByRole('alert').allTextContents(), canvasCount: await page.getByTestId('province-map').locator('canvas').count() }, null, 2));
  130 |      await page.screenshot({ path: `${evidence}/${info.project.name}-${resource}-${reload}.png` });
  131 |     } finally {
  132 |      await source.close();
  133 |      await new Promise<void>((resolve, reject) => { foreign.close(error => error ? reject(error) : resolve()); foreign.closeAllConnections(); });
  134 |     }
  135 |    }
  136 |   });
  137 |  }
  138 | }
  139 |
  140 | for (const resource of ['metadata', 'index'] as const) test(`WP08 P06 ${resource} same-origin redirect is also rejected before follow`, async ({ page, request }, info) => {
  141 |  const metaResponse = await request.get('/maps/testland/metadata'), meta = await metaResponse.json();
  142 |  const path = resource === 'metadata' ? '/maps/testland/metadata' : `/maps/testland/index.bin?pack=${meta.pack_hash}`;
  143 |  const alias = `${path}${resource === 'metadata' ? '?' : '&'}alias=1`;
  144 |  const source = await mapRedirectFixture(new URL(metaResponse.url()).origin, { resource, location: () => alias, redirect: () => true });
  145 |  const direct = await request.get(source.origin + alias); expect(direct.status()).toBe(200);
  146 |  const probe = await request.get(source.origin + path, { maxRedirects: 0 }); expect(probe.status()).toBe(302); expect(probe.headers().location).toBe(alias); source.sent.length = 0;
  147 |  const requests: string[] = []; page.on('request', r => requests.push(r.url()));
  148 |  try {
  149 |   await page.goto(`${source.origin}/?forceWebGL=1`);
  150 |   await expect(page.getByRole('alert')).toHaveText('Map data could not be loaded. Check the bundled scenario files.');
  151 |   await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);
  152 |   expect(requests.filter(url => new URL(url).searchParams.has('alias'))).toEqual([]);
  153 |   expect(source.sent).toEqual([{ url: path, status: 302, location: alias }]);
  154 |  } finally {
  155 |   try { writeFileSync(`${evidence}/${info.project.name}-${resource}-local.json`, JSON.stringify({ path, alias, directStatus: direct.status(), actualSourceRedirects: source.sent, requests, alerts: await page.getByRole('alert').allTextContents() }, null, 2)); }
  156 |   finally { await source.close(); }
  157 |  }
  158 | });
  159 |
```
