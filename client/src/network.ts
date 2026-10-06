import { decode, encode } from '@msgpack/msgpack';
import { PROTOCOL_VERSION, type ClientMessage, type PackInfo, type ServerMessage, type TimeState, type WorldView, type NationView, type SupportView, type StateView, type ScalarView, type ProvinceView, type LedgerView, type LedgerRow } from './proto/protocol';
import { MAP_DISPLAY_VERSION,type MapMetadata,type MapStyle } from './proto/protocol';

type Guard<T> = (value: unknown) => value is T;
type Shape<T> = { [K in keyof T]-?: Guard<T[K]> };
const string: Guard<string> = (value): value is string => typeof value === 'string';
const boolean: Guard<boolean> = (value): value is boolean => typeof value === 'boolean';
// Rust TimeState.hour/speed are u8. This checks their wire representation,
// not calendar/speed game rules; no values are coerced or recalculated here.
const u8: Guard<number> = (value): value is number => typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 0xff;
const u16: Guard<number> = (v): v is number => typeof v === 'number' && Number.isInteger(v) && v >= 0 && v <= 65535;
const u32: Guard<number> = (v): v is number => typeof v === 'number' && Number.isInteger(v) && v >= 0 && v <= 4294967295;
const color: Guard<[number, number, number]> = (v): v is [number, number, number] => Array.isArray(v) && v.length === 3 && v.every(u8);
const literal = <T extends string>(expected: T): Guard<T> => (value): value is T => value === expected;
const nullable = <T,>(guard: Guard<T>): Guard<T | null> => (value): value is T | null => value === null || guard(value);
const array = <T,>(guard: Guard<T>): Guard<T[]> => (value): value is T[] => Array.isArray(value) && value.every(guard);
function shape<T>(fields: Shape<T>): Guard<T> {
  return (value): value is T => typeof value === 'object' && value !== null && !Array.isArray(value)
    && Object.entries(fields).every(([key, guard]) => (guard as Guard<unknown>)((value as Record<string, unknown>)[key]));
}
const timeState = shape<TimeState>({ date: string, hour: u8, tick: string, paused: boolean, speed: u8 });
const packInfo = shape<PackInfo>({ id: string, version: string, hash: string });
const positiveU32:Guard<number>=(v):v is number=>u32(v)&&v>0;
const decimal:Guard<string>=(v):v is string=>string(v)&&/^(0|[1-9]\d*)$/.test(v);
const hash:Guard<string>=(v):v is string=>string(v)&&/^[0-9a-f]{16}$/.test(v);
const mapStyle=shape<MapStyle>({background:color,nation_border:color,state_border:color,province_border:color,selected:color,hovered:color,nation_width_milli:positiveU32,state_width_milli:positiveU32,province_width_milli:positiveU32,highlight_milli:positiveU32,fit_milli:positiveU32,zoom_min_milli:positiveU32,zoom_max_milli:positiveU32,wheel_milli:positiveU32,drag_threshold:positiveU32});
const mapMetadata=shape<MapMetadata>({schema_version:u16,map_id:string,width:positiveU32,height:positiveU32,province_ids:array(u16),pack_hash:hash,index_hash:hash,byte_length:decimal,style:mapStyle});
export function isMapMetadata(v:unknown):v is MapMetadata{return mapMetadata(v)&&v.schema_version===MAP_DISPLAY_VERSION&&v.style.fit_milli<=1000&&v.style.highlight_milli<=1000&&v.style.zoom_min_milli<=v.style.zoom_max_milli;}

const ledgerRow = shape<LedgerRow>({ id: string, label_key: string, operation_key: string, value: string, accumulated: string, source_key: nullable(string) });
const ledger = shape<LedgerView>({ base: string, final_value: string, tick: string, entries: array(ledgerRow) });
const support = shape<SupportView>({ name_key: string, value: string });
const nation = shape<NationView>({ id: u16, tag: string, name_key: string, color, capital: u16, government_key: string, support: array(support) });
const scalar = shape<ScalarView>({ name_key: string, value: string });
const stateView = shape<StateView>({ id: u16, name_key: string, provinces: array(u16), owner: u16, population: string, resources: array(scalar), buildings: array(scalar), infrastructure: ledger });
const province = shape<ProvinceView>({ id: u16, state: nullable(u16), owner: nullable(u16), controller: nullable(u16), owner_color: nullable(color), controller_color: nullable(color), terrain_key: string, terrain_color: color, state_color: nullable(color) });
const world = shape<WorldView>({ tick: string, map_id: string, width: u32, height: u32, province_ids: array(u16), neutral_color: color, mode_keys: array(string), nations: array(nation), states: array(stateView), provinces: array(province) });
// Mapped from the generated Rust union: adding a variant/field or changing a
// field type breaks typecheck until its runtime validator is updated.
const serverFields = {
  Welcome: { type: literal('Welcome'), engine_version: string, protocol_version: string, accepted: boolean, reason_key: nullable(string), packs: array(packInfo), sessions: array(string) },
  CommandResult: { type: literal('CommandResult'), sequence: string, accepted: boolean, reason_key: nullable(string) },
  Snapshot: { type: literal('Snapshot'), state: timeState },
  Delta: { type: literal('Delta'), sequence: string, state: timeState },
  QueryResult: { type: literal('QueryResult'), request: string, supported: boolean, reason_key: nullable(string), state: nullable(timeState) },
  WorldResult: { type: literal('WorldResult'), request: string, supported: boolean, reason_key: nullable(string), world: nullable(world) },
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
