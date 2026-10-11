import {readFileSync} from 'node:fs';
import {expect,test} from 'vitest';
import {landSet,WorldAuthority} from './worldAuthority';
import type {WorldView,ServerMessage} from './proto/protocol';
const world=():WorldView=>JSON.parse(readFileSync('../target/wp09/national-wire-fixtures.json','utf8'))[0].expected.world;
const result=(request:string,w:WorldView|null=world()):Extract<ServerMessage,{type:'WorldResult'}>=>({type:'WorldResult',request,supported:true,reason_key:null,world:w});
test('REQ-MIL-03 land membership closes on duplicates, unknown refs, partial world and either direction of disagreement',()=>{
 const w=world(),land=landSet(w)!;expect(land).not.toBeNull();
 expect([...land].sort((a,b)=>a-b)).toEqual(w.provinces.filter(p=>p.state!==null).map(p=>p.id).sort((a,b)=>a-b));
 for(const mutate of [(v:WorldView)=>v.provinces.push(v.provinces[0]),(v:WorldView)=>v.states.push(v.states[0]),(v:WorldView)=>v.states[0].provinces.push(v.states[0].provinces[0]),(v:WorldView)=>v.states[1].provinces.push(v.states[0].provinces[0]),(v:WorldView)=>v.states[0].provinces.push(65534),(v:WorldView)=>v.provinces[0].state=null,(v:WorldView)=>v.states[0].provinces=[],(v:WorldView)=>v.states.pop(),(v:WorldView)=>v.provinces.pop()]){const v=world();mutate(v);expect(landSet(v)).toBeNull();}
});
test('only matched full world authorizes controller land, not owner/terrain/water; dirty refresh revokes authority',()=>{
 const inbox=new WorldAuthority(),a={},b={};inbox.begin(a);expect(inbox.issue(b,()=>true)).toBeNull();
 expect(inbox.issue(a,()=>false)).toBeNull();expect(inbox.issuedSerial).toBe(0n);
 const one=inbox.issue(a,()=>true)!;expect(inbox.accept(a,result('world'))).toBe(false);expect(inbox.accept(b,result(one))).toBe(false);
 const w=world();expect(inbox.accept(a,result(one,w))).toBe(true);expect(inbox.authority?.serial).toBe(1n);
 const own=w.provinces.find(p=>p.owner!==p.controller&&p.state!==null)!;expect(inbox.candidates(own.controller!)).toContain(own.id);expect(inbox.candidates(own.owner!)).not.toContain(own.id);
 expect(inbox.candidates(1)).not.toContain(w.provinces.find(p=>p.state===null)!.id);
 const two=inbox.issue(a,()=>true)!;expect(inbox.authority).toBeNull();for(let i=0;i<8;i++)expect(inbox.issue(a,()=>true)).toBeNull();
 expect(inbox.accept(a,result(two))).toBe(true);expect(inbox.followUp(a,()=>true)).toBe('deploy-world:3');expect(inbox.authority).toBeNull();expect(inbox.followUp(a,()=>true)).toBeNull();
});
test('unissued, duplicate, late epoch/socket, missing, malformed and unsupported results never restore authority',()=>{
 const inbox=new WorldAuthority(),a={},b={};inbox.begin(a);const old=inbox.issue(a,()=>true)!;inbox.begin(a);const cur=inbox.issue(a,()=>true)!;
 expect(inbox.accept(a,result(old))).toBe(false);expect(inbox.accept(a,result(cur,null))).toBe(true);expect(inbox.authority).toBeNull();
 const bad=inbox.issue(a,()=>true)!;const w=world();w.states=[];expect(inbox.accept(a,result(bad,w))).toBe(true);expect(inbox.authority).toBeNull();
 const nope=inbox.issue(a,()=>true)!;expect(inbox.accept(a,{type:'QueryResult',request:nope,supported:false,reason_key:'unsupported-query',state:null})).toBe(true);expect(inbox.authority).toBeNull();
 const yes=inbox.issue(a,()=>true)!;expect(inbox.accept(a,result(yes))).toBe(true);expect(inbox.accept(a,result(yes))).toBe(false);inbox.revoke();expect(inbox.authority).toBeNull();
 inbox.begin(b);expect(inbox.accept(a,result(yes))).toBe(false);expect(inbox.end(a)).toBe(false);expect(inbox.end(b)).toBe(true);expect(inbox.issue(b,()=>true)).toBeNull();
});
