import { afterEach, describe, expect, test, vi } from 'vitest';
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


describe('S1 MovementResult transport guard', () => {
  const zero = { value: '0', bits: '0', fractional_bits: 32 };
  const oneAndHalf = { value: '1.5', bits: '6442450944', fractional_bits: 32 };
  const unit = { division: '2', nation: 1, province: 10, route: [11, 12], elapsed: oneAndHalf, current_leg_hours: oneAndHalf, remaining_hours: oneAndHalf };
  const movement = { state_hash: '0123456789abcdef', tick: '18446744073709551615', units: [unit, { ...unit, division: '10' }] };
  const message = { type: 'MovementResult', request: 'movement', supported: true, reason_key: null, movement };
  const withUnit = (change: Record<string, unknown>) => ({ ...message, movement: { ...movement, units: [{ ...unit, ...change }] } });
  const withFixed = (change: Record<string, unknown>) => withUnit({ elapsed: { ...oneAndHalf, ...change } });
  const without = (value: object, key: string) => {
    const copy: Record<string, unknown> = { ...value };
    delete copy[key];
    return copy;
  };

  const accepted = [
    ['empty movement context', { ...message, movement: { ...movement, units: [] } }],
    ['stopped unit', withUnit({ route: [], elapsed: zero, current_leg_hours: zero, remaining_hours: zero })],
    ['moving units in numeric division order', message],
    ['not joined', { ...message, supported: false, reason_key: 'not-joined', movement: null }],
    ['u32/u16 endpoints', { ...message, movement: { ...movement, tick: '0', units: [{ ...unit, division: '0', nation: 0, province: 0, route: [65535, 0] }, { ...unit, division: '4294967295', nation: 65535, province: 65535 }] } }],
    ['route preserves server order and repeats', withUnit({ route: [12, 11, 12] })],
    ['transport does not recalculate time semantics', withUnit({ elapsed: oneAndHalf, current_leg_hours: zero, remaining_hours: zero })],
    ['fixed maximum raw value', withFixed({ value: '2147483647.99999999976716935634613037109375', bits: '9223372036854775807' })],
    ['fixed 128 character limit inclusive', withFixed({ value: '0.' + '0'.repeat(126), bits: '0' })],
    ['half-even tie to even zero', withFixed({ value: '0.000000000116415321826934814453125', bits: '0' })],
    ['half-even tie to even two', withFixed({ value: '0.000000000349245965480804443359375', bits: '2' })],
    ['below half', withFixed({ value: '0.0000000001', bits: '0' })],
    ['above half', withFixed({ value: '0.0000000002', bits: '1' })],
  ] as const;
  for (const [name, value] of accepted) {
    test(`accepts ${name}`, () => expect(isServerMessage(value)).toBe(true));
  }

  const rejected: [string, unknown][] = [
    ...Object.keys(message).map(key => [`missing envelope ${key}`, without(message, key)] as [string, unknown]),
    ...Object.keys(movement).map(key => [`missing view ${key}`, { ...message, movement: without(movement, key) }] as [string, unknown]),
    ...Object.keys(unit).map(key => [`missing unit ${key}`, { ...message, movement: { ...movement, units: [without(unit, key)] } }] as [string, unknown]),
    ['extra envelope field', { ...message, extra: 1 }],
    ['extra view field', { ...message, movement: { ...movement, extra: 1 } }],
    ['extra unit field', withUnit({ extra: 1 })],
    ['supported without movement', { ...message, movement: null }],
    ['unsupported with movement', { ...message, supported: false, reason_key: 'not-joined' }],
    ['supported with reason', { ...message, reason_key: 'not-joined' }],
    ['unsupported without reason', { ...message, supported: false, movement: null }],
    ...['01', '-1', '4294967296', 1, '', '1.0', '1e1'].map(division => [`invalid division ${division}`, withUnit({ division })] as [string, unknown]),
    ['reverse numeric units', { ...message, movement: { ...movement, units: [...movement.units].reverse() } }],
    ['duplicate units', { ...message, movement: { ...movement, units: [unit, unit] } }],
    ['sparse units', { ...message, movement: { ...movement, units: new Array(1) } }],
    ['non-array units', { ...message, movement: { ...movement, units: {} } }],
    ...[65536, -1, 1.5, '11'].map(province => [`invalid route item ${province}`, withUnit({ route: [province] })] as [string, unknown]),
    ['sparse route', withUnit({ route: new Array(1) })],
    ['non-array route', withUnit({ route: {} })],
    ['invalid nation', withUnit({ nation: 65536 })],
    ['invalid province', withUnit({ province: -1 })],
    ['fractional bits 16', withFixed({ fractional_bits: 16 })],
    ['value and raw bits disagree', withFixed({ bits: '1' })],
    ['negative fixed value', withFixed({ value: '-1.5' })],
    ['leading zero raw bits', withFixed({ bits: '06442450944' })],
    ['raw bits overflow', withFixed({ value: '2147483648', bits: '9223372036854775808' })],
    ['numeric raw bits', withFixed({ bits: 6442450944 })],
    ['leading zero value', withFixed({ value: '01.5' })],
    ['exponent value', withFixed({ value: '15e-1' })],
    ['value over 128 characters', withFixed({ value: '0.' + '0'.repeat(127), bits: '0' })],
    ['half-even zero tie rounded up', withFixed({ value: '0.000000000116415321826934814453125', bits: '1' })],
    ['half-even two tie rounded down', withFixed({ value: '0.000000000349245965480804443359375', bits: '1' })],
    ['current leg uses fixed guard', withUnit({ current_leg_hours: { ...zero, fractional_bits: 16 } })],
    ['remaining uses fixed guard', withUnit({ remaining_hours: { ...zero, bits: '1' } })],
    ...Object.keys(oneAndHalf).map(key => [`missing fixed ${key}`, withUnit({ elapsed: without(oneAndHalf, key) })] as [string, unknown]),
    ['extra fixed field', withFixed({ extra: 1 })],
    ...['0123456789abcde', '0123456789ABCDEF'].map(state_hash => [`invalid hash ${state_hash}`, { ...message, movement: { ...movement, state_hash } }] as [string, unknown]),
    ...['01', '18446744073709551616', '-1', 1].map(tick => [`invalid tick ${tick}`, { ...message, movement: { ...movement, tick } }] as [string, unknown]),
  ];
  for (const [name, value] of rejected) {
    test(`rejects ${name}`, () => expect(isServerMessage(value)).toBe(false));
  }

  test('invalid decoded MovementResult closes once without receive', () => {
    vi.stubGlobal('WebSocket', Socket); vi.stubGlobal('location', { protocol: 'http:', host: 'localhost' });
    const receive = vi.fn(); const close = vi.fn(); connect(receive, close);
    const socket = Socket.latest;
    socket.receive({ ...message, extra: 1 });
    socket.receive(message); socket.onerror?.(); socket.onclose?.();
    expect(receive).not.toHaveBeenCalled();
    expect(close).toHaveBeenCalledExactlyOnceWith('invalid-server-message');
    expect(socket.closeCount).toBe(1);
  });

  test('valid decoded MovementResult is delivered once without coercion', () => {
    vi.stubGlobal('WebSocket', Socket); vi.stubGlobal('location', { protocol: 'http:', host: 'localhost' });
    const receive = vi.fn(); const close = vi.fn(); connect(receive, close);
    const socket = Socket.latest;
    socket.receive(message);
    expect(receive).toHaveBeenCalledExactlyOnceWith(message);
    expect(close).not.toHaveBeenCalled();
    expect(socket.closeCount).toBe(0);
  });
});
