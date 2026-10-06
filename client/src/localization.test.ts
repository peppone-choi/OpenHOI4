import { readFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { expect, test } from 'vitest';

const root = resolve(import.meta.dirname, '../..');
test('REQ-LOC-01 ko/en Fluent catalogs cover M0 keys and interpolated shell strings', () => {
  for (const language of ['ko', 'en']) {
    const file = resolve(root, `client/public/locales/${language}.ftl`);
    expect(existsSync(file)).toBe(true);
    const ftl = readFileSync(file, 'utf8');
    const original = JSON.parse(readFileSync(resolve(root, `client/public/locales/${language}.json`), 'utf8'));
    for (const key of Object.keys(original)) expect(ftl).toMatch(new RegExp(`^${key} =`, 'm'));
    expect(ftl).toContain('speed-choice =');
    expect(ftl).toContain('$speed');
    expect(ftl).toContain('ledger-source =');
  }
});
test('REQ-LOC-02 ships an OFL CJK font and local font-face without CDN', () => {
  const css = readFileSync(resolve(root, 'client/src/shell.css'), 'utf8');
  expect(css).toContain('@font-face');
  expect(css).toContain('/fonts/NotoSansKR.ttf');
  expect(css).not.toMatch(/https?:\/\//);
  expect(existsSync(resolve(root, 'client/public/fonts/NotoSansKR.ttf'))).toBe(true);
  expect(readFileSync(resolve(root, 'client/public/fonts/OFL.txt'), 'utf8')).toContain('SIL OPEN FONT LICENSE Version 1.1');
});

test('REQ-LOC-02 font cmap contains actual ko catalog glyphs, all Hangul syllables and CJK examples', () => {
  const font = readFileSync(resolve(root, 'client/public/fonts/NotoSansKR.ttf'));
  const tables = Array.from({ length: font.readUInt16BE(4) }, (_, index) => 12 + index * 16);
  const cmapTable = tables.find(offset => font.toString('ascii', offset, offset + 4) === 'cmap')!;
  const cmap = font.readUInt32BE(cmapTable + 8);
  const records = Array.from({ length: font.readUInt16BE(cmap + 2) }, (_, index) => cmap + 4 + index * 8);
  const subtable = records.map(offset => cmap + font.readUInt32BE(offset + 4)).find(offset => font.readUInt16BE(offset) === 12)!;
  expect(subtable).toBeDefined();
  const groups = Array.from({ length: font.readUInt32BE(subtable + 12) }, (_, index) => {
    const offset = subtable + 16 + index * 12;
    return { start: font.readUInt32BE(offset), end: font.readUInt32BE(offset + 4), glyph: font.readUInt32BE(offset + 8) };
  });
  const hasGlyph = (code: number) => groups.some(group => code >= group.start && code <= group.end && group.glyph + code - group.start > 0);
  const korean = readFileSync(resolve(root, 'client/public/locales/ko.ftl'), 'utf8').split('\n').filter(line => line.includes(' = ')).map(line => line.split(' = ')[1]).join('');
  for (const character of new Set(korean + '한국어 한글 漢字')) {
    if (!/\s/.test(character)) expect(hasGlyph(character.codePointAt(0)!)).toBe(true);
  }
  for (let code = 0xAC00; code <= 0xD7A3; code++) expect(hasGlyph(code)).toBe(true);
});
