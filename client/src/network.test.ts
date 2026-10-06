import { expect, test } from 'vitest';
import { readFileSync } from 'node:fs';
import { decode } from '@msgpack/msgpack';
import { applyServerMessage } from './network';
import type { ServerMessage } from './proto/protocol';

test('REQ-NET-04 Rust MessagePack -> TS decode all message contracts', () => {
  const fixtures = JSON.parse(readFileSync('../target/wp05/wire-fixtures.json', 'utf8')) as { name: string; bytes: number[]; expected: unknown }[];
  expect(fixtures.length).toBe(12);
  for (const fixture of fixtures) expect(decode(Uint8Array.from(fixture.bytes))).toEqual(fixture.expected);
});
test('REQ-GEN-05 REQ-NET-05 disconnected display never advances simulation', () => {
  const message: ServerMessage = { type: 'Snapshot', state: { date: '2000-02-29', hour: 23, tick: '1439', paused: false, speed: 5 } };
  const display = applyServerMessage(null, message);
  expect(display).toEqual(message.state);
  expect(applyServerMessage(display, { type: 'Notice', key: 'unsupported-query' })).toEqual(display);
  const source = readFileSync('src/network.ts', 'utf8') + readFileSync('src/App.tsx', 'utf8');
  expect(source).not.toMatch(/setInterval|setTimeout|Date\.now|new Date|oh_sim/);
});
