import {readFileSync} from 'node:fs';
import {test,expect} from 'vitest';
import {CommandLedger} from './commandLedger';
import {MilitaryCommands} from './militaryCommands';
import type {MilitaryView} from './proto/protocol';
const view=JSON.parse(readFileSync(new URL('../../target/wp16/military-wire-fixture.json',import.meta.url),'utf8')).military as MilitaryView;
const train={type:'Train' as const,template:view.templates[0].template};
const reply=(sequence:string,accepted=true)=>({type:'CommandResult' as const,sequence,accepted,reason_key:accepted?null:'invalid-message'});
test('shared sequence commits only successful sends; bounded issued ledger discards unknown/duplicate/old-socket results',()=>{
  const ledger=new CommandLedger(),a={},b={};ledger.begin(a);
  expect(ledger.issue(a,{type:'Command',command:{type:'Pause',paused:true}},()=>false)).toBeNull();
  const one=ledger.issue(a,{type:'Command',command:{type:'Pause',paused:true}},()=>true)!;expect(one).toBe('1');
  const two=ledger.issue(a,{type:'MilitaryCommand',command:train},()=>true)!;expect(two).toBe('2');
  expect(ledger.accept(a,reply('999'))).toBeNull();expect(ledger.accept(b,reply(two))).toBeNull();
  expect(ledger.accept(a,reply(two))).toBe('MilitaryCommand');expect(ledger.accept(a,reply(two))).toBeNull();
  ledger.begin(b);expect(ledger.accept(a,reply(one))).toBeNull();
  for(let i=0;i<64;i++)expect(ledger.issue(b,{type:'Command',command:{type:'Pause',paused:true}},()=>true)).not.toBeNull();
  expect(ledger.issue(b,{type:'MilitaryCommand',command:train},()=>true)).toBeNull();
});
test('CommandResult establishes query serial barrier: pre-result authority cannot release pending; next fresh view can',()=>{
  const channel=new MilitaryCommands(),socket={};channel.begin(socket,1);channel.setAuthority(socket,view,3n);
  expect(channel.issue(socket,null,train,()=>null)).toBeNull();expect(channel.feedback.status).toBe('idle');
  expect(channel.issue(socket,null,train,()=>'7')).toBe('7');expect(channel.can(socket,null,train)).toBe(false);
  expect(channel.accept(socket,reply('6'),4n)).toBe(false);expect(channel.accept(socket,reply('7'),4n)).toBe(true);
  channel.setAuthority(socket,view,4n);expect(channel.feedback.status).toBe('refreshing');expect(channel.can(socket,null,train)).toBe(false);
  channel.setAuthority(socket,null,5n);expect(channel.feedback.status).toBe('refreshing');
  expect(channel.accept(socket,reply('7'),5n)).toBe(false);
  channel.setAuthority(socket,view,5n);expect(channel.feedback.status).toBe('success');expect(channel.can(socket,null,train)).toBe(true);
});
test('spectator, foreign filter/jobs, terminal jobs, missing authority and scope/socket transitions cannot authorize',()=>{
  const channel=new MilitaryCommands(),a={},b={};channel.begin(a,null);channel.setAuthority(a,view,1n);expect(channel.can(a,null,train)).toBe(false);
  channel.begin(a,1);channel.setAuthority(a,view,2n);expect(channel.can(a,2,train)).toBe(false);
  const own={...view,jobs:view.jobs.map(j=>({...j,nation:1,status:'Training' as const}))};channel.setAuthority(a,own,3n);
  const cancel={type:'Cancel' as const,job:own.jobs[0].id};expect(channel.can(a,null,cancel)).toBe(true);
  for(const status of ['Cancelled','Deployed'] as const){channel.setAuthority(a,{...own,jobs:own.jobs.map(j=>({...j,status}))},4n);expect(channel.can(a,null,cancel)).toBe(false);}
  channel.setAuthority(a,{...own,jobs:own.jobs.map(j=>({...j,nation:2}))},5n);expect(channel.can(a,null,cancel)).toBe(false);
  channel.setAuthority(a,null,6n);expect(channel.can(a,null,train)).toBe(false);
  channel.setAuthority(a,own,7n);channel.issue(a,null,train,()=>'8');channel.begin(b,1);
  expect(channel.accept(a,reply('8'),8n)).toBe(false);channel.setAuthority(a,own,8n);expect(channel.can(b,null,train)).toBe(false);
  channel.setAuthority(b,own,9n);channel.issue(b,null,train,()=>'9');expect(channel.accept(b,reply('9',false),9n)).toBe(true);
  channel.setAuthority(b,own,10n);expect(channel.feedback).toEqual({status:'rejected',reasonKey:'invalid-message'});channel.end(b);expect(channel.can(b,null,train)).toBe(false);
});
test('filter scope preserves the query slot but requires a query issued after the scope change for authority',()=>{
  const channel=new MilitaryCommands(),socket={};channel.begin(socket,1);channel.setAuthority(socket,view,2n);channel.issue(socket,null,train,()=>'11');
  channel.begin(socket,1,3n);expect(channel.accept(socket,reply('11'),3n)).toBe(false);
  channel.setAuthority(socket,view,3n);expect(channel.can(socket,null,train)).toBe(false);
  channel.setAuthority(socket,view,4n);expect(channel.can(socket,null,train)).toBe(true);
});

