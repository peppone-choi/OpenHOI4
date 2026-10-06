import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test } from 'vitest';
import { App } from './App';

test('REQ-PLAT-01 React shell renders without a simulation or server', () => {
  expect(renderToStaticMarkup(<App />)).toBe('<main data-openhoi-shell=""></main>');
});
