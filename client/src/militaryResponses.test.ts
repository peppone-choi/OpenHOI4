import {readFileSync} from 'node:fs';
import {expect,test} from 'vitest';
import {MilitaryResponses} from './militaryResponses';
import type {ServerMessage} from './proto/protocol';
const result=JSON.parse(readFileSync(new URL('../../target/wp16/military-wire-fixture.json',import.meta.url),'utf8')) as Extract<ServerMessage,{type:'MilitaryResult'}>;
test('closed panel sends nothing and discards late replies across opening epochs',()=>{
  const inbox=new MilitaryResponses(),socket={},sent:string[]=[];
  const send=(request:string)=>{sent.push(request);return true;};
  expect(inbox.issue(socket,send)).toBeNull();
  inbox.begin(socket);const old=inbox.issue(socket,send)!;
  inbox.close();expect(inbox.accept(socket,{...result,request:old})).toBeNull();
  expect(inbox.issue(socket,send)).toBeNull();
  inbox.begin(socket);const current=inbox.issue(socket,send)!;
  expect(current).not.toBe(old);
  expect(inbox.accept(socket,{...result,request:old})).toBeNull();
  expect(inbox.accept(socket,{...result,request:current})?.status).toBe('ready');
  expect(sent).toEqual([old,current]);
});
test('continuous Delta coalesces pending updates into exactly one follow-up',()=>{
  const inbox=new MilitaryResponses(),socket={};inbox.begin(socket);
  const first=inbox.issue(socket,()=>true)!;expect(first).toBe('military:1');
  for(let i=0;i<20;i++)expect(inbox.issue(socket,()=>true)).toBeNull();
  expect(inbox.accept(socket,{...result,request:'military:999'})).toBeNull();
  expect(inbox.accept(socket,{...result,request:first})?.view).toBe(result.military);
  const second=inbox.followUp(socket,()=>true)!;expect(second).toBe('military:2');
  expect(inbox.followUp(socket,()=>true)).toBeNull();
  expect(inbox.accept(socket,{...result,request:first})).toBeNull();
  expect(inbox.accept(socket,{...result,request:second})).not.toBeNull();
  expect(inbox.accept(socket,{...result,request:second})).toBeNull();
  expect(inbox.followUp(socket,()=>true)).toBeNull();
});
test('identity changes reject old socket responses and invalidate dirty refresh',()=>{
  const inbox=new MilitaryResponses(),a={},b={};inbox.begin(a);
  const old=inbox.issue(a,()=>true)!;inbox.issue(a,()=>true);
  inbox.begin(b);expect(inbox.followUp(b,()=>true)).toBeNull();
  const current=inbox.issue(b,()=>true)!;
  expect(inbox.accept(a,{...result,request:old})).toBeNull();
  expect(inbox.accept(a,{...result,request:current})).toBeNull();
  expect(inbox.end(a)).toBe(false);
  expect(inbox.accept(b,{...result,request:current})).not.toBeNull();
  expect(inbox.end(b)).toBe(true);
  expect(inbox.issue(b,()=>true)).toBeNull();
});
test('CONNECTING refusal does not create a pending request or consume serial',()=>{
  const inbox=new MilitaryResponses(),socket={};inbox.begin(socket);
  expect(inbox.issue(socket,()=>false)).toBeNull();
  expect(inbox.accept(socket,{...result,request:'military:1'})).toBeNull();
  const first=inbox.issue(socket,()=>true)!;expect(first).toBe('military:1');
  expect(inbox.accept(socket,{...result,request:first})).not.toBeNull();
});
test('only correlated generic unsupported fallback can replace military data',()=>{
  const inbox=new MilitaryResponses(),socket={};inbox.begin(socket);
  const request=inbox.issue(socket,()=>true)!;
  expect(inbox.accept(socket,{type:'QueryResult',request:'world',supported:false,reason_key:'unsupported-query',state:null})).toBeNull();
  expect(inbox.accept(socket,{type:'QueryResult',request,supported:true,reason_key:null,state:{tick:'0',date:'2000-01-01',hour:0,paused:true,speed:1}})).toBeNull();
  expect(inbox.accept(socket,{type:'QueryResult',request,supported:false,reason_key:'unsupported-query',state:null})).toEqual({view:null,status:'unsupported',reasonKey:'unsupported-query'});
  const next=inbox.issue(socket,()=>true)!;
  expect(inbox.accept(socket,{type:'MilitaryResult',request:next,supported:false,reason_key:'not-joined',military:null})?.status).toBe('unsupported');
});
