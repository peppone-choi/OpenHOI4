import {test,expect,vi,afterEach} from 'vitest';
import {readFileSync} from 'node:fs';
import {decode,encode} from '@msgpack/msgpack';
import {isServerMessage,connect} from './network';
const fixtures=JSON.parse(readFileSync(new URL('../../target/wp14/economy-wire-fixtures.json',import.meta.url),'utf8'));
const expected=fixtures[0].expected;
test('actual Rust economic query roundtrips and all raw fixed units agree',()=>{
  expect(decode(Uint8Array.from(fixtures[0].bytes))).toEqual(expected);expect(isServerMessage(expected)).toBe(true);
  expect(expected.economy.nations[0].ledger.total_ic.bits).toBe('655360');
  expect(expected.economy.nations[0].political_capital.bits).toBe('196608');
  expect(expected.economy.nations[0].projects[0].progress.bits).toBe('163840');
});
test('malformed or inconsistent economic numeric fields close the typed boundary',()=>{
  const mutate=(fn:(v:any)=>void)=>{const value=structuredClone(expected);fn(value);expect(isServerMessage(value)).toBe(false);};
  for(const v of ['NaN','Infinity','1e6',3,null,'-1','9223372036854775808'])mutate(r=>r.economy.nations[0].political_capital.bits=v);
  mutate(r=>r.economy.nations[0].political_capital.fractional_bits=32);mutate(r=>r.economy.nations[0].political_capital.value='4');
  mutate(r=>delete r.economy.nations[0].ledger.consumer_residual);mutate(r=>r.economy.nations[0].ledger.capacity=125);
  mutate(r=>r.economy.nations.reverse());mutate(r=>r.economy.nations.push(r.economy.nations[0]));mutate(r=>r.economy.nations[0].projects.push(r.economy.nations[0].projects[0]));
  mutate(r=>r.economy.nations[0].projects[0].dormancy='unknown');mutate(r=>r.economy.nations[0].ledger.allocation.pop());mutate(r=>r.economy.nations[0].intruder=true);
  mutate(r=>r.supported=false);mutate(r=>r.economy=null);mutate(r=>r.reason_key='error');
});
afterEach(()=>vi.unstubAllGlobals());
test('malformed economic server frame retains last snapshot and disconnects',()=>{
  const instances:any[]=[];class Socket{static OPEN=1;readyState=1;onopen:any;onmessage:any;onclose:any;onerror:any;send=vi.fn();close=vi.fn();constructor(){instances.push(this);}}
  vi.stubGlobal('WebSocket',Socket);vi.stubGlobal('location',{protocol:'http:',host:'localhost'});const onMessage=vi.fn(),onClose=vi.fn();const connection=connect(onMessage,onClose);const socket=instances[0];
  const bad=structuredClone(expected);bad.economy.nations[0].political_capital.value='NaN';const bytes=encode(bad);socket.onmessage({data:bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)});
  expect(onMessage).not.toHaveBeenCalled();expect(onClose).toHaveBeenCalledWith('invalid-server-message');expect(socket.close).toHaveBeenCalledOnce();connection.close();
});
