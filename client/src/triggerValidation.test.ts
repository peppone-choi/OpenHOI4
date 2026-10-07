import { test,expect } from 'vitest';
import { isServerMessage } from './network';
import { encode,decode } from '@msgpack/msgpack';
const result={type:'TriggerResult',request:'end',supported:true,reason_key:null,trigger:{definitions_hash:'1234567890abcdef',flags:[{nation:1,keys:['x']}],ended:{tick:'48',date:'2000-03-01',hour:0,causes:[{type:'Date'},{type:'Condition'},{type:'Explicit',source:'a'}]}}};
test('trigger authoritative query roundtrips and rejects malformed new fields',()=>{
  expect(isServerMessage(decode(encode(result)))).toBe(true);
  for(const trigger of [null,{...result.trigger,definitions_hash:1},{...result.trigger,flags:[{nation:65536,keys:[]}]},{...result.trigger,flags:[{nation:1,keys:[null]}]},{...result.trigger,ended:{...result.trigger.ended,tick:48}},{...result.trigger,ended:{...result.trigger.ended,causes:[{type:'Explicit',source:null}]}},{...result.trigger,ended:{...result.trigger.ended,causes:[{type:'Date',source:'x'}]}}]){
    expect(isServerMessage({...result,trigger})).toBe(false);
  }
  for(const key of Object.keys(result.trigger)){const missing={...result.trigger} as Record<string,unknown>;delete missing[key];expect(isServerMessage({...result,trigger:missing})).toBe(false);}
});
