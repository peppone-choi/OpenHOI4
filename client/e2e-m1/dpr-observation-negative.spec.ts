import { expect, test, type Page } from '@playwright/test';
import { assertDprEpoch, readDprBuffer, readDprEpoch, readDprPixel } from './dpr-observation';

async function surface(page: Page) {
  // A real DOM/GL surface for observer guards, not a game-renderer proof.
  await page.setContent('<div data-testid="province-map" data-backend="webgl2" style="width:80px;height:60px"><canvas width="80" height="60" style="width:80px;height:60px"></canvas></div>');
  await page.evaluate(() => {
    const host = document.querySelector<HTMLElement>('[data-testid="province-map"]')!;
    host.dataset.camera = JSON.stringify({ x: 1, y: 2, zoom: 3, width: 4, height: 5 });
    Object.assign(window, { __dprLife: () => ({ media: 0, listeners: 6, observers: 1, rafs: 0 }) });
    const gl = host.querySelector('canvas')!.getContext('webgl2', { preserveDrawingBuffer: true })!;
    gl.clearColor(77 / 255, 111 / 255, 155 / 255, 1); gl.clear(gl.COLOR_BUFFER_BIT);
  });
  return (await page.locator('canvas').elementHandle())!;
}

test('DPR observer reads native buffer and rejects a false epoch observation', async ({ page }) => {
  const canvas = await surface(page), epoch = await canvas.evaluate(readDprEpoch);
  const expected = { backend: 'webgl2', dpr: 1, box: { x: 8, y: 8, width: 80, height: 60 }, camera: { x: 1, y: 2, zoom: 3, width: 4, height: 5 } };
  assertDprEpoch(epoch, expected);
  expect(() => assertDprEpoch({ ...epoch, current: false }, expected)).toThrow();
  expect(await page.evaluate(readDprBuffer)).toEqual({ width: 80, height: 60, expectedWidth: 80, expectedHeight: 60 });
  const pixel = await canvas.evaluate(readDprPixel, { x: 40, y: 30 });
  expect(pixel).toEqual({ pixel: [77, 111, 155, 255], error: 0, lost: false });
});

test('DPR observer refuses duplicate hosts and canvases', async ({ page }) => {
  const canvas = await surface(page);
  await page.evaluate(() => document.body.append(document.querySelector('[data-testid="province-map"]')!.cloneNode(true)));
  await expect(page.evaluate(readDprBuffer)).rejects.toThrow('exactly one host');
  await expect(canvas.evaluate(readDprPixel, { x: 40, y: 30 })).rejects.toThrow('before readPixels');
  await page.evaluate(() => document.querySelectorAll('[data-testid="province-map"]')[1].remove());
  await page.evaluate(() => document.querySelector('[data-testid="province-map"]')!.append(document.createElement('canvas')));
  await expect(page.evaluate(readDprBuffer)).rejects.toThrow('exactly one canvas');
  await expect(canvas.evaluate(readDprPixel, { x: 40, y: 30 })).rejects.toThrow('before readPixels');
});

test('DPR observer refuses a detached epoch and missing buffer cannot pass readiness', async ({ page }) => {
  const canvas = await surface(page);
  await canvas.evaluate(c => c.remove());
  await expect(canvas.evaluate(readDprPixel, { x: 40, y: 30 })).rejects.toThrow('before readPixels');
  const b = await page.evaluate(readDprBuffer);
  expect({ width: b.width - b.expectedWidth, height: b.height - b.expectedHeight }).toEqual({ width: -1, height: -1 });
});

test('DPR observer refuses replacement between request and actual RAF', async ({ page }) => {
  const canvas = await surface(page);
  await page.evaluate(() => {
    const raf = window.requestAnimationFrame;
    window.requestAnimationFrame = fn => {
      window.requestAnimationFrame = raf;
      return raf(time => {
        const c = document.querySelector('canvas')!; c.replaceWith(c.cloneNode(true));
        fn(time);
      });
    };
  });
  await expect(canvas.evaluate(readDprPixel, { x: 40, y: 30 })).rejects.toThrow('at readPixels frame');
});
