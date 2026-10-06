import { expect, test } from '@playwright/test';
import { encode, decode } from '@msgpack/msgpack';
import { mkdirSync, writeFileSync } from 'node:fs';
const evidence = process.env.OH_MALFORMED_EVIDENCE ?? '../docs/worklog/evidence/WP-12/p06';
mkdirSync(evidence, { recursive: true });

test('P-06 malformed MessagePack Snapshot preserves shell and closes connection', async ({ page }, info) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.routeWebSocket('**/ws', route => {
    const server = route.connectToServer();
    server.onMessage(data => {
      const message = decode(data as Buffer) as { type: string; state?: object };
      if (message.type === 'Snapshot') route.send(Buffer.from(encode({ ...message, state: { ...message.state, date: { invalid: true } } })));
      else if (message.type !== 'Delta') route.send(data);
    });
  });
  await page.goto('/');
  await page.waitForTimeout(1000);
  const observed = { errors, shell: await page.locator('main[data-openhoi-shell]').count(), text: await page.locator('body').innerText() };
  writeFileSync(`${evidence}/${info.project.name}-malformed-snapshot.json`, JSON.stringify(observed, null, 2));
  await page.screenshot({ path: `${evidence}/${info.project.name}-malformed-snapshot.png` });
  expect(errors, 'Invalid server structure must not crash React').toEqual([]);
  await expect(page.getByTestId('connection')).toHaveText('Disconnected');
  await expect(page.getByTestId('pause')).toBeDisabled();
  for (const speed of [1, 2, 3, 4, 5]) await expect(page.getByTestId(`speed-${speed}`)).toBeDisabled();
  await expect(page.getByRole('alert')).toHaveText('Invalid server message; connection closed');
  await page.getByRole('combobox').selectOption('ko');
  await expect(page.getByRole('alert')).toHaveText('잘못된 서버 메시지로 연결을 종료했습니다');
  await page.screenshot({ path: `${evidence}/${info.project.name}-malformed-snapshot-ko.png` });
});
