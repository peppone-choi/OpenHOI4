import { expect, test } from '@playwright/test';
import { encode, decode } from '@msgpack/msgpack';
import { writeFileSync } from 'node:fs';
import type { ClientMessage, ServerMessage } from '../src/proto/protocol';

test('REQ-GEN-01 REQ-TIME-01 REQ-NET-01 authoritative pause, five speeds and date', async ({ page, browser }, info) => {
  writeFileSync(`../target/wp05/${info.project.name}-browser.json`, JSON.stringify({ version: browser.version(), project: info.project.name, platform: process.platform, viewport: { width: 1280, height: 720 }, locale: 'en-US', render: 'headless M0 DOM, no game map/GPU measurement' }, null, 2));
  await page.goto('/');
  await expect(page.getByTestId('connection')).toHaveText('Connected');
  await expect(page.getByTestId('date')).toContainText('2000-');
  await page.getByTestId('pause').click();
  await expect(page.getByTestId('pause')).toHaveText('Resume');
  const tick = await page.getByTestId('tick').textContent();
  const date = await page.getByTestId('date').textContent();
  await page.waitForTimeout(700);
  await expect(page.getByTestId('tick')).toHaveText(tick!);
  for (const speed of ['1', '2', '3', '4', '5']) {
    await page.getByTestId(`speed-${speed}`).click();
    await expect(page.getByTestId('speed')).toHaveText(speed);
  }
  await page.getByTestId('pause').click();
  await expect(page.getByTestId('pause')).toHaveText('Pause');
  await expect(page.getByTestId('tick')).not.toHaveText(tick!);
  await expect(page.getByTestId('date')).not.toHaveText(date!);
  await page.screenshot({ path: `../target/wp05/${info.project.name}-running.png`, fullPage: true });
  await page.getByTestId('pause').click();
  await expect(page.getByTestId('pause')).toHaveText('Resume');
  await page.screenshot({ path: `../target/wp05/${info.project.name}-paused.png`, fullPage: true });
});

test('REQ-NET-02 executable serves built assets and ko/en localization', async ({ request }) => {
  const index = await request.get('/');
  expect(index.status()).toBe(200);
  const html = await index.text();
  const asset = html.match(/src="([^"]+\.js)"/)!;
  expect(asset).toBeTruthy();
  const js = await request.get(asset[1]);
  expect(js.headers()['content-type']).toContain('javascript');
  expect((await js.body()).length).toBeGreaterThan(1000);
  for (const locale of ['ko', 'en']) expect((await request.get(`/locales/${locale}.json`)).status()).toBe(200);
  expect((await request.get('/missing.js')).status()).toBe(404);
});

test('REQ-NET-01 version mismatch, acceptance, rejection and unsupported scope', async ({ page }) => {
  await page.goto('/');
  const requests: ClientMessage[] = [
    { type: 'Join', session: 'local', nation: null },
    { type: 'Command', sequence: '1', command: { type: 'Pause', paused: true } },
    { type: 'Command', sequence: '2', command: { type: 'SetSpeed', speed: 9 } },
    { type: 'Command', sequence: '2', command: { type: 'Pause', paused: false } },
    { type: 'Query', request: 'q', kind: 'ledger' },
  ];
  async function exchange(version: string, messages: ClientMessage[]) {
    const bytes = [ { type: 'Hello', protocol_version: version }, ...messages ].map(m => Array.from(encode(m)));
    const frames = await page.evaluate(async bytes => new Promise<number[][]>(resolve => {
      const frames: number[][] = [];
      const ws = new WebSocket(`ws://${location.host}/ws`); ws.binaryType = 'arraybuffer';
      ws.onopen = () => { for (const frame of bytes) ws.send(Uint8Array.from(frame)); };
      ws.onmessage = event => frames.push(Array.from(new Uint8Array(event.data)));
      ws.onclose = () => resolve(frames);
      setTimeout(() => { ws.close(); resolve(frames); }, 1500);
    }), bytes);
    return frames.map(frame => decode(Uint8Array.from(frame))) as ServerMessage[];
  }
  const result = { bad: await exchange('unsupported', []), good: await exchange('m0-v1', requests) };
  expect(result.bad[0]).toMatchObject({ accepted: false, reason_key: 'protocol-version' });
  expect(result.good).toContainEqual(expect.objectContaining({ type: 'CommandResult', sequence: '1', accepted: true }));
  expect(result.good).toContainEqual(expect.objectContaining({ type: 'CommandResult', sequence: '2', accepted: false, reason_key: 'invalid-speed' }));
  expect(result.good).toContainEqual(expect.objectContaining({ reason_key: 'invalid-sequence' }));
  expect(result.good.find(m => m.type === 'QueryResult')).toMatchObject({ type: 'QueryResult', supported: false, reason_key: 'unsupported-query' });
});

