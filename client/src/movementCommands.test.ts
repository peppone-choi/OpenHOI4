import {expect,test} from 'vitest';
import {MovementCommands,MovementResponses} from './movementCommands';
import {CommandLedger} from './commandLedger';
import type {MilitaryView,ServerMessage} from './proto/protocol';
const result=(request:string):Extract<ServerMessage,{type:'MovementResult'}>=>({type:'MovementResult',request,supported:true,reason_key:null,movement:{tick:'0',state_hash:'s',units:[]}});
const ack=(sequence:string,accepted=true):Extract<ServerMessage,{type:'CommandResult'}>=>({type:'CommandResult',sequence,accepted,reason_key:accepted?null:'invalid-message'});
const base={divisions:[{id:'10',nation:1},{id:'2',nation:1},{id:'3',nation:2}]} as MilitaryView;
test('query is open-panel only, coalesces dirty refresh, discards old socket/epoch/serial',()=>{
 const r=new MovementResponses(),a={},b={};expect(r.issue(a,()=>true)).toBeNull();r.begin(a);
 expect(r.issue(a,()=>false)).toBeNull();const first=r.issue(a,()=>true)!;expect(first).toBe('movement:1');
 for(let i=0;i<10;i++)expect(r.issue(a,()=>true)).toBeNull();expect(r.accept(b,result(first))).toBeNull();expect(r.accept(a,result('movement:99'))).toBeNull();
 expect(r.accept(a,result(first))?.status).toBe('ready');const second=r.followUp(a,()=>true)!;expect(second).toBe('movement:2');expect(r.followUp(a,()=>true)).toBeNull();
 r.close();r.begin(a);const third=r.issue(a,()=>true)!;expect(r.accept(a,result(second))).toBeNull();expect(r.accept(a,result(third))).not.toBeNull();
 r.begin(b);expect(r.end(a)).toBe(false);expect(r.accept(a,result(third))).toBeNull();expect(r.end(b)).toBe(true);
});
test('generic unsupported requires correlation and unsupported null-state',()=>{
 const r=new MovementResponses(),s={};r.begin(s);const request=r.issue(s,()=>true)!;
 expect(r.accept(s,{type:'QueryResult',request,supported:true,reason_key:null,state:null})).toBeNull();
 expect(r.accept(s,{type:'QueryResult',request,supported:false,reason_key:'unsupported-query',state:null})).toEqual({view:null,status:'unsupported',reasonKey:'unsupported-query'});
});
test('own selection sorted, one command each, waits all ACKs AND post-last-ACK query',()=>{
 const c=new MovementCommands(),s={},sent:string[]=[];c.begin(s,1);c.setMilitary(s,base);
 expect(c.issue(s,['3'],14,()=>{throw Error('foreign');})).toBe(false);
 expect(c.issue(s,['10','2','10'],14,cmd=>{sent.push(cmd.division);return String(sent.length);})).toBe(true);
 expect(sent).toEqual(['2','10']);expect(c.busy).toBe(true);expect(c.issue(s,['2'],null,()=>null)).toBe(false);
 expect(c.accept(s,ack('2',false),3n)).toBe(false);c.setMovement(s,4n);expect(c.busy).toBe(true);
 expect(c.accept(s,ack('1'),4n)).toBe(true);c.setMovement(s,4n);expect(c.busy).toBe(true);c.setMovement(s,5n);expect(c.busy).toBe(false);
 expect(c.feedback.map(f=>[f.division,f.status,f.reasonKey])).toEqual([['2','accepted',null],['10','rejected','invalid-message']]);
});
test('shared ledger capacity stops batch, marks remaining unsent, never retries',()=>{
 const c=new MovementCommands(),l=new CommandLedger(),s={};c.begin(s,1);c.setMilitary(s,base);l.begin(s);
 for(let i=0;i<63;i++)l.issue(s,{type:'MovementCommand',command:{type:'Stop',division:'2'}},()=>true);
 const sent:string[]=[];expect(c.issue(s,['10','2'],null,command=>l.issue(s,{type:'MovementCommand',command},()=>{sent.push(command.division);return true;}))).toBe(true);
 expect(sent).toEqual(['2']);expect(c.feedback[1].status).toBe('not-sent');expect(c.accept(s,ack('64'),0n)).toBe(true);c.setMovement(s,1n);expect(c.busy).toBe(false);expect(sent).toEqual(['2']);
 c.begin(s,1);c.setMilitary(s,base);c.issue(s,['2','10'],14,()=>null);expect(c.busy).toBe(false);expect(c.feedback.every(f=>f.status==='not-sent')).toBe(true);
});
test('connection/Join changes clear pending and reject late ACK/authority',()=>{
 const c=new MovementCommands(),a={},b={};c.begin(a,1);c.setMilitary(a,base);c.issue(a,['2'],40,()=> '1');c.begin(b,2);c.setMilitary(a,base);
 expect(c.accept(a,ack('1'),0n)).toBe(false);expect(c.busy).toBe(false);expect(c.feedback).toEqual([]);expect(c.issue(b,['2'],14,()=> '2')).toBe(false);
 c.end(a);c.setMilitary(b,base);expect(c.issue(b,['3'],40,()=> '3')).toBe(true);c.end(b);expect(c.busy).toBe(false);
});
