import { expect } from '@playwright/test';

// These functions run in the page through Playwright serialization. Keep each
// browser reader self-contained: it must not close over Node helpers/imports.
export function readDprBuffer() {
  const hosts = document.querySelectorAll<HTMLElement>('[data-testid="province-map"]');
  if (hosts.length !== 1) throw new Error('DPR buffer requires exactly one host');
  const canvases = hosts[0].querySelectorAll<HTMLCanvasElement>('canvas');
  if (canvases.length > 1) throw new Error('DPR buffer requires exactly one canvas');
  const canvas = canvases[0];
  // A replacement may briefly have no surface. This sentinel cannot satisfy
  // the existing zero-difference poll; it does not fabricate readiness.
  return canvas ? {
    width: canvas.width, height: canvas.height,
    expectedWidth: Math.floor(canvas.clientWidth * devicePixelRatio),
    expectedHeight: Math.floor(canvas.clientHeight * devicePixelRatio),
  } : { width: -1, height: -1, expectedWidth: 0, expectedHeight: 0 };
}

export function readDprEpoch(canvas: HTMLCanvasElement) {
  const host = canvas.parentElement;
  if (!host) throw new Error('DPR epoch has no host');
  const hosts = document.querySelectorAll<HTMLElement>('[data-testid="province-map"]');
  const canvases = host.querySelectorAll('canvas'), rect = host.getBoundingClientRect();
  return {
    hostCount: hosts.length, canvasCount: canvases.length,
    hostCurrent: hosts.length === 1 && hosts[0] === host,
    connected: canvas.isConnected, current: host.querySelector('canvas') === canvas,
    backend: host.dataset.backend, dpr: devicePixelRatio,
    box: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
    camera: JSON.parse(host.dataset.camera!) as { x: number; y: number; zoom: number; width: number; height: number },
    viewport: { width: innerWidth, height: innerHeight, scrollX, scrollY },
    buffer: {
      width: canvas.width, height: canvas.height,
      expectedWidth: Math.floor(canvas.clientWidth * devicePixelRatio),
      expectedHeight: Math.floor(canvas.clientHeight * devicePixelRatio),
    },
    resources: (window as unknown as { __dprLife: () => unknown }).__dprLife(),
  };
}

export type DprEpoch = ReturnType<typeof readDprEpoch>;
export function assertDprEpoch(epoch: DprEpoch, expected: {
  backend: string | null; dpr: number; box: DprEpoch['box']; camera: DprEpoch['camera'];
}) {
  expect(epoch.hostCount).toBe(1); expect(epoch.canvasCount).toBe(1);
  expect(epoch.hostCurrent).toBe(true); expect(epoch.connected).toBe(true); expect(epoch.current).toBe(true);
  expect(epoch.backend).toBe(expected.backend); expect(epoch.dpr).toBe(expected.dpr);
  expect(epoch.box).toEqual(expected.box); expect(epoch.camera).toEqual(expected.camera);
}

export async function readDprPixel(canvas: HTMLCanvasElement, { x, y }: { x: number; y: number }) {
  const host = canvas.parentElement;
  if (!host || !canvas.isConnected || document.querySelectorAll('[data-testid="province-map"]').length !== 1
    || document.querySelector('[data-testid="province-map"]') !== host
    || host.querySelectorAll('canvas').length !== 1 || host.querySelector('canvas') !== canvas) {
    throw new Error('DPR epoch changed before readPixels');
  }
  if (host.dataset.backend !== 'webgl2') return null;
  const gl = canvas.getContext('webgl2')!;
  return new Promise<{ pixel: number[]; error: number; lost: boolean }>((resolve, reject) => requestAnimationFrame(() => {
    if (!canvas.isConnected || document.querySelectorAll('[data-testid="province-map"]').length !== 1
      || document.querySelector('[data-testid="province-map"]') !== host
      || host.querySelectorAll('canvas').length !== 1 || host.querySelector('canvas') !== canvas) {
      reject(new Error('DPR epoch changed at readPixels frame')); return;
    }
    const pixel = new Uint8Array(4);
    gl.readPixels(Math.floor(x * canvas.width / host.clientWidth), canvas.height - 1 - Math.floor(y * canvas.height / host.clientHeight),
      1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
    resolve({ pixel: Array.from(pixel), error: gl.getError(), lost: gl.isContextLost() });
  }));
}