test('REQ-NET-01 Create, supported query, Delta and explicit refusal', async ({ page }) => {
  await page.goto('/');
  const messages: ClientMessage[] = [
    { type: 'Hello', protocol_version: 'm0-v1' },
    { type: 'Command', sequence: '1', command: { type: 'Pause', paused: true } },
    { type: 'Create', scenario: 'unsupported', seed: '1', mode: 'single' },
    { type: 'Create', scenario: 'testland', seed: '18446744073709551615', mode: 'single' },
    { type: 'Query', request: 'time', kind: 'time' },
  ];
  const values: ServerMessage[] = [];
  let closed = false;
  await page.exposeFunction('recordCreateFrame', (frame: number[]) => {
    values.push(decode(Uint8Array.from(frame)) as ServerMessage);
  });
  await page.exposeFunction('recordCreateClose', () => { closed = true; });
  await page.evaluate(bytes => {
    const probe = window as unknown as {
      createProbe?: WebSocket;
      recordCreateFrame: (frame: number[]) => Promise<void>;
      recordCreateClose: () => Promise<void>;
    };
    const ws = new WebSocket(`ws://${location.host}/ws`); ws.binaryType = 'arraybuffer';
    probe.createProbe = ws;
    ws.onopen = () => { for (const data of bytes) ws.send(Uint8Array.from(data)); ws.send('invalid text'); ws.send(new Uint8Array([193])); };
    ws.onmessage = event => { void probe.recordCreateFrame(Array.from(new Uint8Array(event.data))); };
    ws.onclose = () => { void probe.recordCreateClose(); };
  }, messages.map(m => Array.from(encode(m))));
  try {
    // Connection setup and Create processing are readiness, not the Delta
    // observation window. Keep the original 750ms budget and every assertion.
    await expect.poll(() => values.some(value => value.type === 'Snapshot')).toBe(true);
    await page.waitForTimeout(750);
  } finally {
    await page.evaluate(() => {
      const probe = window as unknown as {createProbe?: WebSocket};
      probe.createProbe?.close();
      delete probe.createProbe;
    });
    await expect.poll(() => closed).toBe(true);
  }
  expect(values).toContainEqual(expect.objectContaining({ type: 'CommandResult', accepted: false, reason_key: 'not-joined' }));
  expect(values).toContainEqual({ type: 'Notice', key: 'unsupported-create' });
  expect(values).toContainEqual({ type: 'Notice', key: 'invalid-message' });
  expect(values).toContainEqual(expect.objectContaining({ type: 'Snapshot' }));
  expect(values).toContainEqual(expect.objectContaining({ type: 'QueryResult', request: 'time', supported: true, reason_key: null }));
  const deltas = values.filter(v => v.type === 'Delta');
  expect(deltas.length).toBeGreaterThan(0);
  expect(deltas.length).toBeLessThanOrEqual(8);
  expect(deltas.map(d => d.sequence)).toEqual(deltas.map((_, index) => String(index + 1)));
});

test('REQ-NET-02 ko localization', async ({ page }, info) => {
  await page.goto('/');
  await expect(page.getByTestId('connection')).toHaveText('Connected');
  await page.getByRole('combobox').selectOption('ko');
  await expect(page.getByTestId('connection')).toHaveText('연결됨');
  await expect(page.getByTestId('pause')).toHaveText('일시정지');
  await page.getByTestId('pause').click();
  await expect(page.getByTestId('pause')).toHaveText('재개');
  await page.screenshot({ path: `../target/wp05/${info.project.name}-ko.png`, fullPage: true });
});

test('REQ-GEN-05 REQ-NET-05 server disconnect freezes displayed time and input', async ({ page }) => {
  let disconnect: (() => Promise<void>) | undefined;
  await page.routeWebSocket('**/ws', route => {
    route.connectToServer(); // Actual Rust server, forwarded unchanged until closure.
    disconnect = () => route.close();
  });
  await page.goto('/');
  await expect(page.getByTestId('connection')).toHaveText('Connected');
  await page.getByTestId('speed-5').click();
  await expect(page.getByTestId('speed')).toHaveText('5');
  await disconnect!();
  await expect(page.getByTestId('connection')).toHaveText('Disconnected');
  const tick = await page.getByTestId('tick').textContent();
  await page.waitForTimeout(700);
  await expect(page.getByTestId('tick')).toHaveText(tick!);
  await expect(page.getByTestId('pause')).toBeDisabled();
  await expect(page.getByTestId('speed-1')).toBeDisabled();
});
