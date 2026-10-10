import { expect, test } from '@playwright/test';
import { encode, decode } from '@msgpack/msgpack';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import type { ClientMessage, ServerMessage } from '../src/proto/protocol';
const evidence = process.env.OH_E2E_EVIDENCE ?? '../docs/worklog/evidence/WP-12';
mkdirSync(evidence, { recursive: true });

test('REQ-LOC-01 REQ-LOC-02 actual Rust executable ko/en switch and local CJK font', async ({ page, request, browser }, info) => {
  const requests: string[] = [];
  page.on('request', request => requests.push(request.url()));
  await page.goto('/');
  await expect(page.getByTestId('connection')).toHaveText('Connected');
  await page.getByTestId('pause').click();
  await expect(page.getByTestId('pause')).toHaveText('Resume');
  const tick = await page.getByTestId('tick').textContent();
  await page.getByRole('combobox').selectOption('ko');
  await expect(page.locator('html')).toHaveAttribute('lang', 'ko');
  await expect(page).toHaveTitle('OpenHOI4');
  await expect(page.getByTestId('connection')).toHaveText('연결됨');
  await expect(page.getByTestId('pause')).toHaveText('재개');
  await expect(page.getByRole('button', { name: '속도 3', exact: true })).toBeVisible();
  const font = await page.evaluate(async () => {
    const faces = await document.fonts.load('400 16px "Noto Sans KR"', '한국어 한글 漢字');
    await document.fonts.ready;
    return { loaded: faces.map(face => ({ family: face.family, status: face.status })), check: document.fonts.check('400 16px "Noto Sans KR"', '한국어 한글 漢字'), applied: getComputedStyle(document.querySelector('select')!).fontFamily, weightAxis: getComputedStyle(document.body).fontVariationSettings };
  });
  expect(font.loaded.some(face => face.family.includes('Noto Sans KR') && face.status === 'loaded')).toBe(true);
  expect(font.check).toBe(true);
  expect(font.applied).toContain('Noto Sans KR');
  expect(font.weightAxis).toContain('"wght" 400');
  await page.screenshot({ path: `${evidence}/${info.project.name}-ko.png`, fullPage: true });
  await page.getByRole('combobox').selectOption('en');
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await expect(page.getByRole('button', { name: 'Speed 3', exact: true })).toBeVisible();
  await expect(page.getByTestId('connection')).toHaveText('Connected');
  await expect(page.getByTestId('tick')).toHaveText(tick!);
  await page.screenshot({ path: `${evidence}/${info.project.name}-en.png`, fullPage: true });
  const response = await request.get('/fonts/NotoSansKR.ttf');
  expect(response.status()).toBe(200);
  expect(response.headers()['content-type']).toBe('font/ttf');
  const bytes = await response.body();
  expect(bytes.equals(readFileSync('public/fonts/NotoSansKR.ttf'))).toBe(true);
  expect(bytes.subarray(0, 4)).toEqual(Buffer.from([0, 1, 0, 0]));
  const license = await request.get('/fonts/OFL.txt');
  expect(await license.text()).toContain('SIL OPEN FONT LICENSE Version 1.1');
  for (const lang of ['ko', 'en']) {
    const response = await request.get(`/locales/${lang}.ftl`);
    expect(response.status()).toBe(200);
    expect(response.headers()['content-type']).toContain('text/plain');
    expect(await response.text()).toContain('ledger-accumulated =');
  }
  expect(requests.some(url => url.includes('/fonts/NotoSansKR.ttf'))).toBe(true);
  expect(requests.every(url => new URL(url).origin === new URL(page.url()).origin)).toBe(true);
  writeFileSync(`${evidence}/${info.project.name}-font.json`, JSON.stringify({ project: info.project.name, browser: browser.version(), viewport: { width: 1280, height: 720 }, server: 'Rust embedded executable, no Vite', locale: 'ko then en; paused authoritative time', font, requests, mime: response.headers()['content-type'], bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') }, null, 2));
});

test('REQ-LOC-01 correlated actual server CommandResult failures translate on language switch', async ({ page }) => {
  let injected = false;
  let unissuedAcknowledged = false;
  page.on('websocket', socket => socket.on('framereceived', event => {
    if (!(event.payload instanceof Buffer)) return;
    const message = decode(event.payload) as ServerMessage;
    if (message.type === 'CommandResult' && message.sequence === '99' && message.accepted) unissuedAcknowledged = true;
  }));
  await page.routeWebSocket('**/ws', route => {
    const server = route.connectToServer();
    route.onMessage(data => {
      const message = decode(data as Buffer) as ClientMessage;
      if (!injected && message.type === 'Command' && message.command.type === 'SetSpeed') {
        injected = true;
        // Keep the UI-issued sequence; only inject the invalid payload sent to Rust.
        server.send(Buffer.from(encode({ ...message, command: { type: 'SetSpeed', speed: 9 } } satisfies ClientMessage)));
        // A native unissued ACK must not clear that correlated failure. This also
        // raises the server watermark so the next actual UI command is rejected.
        server.send(Buffer.from(encode({ type: 'Command', sequence: '99', command: { type: 'Pause', paused: true } } satisfies ClientMessage)));
        return;
      }
      server.send(data);
    });
  });
  await page.goto('/');
  await expect(page.getByTestId('connection')).toHaveText('Connected');
  await page.getByTestId('speed-3').click();
  await expect.poll(() => unissuedAcknowledged).toBe(true);
  await expect(page.getByRole('alert')).toHaveText('Speed must be from 1 to 5');
  await page.getByRole('combobox').selectOption('ko');
  await expect(page.getByRole('alert')).toHaveText('속도는 1~5 단계입니다');
  await page.getByTestId('pause').click();
  await expect(page.getByRole('alert')).toHaveText('명령 순번이 잘못되었거나 중복되었습니다');
});

test('REQ-LOC-01 actual server Notice renders through Fluent', async ({ page }) => {
  await page.routeWebSocket('**/ws', route => {
    const server = route.connectToServer();
    route.onMessage(data => {
      server.send(data);
      if ((decode(data as Buffer) as ClientMessage).type === 'Join') server.send('invalid text');
    });
  });
  await page.goto('/');
  await expect(page.getByRole('alert')).toHaveText('Invalid protocol message');
  await page.getByRole('combobox').selectOption('ko');
  await expect(page.getByRole('alert')).toHaveText('잘못된 프로토콜 메시지입니다');
});

test('AC-M1-03 ledger display fixture opens with keyboard and preserves supplied values', async ({ page }, info) => {
  // Exact LedgerValue React markup rendered by Vitest, not a game query or wire fixture.
  // Real executable supplies this page's theme and local font. WP-09 wires live ledger queries.
  await page.goto('/');
  await expect(page.getByTestId('connection')).toHaveText('Connected');
  for (const language of ['ko', 'en']) {
    const markup = readFileSync(`../target/wp12/ledger-${language}.html`, 'utf8');
    await page.evaluate(({ markup, language }) => {
      document.getElementById('root')!.hidden = true;
      const fixture = document.getElementById('ledger-fixture') ?? document.createElement('main');
      fixture.id = 'ledger-fixture';
      fixture.innerHTML = markup;
      document.body.append(fixture);
      document.documentElement.lang = language;
    }, { markup, language });
    const summary = page.locator('summary');
    await summary.focus();
    await summary.press('Enter');
    await expect(page.locator('details')).toHaveAttribute('open', '');
    await expect(page.locator('.ledger-tooltip')).toBeVisible();
    await expect(page.locator('.ledger-tooltip')).toContainText('18446744073709551615');
    await expect(page.locator('.ledger-tooltip')).toContainText('9007199254740993.000001');
    await expect(page.locator('.ledger-tooltip')).toContainText('MUL-EXACT');
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({ path: `${evidence}/${info.project.name}-ledger-fixture-${language}.png`, fullPage: true });
    await summary.press('Enter');
    await expect(page.locator('details')).not.toHaveAttribute('open');
  }
});
