import {expect,test} from 'vitest';
import {MilitaryLedgerResponses} from './militaryLedgerResponses';

test('capability first, one pending, and A-B-A selection invalidates the first A reply',()=>{
  const inbox=new MilitaryLedgerResponses(),socket={},sent:{request:string;kind:string}[]=[];
  const send=(request:string,kind:string)=>{sent.push({request,kind});return true;};
  inbox.begin(socket);inbox.select('a');inbox.issue(socket,send);
  expect(sent[0].kind).toBe('military-normal-ledger.v1');
  inbox.accept(socket,{type:'MilitaryNormalLedgerCapabilityResult',request:sent[0].request,supported:true,reason_key:null});
  inbox.issue(socket,send);const first=sent[1];expect(first.kind).toBe('military-normal-ledger.v1:a');
  inbox.select('b');inbox.issue(socket,send);inbox.select('a');inbox.issue(socket,send);
  expect(sent).toHaveLength(2);
  expect(inbox.accept(socket,{type:'MilitaryNormalLedgerResult',request:first.request,supported:false,reason_key:'unknown-template',ledger:null})).toBeNull();
  inbox.issue(socket,send);expect(sent).toHaveLength(3);expect(sent[2].kind).toBe(first.kind);expect(sent[2].request).not.toBe(first.request);
});
test('old-server capability fallback stops only this channel; domain failure does not',()=>{
  const socket={},inbox=new MilitaryLedgerResponses(),sent:string[]=[];
  const send=(request:string)=>{sent.push(request);return true;};
  inbox.begin(socket);inbox.select('a');inbox.issue(socket,send);
  expect(inbox.accept(socket,{type:'QueryResult',request:sent[0],supported:false,reason_key:'unsupported-query',state:null})?.status).toBe('unsupported');
  inbox.select('b');inbox.issue(socket,send);expect(sent).toHaveLength(1);
  inbox.begin(socket);inbox.select('a');inbox.issue(socket,send);
  inbox.accept(socket,{type:'MilitaryNormalLedgerCapabilityResult',request:sent[1],supported:true,reason_key:null});inbox.issue(socket,send);
  expect(inbox.accept(socket,{type:'MilitaryNormalLedgerResult',request:sent[2],supported:false,reason_key:'ledger-too-large',ledger:null})?.status).toBe('unavailable');
  inbox.select('b');inbox.issue(socket,send);expect(sent).toHaveLength(4);
});
test('CONNECTING, close/reopen, old/future/duplicate replies, and wrong socket cannot take a slot',()=>{
  const socket={},oldSocket={},inbox=new MilitaryLedgerResponses(),sent:string[]=[];
  const send=(request:string)=>{sent.push(request);return true;};
  inbox.begin(socket);inbox.select('a');expect(inbox.issue(socket,()=>false)).toBeNull();inbox.issue(socket,send);
  const old=sent[0];inbox.close();inbox.begin(socket);inbox.select('a');inbox.issue(socket,send);
  const reply={type:'MilitaryNormalLedgerCapabilityResult' as const,request:sent[1],supported:true,reason_key:null};
  expect(inbox.accept(oldSocket,reply)).toBeNull();expect(inbox.accept(socket,{...reply,request:old})).toBeNull();expect(inbox.accept(socket,{...reply,request:'military-ledger:999'})).toBeNull();
  expect(inbox.accept(socket,reply)).not.toBeNull();expect(inbox.accept(socket,reply)).toBeNull();
  inbox.issue(socket,send);expect(sent).toHaveLength(3);expect(inbox.end(oldSocket)).toBe(false);expect(inbox.end(socket)).toBe(true);
});
