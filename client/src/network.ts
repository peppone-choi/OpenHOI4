import { decode, encode } from '@msgpack/msgpack';
import { PROTOCOL_VERSION, type ClientMessage, type PackInfo, type ServerMessage, type TimeState } from './proto/protocol';

type Guard<T> = (value: unknown) => value is T;
type Shape<T> = { [K in keyof T]-?: Guard<T[K]> };
const string: Guard<string> = (value): value is string => typeof value === 'string';
const boolean: Guard<boolean> = (value): value is boolean => typeof value === 'boolean';
// Rust TimeState.hour/speed are u8. This checks their wire representation,
// not calendar/speed game rules; no values are coerced or recalculated here.
const u8: Guard<number> = (value): value is number => typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 0xff;
const literal = <T extends string>(expected: T): Guard<T> => (value): value is T => value === expected;
const nullable = <T,>(guard: Guard<T>): Guard<T | null> => (value): value is T | null => value === null || guard(value);
const array = <T,>(guard: Guard<T>): Guard<T[]> => (value): value is T[] => Array.isArray(value) && value.every(guard);
function shape<T>(fields: Shape<T>): Guard<T> {
  return (value): value is T => typeof value === 'object' && value !== null && !Array.isArray(value)
    && Object.entries(fields).every(([key, guard]) => (guard as Guard<unknown>)((value as Record<string, unknown>)[key]));
}
const timeState = shape<TimeState>({ date: string, hour: u8, tick: string, paused: boolean, speed: u8 });
const packInfo = shape<PackInfo>({ id: string, version: string, hash: string });

// Mapped from the generated Rust union: adding a variant/field or changing a
// field type breaks typecheck until its runtime validator is updated.
const serverFields = {
  Welcome: { type: literal('Welcome'), engine_version: string, protocol_version: string, accepted: boolean, reason_key: nullable(string), packs: array(packInfo), sessions: array(string) },
  CommandResult: { type: literal('CommandResult'), sequence: string, accepted: boolean, reason_key: nullable(string) },
  Snapshot: { type: literal('Snapshot'), state: timeState },
  Delta: { type: literal('Delta'), sequence: string, state: timeState },
  QueryResult: { type: literal('QueryResult'), request: string, supported: boolean, reason_key: nullable(string), state: nullable(timeState) },
  Notice: { type: literal('Notice'), key: string },
} satisfies { [K in ServerMessage['type']]: Shape<Extract<ServerMessage, { type: K }>> };
const serverGuards = Object.values(serverFields).map(fields => shape<Record<string, unknown>>(fields));
export function isServerMessage(value: unknown): value is ServerMessage {
  return serverGuards.some(guard => guard(value));
}
export function applyServerMessage(current: TimeState | null, message: ServerMessage): TimeState | null {
  return message.type === 'Snapshot' || message.type === 'Delta' ? message.state : current;
}
export function connect(onMessage: (message: ServerMessage) => void, onClose: (reasonKey?: 'invalid-server-message') => void) {
  const ws = new WebSocket(`${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/ws`);
  ws.binaryType = 'arraybuffer';
  let closed = false;
  const detach = () => { ws.onopen = null; ws.onmessage = null; ws.onclose = null; ws.onerror = null; };
  const disconnect = (reasonKey?: 'invalid-server-message') => {
    if (closed) return;
    closed = true;
    detach();
    ws.close();
    onClose(reasonKey);
  };
  const send = (message: ClientMessage) => { if (!closed && ws.readyState === WebSocket.OPEN) ws.send(encode(message)); };
  ws.onopen = () => send({ type: 'Hello', protocol_version: PROTOCOL_VERSION });
  ws.onmessage = event => {
    try {
      if (!(event.data instanceof ArrayBuffer)) throw new Error('Expected binary server frame');
      const message: unknown = decode(new Uint8Array(event.data));
      if (!isServerMessage(message)) throw new Error('Invalid server message shape');
      onMessage(message);
      if (message.type === 'Welcome' && message.accepted) send({ type: 'Join', session: 'local', nation: null });
    } catch { disconnect('invalid-server-message'); }
  };
  ws.onclose = () => disconnect();
  ws.onerror = () => disconnect();
  return { send, close: () => {
    closed = true;
    detach();
    ws.close();
  } };
}
