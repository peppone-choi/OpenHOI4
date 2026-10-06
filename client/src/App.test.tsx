import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test } from 'vitest';
import { App } from './App';

test('REQ-PLAT-01 React shell renders without a simulation or server', () => {
  const html = renderToStaticMarkup(<App />);
  expect(html).toContain('<main data-openhoi-shell="">');
  expect(html).toContain('Waiting for server');
  expect(html).toContain('disabled');
});
