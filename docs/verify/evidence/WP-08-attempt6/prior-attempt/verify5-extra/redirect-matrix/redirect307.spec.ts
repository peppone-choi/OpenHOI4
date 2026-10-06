import { expect, test } from 'E:/openhoi/.orchestrator/wt/WP-08-verify5/client/node_modules/@playwright/test';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { mkdirSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { encode, decode } from 'E:/openhoi/.orchestrator/wt/WP-08-verify5/client/node_modules/@msgpack/msgpack';
import type { ClientMessage, ServerMessage } from '../src/proto/protocol';
import { mapRedirectFixture } from './proxy307';

const evidence = process.env.OH_MAP_REDIRECT_EVIDENCE ?? '../target/wp08/p06-redirect/browser';
mkdirSync(evidence, { recursive: true });

for (const resource of ['metadata', 'index'] as const) {
 for (const reload of [false, true]) {
  test(`WP08 P06 HTTP307 ${resource} CORS redirect sends no foreign request ${reload ? 'after valid selection' : 'on first load'}`, async ({ page, context, request }, info) => {
   const metaResponse = await request.get('/maps/testland/metadata');
   const originalMeta = await metaResponse.json();
   const upstream = new URL(metaResponse.url()).origin;
   const bytes = await (await request.get(`/maps/testland/index.bin?pack=${originalMeta.pack_hash}`)).body();
   const newHash = originalMeta.pack_hash === '0000000000000000' ? '0000000000000001' : '0000000000000000';
   let hash = originalMeta.pack_hash;
   const foreignHits: { method: string; url: string | undefined; origin: string | undefined }[] = [];
   const foreign = createServer((req, res) => {
    foreignHits.push({ method: req.method ?? '', url: req.url, origin: req.headers.origin });
    res.writeHead(200, {
     'content-type': resource === 'metadata' ? 'application/json' : 'application/octet-stream',
     'access-control-allow-origin': '*',
     'access-control-expose-headers': 'x-pack-hash',
     'x-pack-hash': hash,
    });
    res.end(resource === 'metadata' ? JSON.stringify({ ...originalMeta, pack_hash: hash }) : bytes);
   });
   foreign.listen(0, '127.0.0.1'); await once(foreign, 'listening');
   const address = foreign.address(); if (!address || typeof address === 'string') throw new Error('foreign fixture address');
   const foreignURL = `http://127.0.0.1:${address.port}/payload`;
   const requests: string[] = [], errors: string[] = [], redirects: { url: string; status: number; location: string | undefined }[] = [];
   const failures: { url: string; error: string | undefined }[] = [];
   page.on('request', r => requests.push(r.url())); page.on('pageerror', e => errors.push(e.message));
   page.on('response', r => { if (r.status() === 307) redirects.push({ url: r.url(), status: r.status(), location: r.headers().location }); });
   page.on('requestfailed', r => failures.push({ url: r.url(), error: r.failure()?.errorText }));
   let shouldRedirect = !reload, sendIdentity = false, injectedIdentity = false;
   const source = await mapRedirectFixture(upstream, { resource, location: () => foreignURL, redirect: () => shouldRedirect,
    metadata: () => reload && resource === 'index' && injectedIdentity ? { ...originalMeta, pack_hash: hash } : undefined,
   });
   let controlEvidence: unknown;
   type UIState = { date: string | null; hour: string | null; tick: string | null; selected: string | null; worldTick: string | null; state: string | null; country: string | null; ledger: string | null };
   let before: UIState | undefined, preserved: UIState | undefined, cameraBefore: string | null = null, rejectedRequestStartIndex = 0;
   const readUI = async (): Promise<UIState> => ({ date: await page.getByTestId('date').textContent(), hour: await page.getByTestId('hour').textContent(), tick: await page.getByTestId('tick').textContent(), selected: await page.getByTestId('selected-province').textContent(), worldTick: await page.getByTestId('national-panels').getAttribute('data-world-tick'), state: await page.getByTestId('state-panel').locator('h3').textContent(), country: await page.getByTestId('country-panel').locator('h3').textContent(), ledger: await page.getByTestId('state-panel').locator('.ledger-value strong').textContent() });
   try {
    // Positive control proves a native browser can consume this actual CORS 200
    // response through a real 307. The game page is a separate negative control.
    const control = await context.newPage();
    const controlRedirects: { status: number; location: string | undefined }[] = [];
    control.on('response', r => { if (r.status() === 307) controlRedirects.push({ status: r.status(), location: r.headers().location }); });
    await control.goto(`${source.origin}/fonts/OFL.txt`);
    const redirectProbe = await request.get(`${source.origin}/map-fetch-control`, { maxRedirects: 0 });
    expect(redirectProbe.status()).toBe(307); expect(redirectProbe.headers().location).toBe(foreignURL);
    controlEvidence = await control.evaluate(async () => {
     const response = await fetch('/map-fetch-control', { mode: 'cors', redirect: 'follow' });
     return { status: response.status, url: response.url, redirected: response.redirected, cors: response.type, packHash: response.headers.get('x-pack-hash'), body: Array.from(new Uint8Array(await response.arrayBuffer())) };
    });
    const controlResult = controlEvidence as { status: number; url: string; redirected: boolean; cors: string; packHash: string; body: number[] };
    expect(controlResult).toMatchObject({ status: 200, url: foreignURL, redirected: true, cors: 'cors', packHash: hash });
    expect(Buffer.from(controlResult.body)).toEqual(resource === 'metadata' ? Buffer.from(JSON.stringify(originalMeta)) : bytes);
    expect(foreignHits).toHaveLength(1);
    expect(source.sent.filter(r => r.url === '/map-fetch-control')).toHaveLength(2);
    controlEvidence = { ...controlResult, body: undefined, controlRedirects, redirectProbe: { status: redirectProbe.status(), location: redirectProbe.headers().location }, bodyLength: controlResult.body.length, bodySha256: createHash('sha256').update(Buffer.from(controlResult.body)).digest('hex') };
    await control.close(); foreignHits.length = 0; source.sent.length = 0;

    let welcome: Extract<ServerMessage, { type: 'Welcome' }> | undefined;
    if (reload) await page.routeWebSocket('**/ws', route => {
     const server = route.connectToServer();
     server.onMessage(data => { const m = decode(data as Buffer) as ServerMessage; if (m.type === 'Welcome') welcome = m; route.send(data); });
     route.onMessage(data => {
      const m = decode(data as Buffer) as ClientMessage;
      // This test-only identity reload exercises the existing MapView effect.
      // Authority values and the original world remain unmodified.
      if (injectedIdentity && m.type === 'Join') return;
      if (sendIdentity && m.type === 'Query') {
       sendIdentity = false; injectedIdentity = true; hash = newHash;
       route.send(Buffer.from(encode({ ...welcome!, packs: welcome!.packs.map(p => ({ ...p, hash })) })));
      }
      server.send(data);
     });
    });
    await page.goto(`${source.origin}/?forceWebGL=1`);
    const map = page.getByTestId('province-map');
    if (reload) {
     await expect(map).toHaveAttribute('data-frames', /^[1-9]\d*$/);
     await page.getByTestId('pause').click(); await expect(page.getByTestId('pause')).toHaveText('Resume');
     const box = (await map.boundingBox())!, camera = JSON.parse((await map.getAttribute('data-camera'))!);
     await map.click({ position: { x: (1 - camera.x) / camera.width * box.width + box.width / 2, y: (3 - camera.y) / camera.height * box.height + box.height / 2 } });
     await expect(page.getByTestId('selected-province')).toHaveText('30');
     await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('Southern Test State');
     before = await readUI(); cameraBefore = await map.getAttribute('data-camera');
     rejectedRequestStartIndex = requests.length;
     shouldRedirect = true; sendIdentity = true;
     await page.getByTestId('state-panel').getByRole('button', { name: 'Southern Test State', exact: true }).click();
    }
    await expect(page.getByRole('alert')).toHaveText('Map data could not be loaded. Check the bundled scenario files.');
    await expect(map.locator('canvas')).toHaveCount(0);
    expect(foreignHits).toEqual([]);
    expect(requests.every(url => new URL(url).origin === new URL(page.url()).origin)).toBe(true);
    // Engines report redirect:error through different event channels. The
    // actual HTTP 307, one source GET, zero foreign GETs and UI are invariant.
    expect(source.sent).toHaveLength(1); expect(source.sent[0]).toMatchObject({ status: 307, location: foreignURL });
    expect(requests.slice(rejectedRequestStartIndex).filter(url => url === source.origin + source.sent[0].url)).toHaveLength(1);
    await expect(page.getByTestId('connection')).toHaveText('Connected');
    if (before) {
     await expect(page.getByTestId('tick')).toHaveText(before.tick!);
     await expect(page.getByTestId('selected-province')).toHaveText('30');
     await expect(page.getByTestId('state-panel').locator('h3')).toHaveText(before.state!);
     await expect(page.getByTestId('country-panel').locator('h3')).toHaveText(before.country!);
     await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');
     preserved = await readUI(); expect(preserved).toEqual(before);
    } else {
     await expect(page.getByTestId('selected-province')).toHaveText('—');
     await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('Northern Test State');
     await expect(page.getByTestId('country-panel').locator('h3')).toHaveText('Northern Test Nation');
    }
    await page.getByRole('combobox').selectOption('ko');
    await expect(page.getByRole('alert')).toHaveText('지도 데이터를 불러올 수 없습니다. 동봉된 시나리오 파일을 확인하세요.');
    // WebKit reports this exact native blocked-fetch diagnostic as pageerror
    // even when fetch is caught; no other JavaScript error is accepted.
    const blockedFetch = `${source.origin.replace(/^http:\//, '')}${source.sent[0].url} due to access control checks.`;
    expect(errors.filter(e => !(info.project.use.browserName === 'webkit' && e === blockedFetch))).toEqual([]);
   } finally {
    try {
     writeFileSync(`${evidence}/${info.project.name}-${resource}-${reload}.json`, JSON.stringify({ resource, reload, controlEvidence, sourceOrigin: source.origin, foreignURL, foreignHits, requests, rejectedRequestStartIndex, actualSourceRedirects: source.sent, failures, responseEvents: redirects, errors, before, preserved, cameraBefore, rendererOutcome: 'initial load creates no renderer; identity reload disposes previous renderer, keeps authority UI/selection only', alerts: await page.getByRole('alert').allTextContents(), canvasCount: await page.getByTestId('province-map').locator('canvas').count() }, null, 2));
     await page.screenshot({ path: `${evidence}/${info.project.name}-${resource}-${reload}.png` });
    } finally {
     await source.close();
     await new Promise<void>((resolve, reject) => { foreign.close(error => error ? reject(error) : resolve()); foreign.closeAllConnections(); });
    }
   }
  });
 }
}

for (const resource of ['metadata', 'index'] as const) test(`WP08 P06 HTTP307 ${resource} same-origin redirect is also rejected before follow`, async ({ page, request }, info) => {
 const metaResponse = await request.get('/maps/testland/metadata'), meta = await metaResponse.json();
 const path = resource === 'metadata' ? '/maps/testland/metadata' : `/maps/testland/index.bin?pack=${meta.pack_hash}`;
 const alias = `${path}${resource === 'metadata' ? '?' : '&'}alias=1`;
 const source = await mapRedirectFixture(new URL(metaResponse.url()).origin, { resource, location: () => alias, redirect: () => true });
 const direct = await request.get(source.origin + alias); expect(direct.status()).toBe(200);
 const probe = await request.get(source.origin + path, { maxRedirects: 0 }); expect(probe.status()).toBe(307); expect(probe.headers().location).toBe(alias); source.sent.length = 0;
 const requests: string[] = []; page.on('request', r => requests.push(r.url()));
 try {
  await page.goto(`${source.origin}/?forceWebGL=1`);
  await expect(page.getByRole('alert')).toHaveText('Map data could not be loaded. Check the bundled scenario files.');
  await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);
  expect(requests.filter(url => new URL(url).searchParams.has('alias'))).toEqual([]);
  expect(source.sent).toEqual([{ url: path, status: 307, location: alias }]);
 } finally {
  try { writeFileSync(`${evidence}/${info.project.name}-${resource}-local.json`, JSON.stringify({ path, alias, directStatus: direct.status(), actualSourceRedirects: source.sent, requests, alerts: await page.getByRole('alert').allTextContents() }, null, 2)); }
  finally { await source.close(); }
 }
});
