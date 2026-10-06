import { afterEach, expect, test, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { decode, encode } from '@msgpack/msgpack';
import { connect, isServerMessage } from './network';
import type { ServerMessage, TimeState } from './proto/protocol';

const state: TimeState = { date: '2000-01-01', hour: 0, tick: '18446744073709551615', paused: false, speed: 1 };
const valid: ServerMessage[] = [
  { type: 'Welcome', engine_version: '0.1.0', protocol_version: 'm0-v1', accepted: true, reason_key: null, packs: [{ id: 'testland', version: '0.1.0', hash: 'ff' }], sessions: ['local'] },
  { type: 'CommandResult', sequence: '18446744073709551615', accepted: false, reason_key: 'invalid-speed' },
  { type: 'Snapshot', state },
  { type: 'Delta', sequence: '18446744073709551615', state },
  { type: 'QueryResult', request: 'time', supported: true, reason_key: null, state },
  { type: 'QueryResult', request: 'ledger', supported: false, reason_key: 'unsupported-query', state: null },
  { type: 'Notice', key: 'unregistered-key' },
];

test('P-06 generated server contract accepts all Rust server wire fixtures without coercion', () => {
  const fixtures = JSON.parse(readFileSync('../target/wp05/wire-fixtures.json', 'utf8')) as { expected: unknown; bytes: number[] }[];
  const serverTypes = new Set(valid.map(message => message.type));
  const serverFixtures = fixtures.filter(fixture => serverTypes.has((fixture.expected as ServerMessage).type));
  expect(serverFixtures).toHaveLength(6);
  for (const fixture of serverFixtures) expect(isServerMessage(decode(Uint8Array.from(fixture.bytes)))).toBe(true);
  for (const message of valid) expect(isServerMessage(message)).toBe(true);
});

const invalid: unknown[] = [null, [], 'Snapshot', 3, {}, { type: 'Unknown' },
  { type: 'Snapshot', state: { ...state, date: { invalid: true } } },
  { type: 'Snapshot', state: null },
  { type: 'Snapshot', state: [] },
  ...Object.keys(state).map(key => ({ type: 'Snapshot', state: { ...state, [key]: { invalid: true } } })),
  ...[NaN, Infinity, -1, 256, 1.5].map(hour => ({ type: 'Snapshot', state: { ...state, hour } })),
  { type: 'Delta', sequence: {}, state },
  { type: 'QueryResult', request: 'time', supported: true, reason_key: null, state: { ...state, tick: 18446744073709551615 } },
  { type: 'QueryResult', request: {}, supported: true, reason_key: null, state },
  { type: 'QueryResult', request: 'time', supported: 'true', reason_key: null, state },
  { type: 'QueryResult', request: 'time', supported: false, reason_key: {}, state: null },
  { type: 'Notice', key: { invalid: true } },
  { type: 'CommandResult', sequence: '1', accepted: true, reason_key: [] },
  { type: 'Welcome', engine_version: '0.1', protocol_version: 'm0-v1', accepted: true, reason_key: null, packs: [{}], sessions: ['local'] },
  { type: 'Welcome', engine_version: '0.1', protocol_version: 'm0-v1', accepted: true, reason_key: null, packs: [], sessions: [{}] },
];
for (const message of valid) {
  for (const key of Object.keys(message)) {
    const missing = { ...message } as Record<string, unknown>;
    delete missing[key]; invalid.push(missing);
  }
}
for (const [index, message] of invalid.entries()) {
  test(`P-06 rejects malformed server contract case ${index}`, () => expect(isServerMessage(message)).toBe(false));
}

class Socket {
  static OPEN = 1;
  static latest: Socket;
  readyState = 1;
  binaryType = '';
  onopen: (() => void) | null = null;
  onmessage: ((event: { data: unknown }) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  sent: unknown[] = [];
  closeCount = 0;
  constructor() { Socket.latest = this; }
  send(data: unknown) { this.sent.push(data); }
  close() { this.closeCount++; this.readyState = 3; this.onclose?.(); }
  receive(message: unknown) { this.onmessage?.({ data: encode(message).slice().buffer }); }
}
afterEach(() => vi.unstubAllGlobals());

for (const message of invalid) {
  test(`P-06 connection rejects decoded shape before React: ${JSON.stringify(message)}`, () => {
    vi.stubGlobal('WebSocket', Socket); vi.stubGlobal('location', { protocol: 'http:', host: 'localhost' });
    const receive = vi.fn(); const close = vi.fn();
    const connection = connect(receive, close);
    const socket = Socket.latest;
    socket.receive(message);
    expect(receive).not.toHaveBeenCalled();
    expect(close).toHaveBeenCalledExactlyOnceWith('invalid-server-message');
    expect(socket.closeCount).toBe(1);
    connection.send({ type: 'Query', request: 'time', kind: 'time' });
    socket.receive(valid[0]);
    expect(receive).not.toHaveBeenCalled();
    expect(socket.sent).toEqual([]);
  });
}
test('P-06 accepted messages preserve full u64 strings and shutdown does not duplicate callbacks', () => {
  vi.stubGlobal('WebSocket', Socket); vi.stubGlobal('location', { protocol: 'http:', host: 'localhost' });
  const receive = vi.fn(); const close = vi.fn(); connect(receive, close);
  const socket = Socket.latest;
  socket.receive(valid[2]);
  expect(receive).toHaveBeenCalledWith(valid[2]);
  expect(close).not.toHaveBeenCalled();
  socket.onmessage?.({ data: new Uint8Array([193]).buffer });
  expect(close).toHaveBeenCalledExactlyOnceWith('invalid-server-message');
  socket.onerror?.(); socket.onclose?.();
  expect(close).toHaveBeenCalledTimes(1);
});
