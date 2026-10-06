import { expect, test } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { mkdirSync, writeFileSync } from 'node:fs';
import { createTranslator, Localization } from './i18n';
import { LedgerValue, type LedgerView } from './components/LedgerValue';

const ledger: LedgerView = {
  base: '9007199254740993.000001', final: 'SERVER-FINAL-17.000007', tick: '18446744073709551615',
  entries: [
    { id: 'base', labelKey: 'ledger-base', operationKey: 'ledger-base', value: '9007199254740993.000001', accumulated: 'BASE-EXACT', sourceKey: null },
    { id: 'add', labelKey: 'speed', operationKey: 'ledger-add', value: '-0.000001', accumulated: 'ADD-EXACT', sourceKey: 'date' },
    { id: 'mul', labelKey: 'hour', operationKey: 'ledger-multiply', value: '1.000007', accumulated: 'MUL-EXACT', sourceKey: 'tick' },
  ],
};
test('REQ-LOC-01 Fluent translates interpolation, M0 notices, failures and localized fallback', () => {
  const ko = createTranslator('ko'); const en = createTranslator('en');
  expect(ko('speed-choice', { speed: 3 })).toBe('속도 3');
  expect(en('speed-choice', { speed: 3 })).toBe('Speed 3');
  expect(ko('invalid-message')).toBe('잘못된 프로토콜 메시지입니다');
  expect(en('protocol-version')).toBe('Protocol version mismatch');
  expect(ko('unregistered-server-key')).toBe('서버 메시지를 표시할 수 없습니다');
  expect(en('speed-choice')).toBe('The server sent an unavailable message');
  const fallback = createTranslator('ko', { ko: 'unknown-message = 알 수 없음', en: 'unknown-message = Unknown\nonly-english = English fallback' });
  expect(fallback('only-english')).toBe('English fallback');
});
test('AC-M1-03 ledger display preserves server precision, order, sources and accumulated values in ko/en', () => {
  for (const language of ['ko', 'en'] as const) {
    const t = createTranslator(language);
    const html = renderToStaticMarkup(<Localization.Provider value={{ language, t }}><LedgerValue labelKey="speed" ledger={ledger} /></Localization.Provider>);
    expect(html).toContain('9007199254740993.000001');
    expect(html).toContain('SERVER-FINAL-17.000007');
    expect(html).toContain('18446744073709551615');
    expect(html.indexOf('ADD-EXACT')).toBeLessThan(html.indexOf('MUL-EXACT'));
    expect(html).toContain(t('ledger-source', { source: t('tick') }));
    expect(html).toContain(t('ledger-accumulated'));
    expect(html).toContain('<details');
    expect(html).toContain('aria-controls=');
    mkdirSync('../target/wp12', { recursive: true });
    writeFileSync(`../target/wp12/ledger-${language}.html`, html);
  }
});
test('AC-M1-03 empty ledger and untrusted server values render safely without calculation', () => {
  const html = renderToStaticMarkup(<LedgerValue labelKey="speed" ledger={{ base: '5', final: '<script>alert(1)</script>', entries: [] }} />);
  expect(html).toContain('No contributions');
  expect(html).toContain('&lt;script&gt;');
  expect(html).not.toContain('<script>');
});