test('REQ-MIL-03 Deploy requires canonical owned Ready references and controlled land; two ACK barriers prevent reactivation',()=>{
 const c=new MilitaryCommands(),socket={};c.begin(socket,1);
 const ready={...view,jobs:[{...view.jobs[0],nation:1,status:'Ready' as const}],armies:[{...view.armies[0],nation:1}]};c.setAuthority(socket,ready,1n);
 const w=JSON.parse(readFileSync(new URL('../../target/wp09/national-wire-fixtures.json',import.meta.url),'utf8'))[0].expected.world;
 const province=w.provinces.find((p:{state:number|null;controller:number|null})=>p.state!==null&&p.controller===1).id;
 const deploy={type:'Deploy' as const,job:ready.jobs[0].id,army:ready.armies[0].id,province,allow_understrength:false};
 expect(c.can(socket,null,deploy)).toBe(false);c.setWorldAuthority(socket,w,1n);expect(c.can(socket,null,deploy)).toBe(true);
 for(const invalid of [{...deploy,job:'01'},{...deploy,army:'18446744073709551616'},{...deploy,province:65536},{...deploy,allow_understrength:undefined},{...deploy,army:'999'},{...deploy,province:w.provinces.find((p:{state:number|null;controller:number|null})=>p.state!==null&&p.controller!==1).id},{...deploy,province:w.provinces.find((p:{state:number|null})=>p.state===null).id}])expect(c.can(socket,null,invalid as typeof deploy)).toBe(false);
 expect(c.can(socket,2,deploy)).toBe(false);expect(c.can(socket,null,{...deploy,allow_understrength:true})).toBe(true);
 for(const status of ['Pending','Training','Cancelled','Deployed'] as const){c.setAuthority(socket,{...ready,jobs:[{...ready.jobs[0],status}]},2n);expect(c.can(socket,null,deploy)).toBe(false);}
 c.setAuthority(socket,{...ready,armies:[{...ready.armies[0],nation:2}]},3n);expect(c.can(socket,null,deploy)).toBe(false);
 c.setAuthority(socket,{...ready,jobs:[{...ready.jobs[0],nation:2}]},4n);expect(c.can(socket,null,deploy)).toBe(false);
 c.setAuthority(socket,ready,5n);c.issue(socket,null,deploy,()=>'55');expect(c.accept(socket,reply('55'),5n,1n)).toBe(true);
 c.setAuthority(socket,ready,6n);expect(c.feedback.status).toBe('refreshing');c.setWorldAuthority(socket,w,1n);expect(c.feedback.status).toBe('refreshing');
 c.setWorldAuthority(socket,w,2n);expect(c.feedback.status).toBe('success');expect(c.can(socket,null,deploy)).toBe(true);
 c.setWorldAuthority(socket,null,2n);expect(c.can(socket,null,deploy)).toBe(false);expect(c.can(socket,null,train)).toBe(true);
});

test('Deploy world-first ACK refresh still waits for newer military; rejection and new scope do not revive old authority',()=>{
 const c=new MilitaryCommands(),a={},b={};c.begin(a,1);
 const ready={...view,jobs:[{...view.jobs[0],nation:1,status:'Ready' as const}],armies:[{...view.armies[0],nation:1}]};
 const w=JSON.parse(readFileSync(new URL('../../target/wp09/national-wire-fixtures.json',import.meta.url),'utf8'))[0].expected.world;
 const province=w.provinces.find((p:{state:number|null;controller:number|null})=>p.state!==null&&p.controller===1).id;
 const deploy={type:'Deploy' as const,job:ready.jobs[0].id,army:ready.armies[0].id,province,allow_understrength:true};
 c.setAuthority(a,ready,2n);c.setWorldAuthority(a,w,3n);c.issue(a,null,deploy,()=>'12');c.accept(a,reply('12',false),2n,3n);
 c.setWorldAuthority(a,w,4n);expect(c.feedback.status).toBe('refreshing');c.setAuthority(a,ready,2n);expect(c.feedback.status).toBe('refreshing');c.setAuthority(a,ready,3n);expect(c.feedback.status).toBe('rejected');
 c.begin(b,1,3n,4n);c.setAuthority(b,ready,4n);c.setWorldAuthority(a,w,5n);c.setWorldAuthority(b,w,4n);expect(c.can(b,null,deploy)).toBe(false);c.setWorldAuthority(b,w,5n);expect(c.can(b,null,deploy)).toBe(true);
});
