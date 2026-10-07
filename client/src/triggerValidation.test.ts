import { test,expect,vi,afterEach } from 'vitest';
import { isServerMessage,connect } from './network';
import { encode,decode } from '@msgpack/msgpack';
const result={type:'TriggerResult',request:'end',supported:true,reason_key:null,trigger:{definitions_hash:'1234567890abcdef',flags:[{nation:1,keys:['x']}],ended:{tick:'48',date:'2000-03-01',hour:0,causes:[{type:'Date'},{type:'Condition'},{type:'Explicit',source:'a'}]}}};
test('trigger authoritative query roundtrips and rejects malformed new fields',()=>{
  expect(isServerMessage(decode(encode(result)))).toBe(true);
  for(const trigger of [null,{...result.trigger,definitions_hash:1},{...result.trigger,flags:[{nation:65536,keys:[]}]},{...result.trigger,flags:[{nation:1,keys:[null]}]},{...result.trigger,ended:{...result.trigger.ended,tick:48}},{...result.trigger,ended:{...result.trigger.ended,causes:[{type:'Explicit',source:null}]}},{...result.trigger,ended:{...result.trigger.ended,causes:[{type:'Date',source:'x'}]}}]){
    expect(isServerMessage({...result,trigger})).toBe(false);
  }
  for(const key of Object.keys(result.trigger)){const missing={...result.trigger} as Record<string,unknown>;delete missing[key];expect(isServerMessage({...result,trigger:missing})).toBe(false);}
});

const sorted={...result,trigger:{...result.trigger,flags:[{nation:1,keys:['a','z']},{nation:42,keys:[]}],ended:{...result.trigger.ended,causes:[{type:'Date'},{type:'Condition'},{type:'Explicit',source:'a'},{type:'Explicit',source:'z'}]}}};
const malformed:[string,unknown][]=[
  ['duplicate nation',{...sorted.trigger,flags:[{nation:1,keys:[]},{nation:1,keys:[]}]}],
  ['nation order',{...sorted.trigger,flags:[{nation:42,keys:[]},{nation:1,keys:[]}]}],
  ['duplicate flag',{...sorted.trigger,flags:[{nation:1,keys:['a','a']}]}],
  ['flag order',{...sorted.trigger,flags:[{nation:1,keys:['z','a']}]}],
  ['empty flag ID',{...sorted.trigger,flags:[{nation:1,keys:['']}]}],
  ...['UPPER','a.b','a-b','a b','한글','a\0','a\n'].map((key):[string,unknown]=>['flag syntax '+JSON.stringify(key),{...sorted.trigger,flags:[{nation:1,keys:[key]}]}]),
  ...[-1,65536,1.5,true,'1',null].map((nation):[string,unknown]=>['nation type/range '+JSON.stringify(nation),{...sorted.trigger,flags:[{nation,keys:[]}]}]),
  ['unknown nation member',{...sorted.trigger,flags:[{nation:1,keys:[],intruder:true}]}],
  ['empty causes',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[]}}],
  ['duplicate Date',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Date'},{type:'Date'}]}}],
  ['duplicate Condition',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Condition'},{type:'Condition'}]}}],
  ['duplicate Explicit',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Explicit',source:'a'},{type:'Explicit',source:'a'}]}}],
  ['cause kind order',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Condition'},{type:'Date'}]}}],
  ['explicit before Condition',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Explicit',source:'a'},{type:'Condition'}]}}],
  ['explicit source order',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Explicit',source:'z'},{type:'Explicit',source:'a'}]}}],
  ...['','UPPER','a.b','a-b','a b','漢','a\0','a\n'].map((source):[string,unknown]=>['source syntax '+JSON.stringify(source),{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Explicit',source}]}}]),
  ['unknown Explicit member',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Explicit',source:'a',intruder:true}]}}],
  ['wrong Explicit field',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Explicit',id:'a'}]}}],
  ['Date has source',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Date',source:'a'}]}}],
  ['unknown cause type',{...sorted.trigger,ended:{...sorted.trigger.ended,causes:[{type:'Unknown'}]}}],
  ['unknown end member',{...sorted.trigger,ended:{...sorted.trigger.ended,intruder:true}}],
  ['unknown trigger member',{...sorted.trigger,intruder:true}],
  ...['18446744073709551616','99999999999999999999','100000000000000000000','','01','-1','1e3','1\n',1,null].map((tick):[string,unknown]=>['u64 tick '+JSON.stringify(tick),{...sorted.trigger,ended:{...sorted.trigger.ended,tick}}]),
];
class Socket{
  static OPEN=1;static latest:Socket;
  readyState=1;binaryType='';onopen:(()=>void)|null=null;onmessage:((event:{data:unknown})=>void)|null=null;onclose:(()=>void)|null=null;onerror:(()=>void)|null=null;
  sent:unknown[]=[];closeCount=0;
  constructor(){Socket.latest=this;}
  send(value:unknown){this.sent.push(value);}
  close(){this.closeCount++;this.readyState=3;}
  receive(value:unknown){this.onmessage?.({data:encode(value).slice().buffer});}
}
afterEach(()=>vi.unstubAllGlobals());
function rejectWithoutPublication(message:unknown){
  vi.stubGlobal('WebSocket',Socket);vi.stubGlobal('location',{protocol:'http:',host:'localhost'});
  const calls:unknown[]=[];const reasons:unknown[]=[];const connection=connect(m=>calls.push(m),reason=>reasons.push(reason));
  const previous={type:'Snapshot',state:{date:'2000-01-01',hour:0,tick:'0',paused:false,speed:1}};
  Socket.latest.receive(previous);Socket.latest.receive(message);
  connection.send({type:'Command',sequence:'1',command:{type:'Pause',paused:false}});
  Socket.latest.receive(sorted);
  expect(calls).toEqual([previous]);expect(reasons).toEqual(['invalid-server-message']);expect(Socket.latest.closeCount).toBe(1);expect(Socket.latest.sent).toEqual([]);
}
for(const [name,trigger] of malformed){
  test(`P06 trigger shape/order/id/range rejects ${name}`,()=>expect(isServerMessage({...sorted,trigger})).toBe(false));
  test(`P06 invalid wire preserves callback and blocks command: ${name}`,()=>rejectWithoutPublication({...sorted,trigger}));
}
test('P06 TriggerResult exact outer members before publication',()=>{
  rejectWithoutPublication({...sorted,intruder:true});
});
test('P06 accepts unsupported, active, tick0, empty flags, bounded u64 and multiple causes',()=>{
  for(const message of [
    {type:'TriggerResult',request:'legacy',supported:false,reason_key:'unsupported-query',trigger:null},
    {...sorted,trigger:{...sorted.trigger,ended:null}},
    {...sorted,trigger:{...sorted.trigger,flags:[],ended:{...sorted.trigger.ended,tick:'0',causes:[{type:'Condition'}]}}},
    {...sorted,trigger:{...sorted.trigger,flags:[{nation:0,keys:['0','_','a_0']},{nation:65535,keys:[]}],ended:{...sorted.trigger.ended,tick:'18446744073709551615'}}},
    sorted,
  ])expect(isServerMessage(decode(encode(message)))).toBe(true);
});
