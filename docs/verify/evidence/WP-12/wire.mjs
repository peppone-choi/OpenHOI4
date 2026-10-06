import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';
import assert from 'node:assert/strict';
import {stripTypeScriptTypes} from 'node:module';
import {encode,decode} from '../../client/node_modules/@msgpack/msgpack/dist.esm/index.mjs';
const root=path.resolve('.'),out=path.resolve('target/wp12-verify2');
const src=fs.readFileSync('client/src/network.ts','utf8');
const converted=stripTypeScriptTypes(src,{mode:'transform'}).replace("'@msgpack/msgpack'",JSON.stringify(pathToFileURL(root+'/client/node_modules/@msgpack/msgpack/dist.esm/index.mjs').href)).replace("'./proto/protocol'",JSON.stringify(pathToFileURL(root+'/client/src/proto/protocol.ts').href));
fs.writeFileSync(out+'/network.mjs',converted);
const {isServerMessage,connect}=await import(pathToFileURL(out+'/network.mjs'));
// Rust-generated fixture frames are the valid roots. Wrong-field candidates are
// judged by the independently compiled Rust decoder, not copied implementation assertions.
const roots=JSON.parse(fs.readFileSync('target/wp05/wire-fixtures.json')).filter(x=>['Welcome','CommandResult','Snapshot','Delta','QueryResult','Notice'].includes(x.expected.type)).map(x=>decode(Uint8Array.from(x.bytes)));
const cases=roots.map(value=>({label:value.type+'/normal',value}));
for(const v of roots){
 const paths=[];
 function walk(x,p=[]){if(x&&typeof x==='object'){for(const k of Object.keys(x)){paths.push([...p,k]);walk(x[k],[...p,k]);}}}
 walk(v);
 for(const p of paths)for(const bad of [null,{},[],false,true,0,255,256,-1,1.5,'bad']){
  const value=structuredClone(v);let dest=value;for(const k of p.slice(0,-1))dest=dest[k];dest[p.at(-1)]=bad;
  cases.push({label:v.type+'/'+p.join('.')+'/'+JSON.stringify(bad),value});
 }
 const extra={...v,unused_future_field:{anything:true}};cases.push({label:v.type+'/extra',value:extra});
}
const state={date:'server opaque date',hour:255,tick:'18446744073709551615',paused:true,speed:255};
for(const value of [{type:'Snapshot',state},{type:'Delta',sequence:'18446744073709551615',state},{type:'QueryResult',request:'x',supported:false,reason_key:null,state:null},{type:'QueryResult',request:'x',supported:true,reason_key:null,state}])cases.push({label:value.type+'/boundary-nullable',value});
const oracle=spawnSync(out+'/oracle/target/debug/wp12_verify_wire_oracle.exe',[],{input:cases.map(c=>Buffer.from(encode(c.value)).toString('hex')).join('\n')+'\n',encoding:'utf8'});
assert.equal(oracle.status,0,oracle.stderr);
const verdicts=oracle.stdout.trim().split(/\r?\n/).map(x=>x==='true');assert.equal(verdicts.length,cases.length);
const mismatches=[],outsideGeneratedUnion=[];let rejected=0,accepted=0;
class Socket { static OPEN=1;static latest;readyState=1;onmessage=null;onopen=null;onerror=null;onclose=null;sent=[];closed=0;constructor(){Socket.latest=this;}send(x){this.sent.push(decode(x));}close(){this.closed++;this.readyState=3;this.onclose?.();}receive(x){this.onmessage?.({data:encode(x).slice().buffer});} }
globalThis.WebSocket=Socket;globalThis.location={protocol:'http:',host:'127.0.0.1:19412'};
for(let i=0;i<cases.length;i++){
 const c=cases[i],want=verdicts[i],got=isServerMessage(decode(encode(c.value)));
 if(got!==want){ if(want && !got && typeof c.value.type !== 'string') outsideGeneratedUnion.push({label:c.label,rust:want,ts:got,reason:'Serde accepts numeric enum discriminant; Rust serializer and generated wire contract use string literals'}); else mismatches.push({label:c.label,rust:want,ts:got}); }
 if(!want){
  rejected++;const messages=[],closures=[];const connection=connect(x=>messages.push(x),x=>closures.push(x));const socket=Socket.latest;
  socket.receive(c.value);assert.deepEqual(messages,[],c.label);assert.deepEqual(closures,['invalid-server-message'],c.label);assert.equal(socket.closed,1,c.label);
  socket.receive(roots[0]);socket.onerror?.();socket.onclose?.();connection.send({type:'Query',request:'verify',kind:'time'});
  assert.deepEqual(messages,[],c.label);assert.deepEqual(socket.sent,[],c.label);assert.equal(closures.length,1,c.label);
 }else accepted++;
}
// All variants are delivered unaltered; full decimal u64 strings stay strings.
const received=[],closed=[];connect(x=>received.push(x),x=>closed.push(x));const socket=Socket.latest;
for(const value of [...roots,{type:'Snapshot',state}])socket.receive(value);
assert.deepEqual(received,[...roots,{type:'Snapshot',state}]);assert.equal(typeof received.at(-1).state.tick,'string');assert.deepEqual(closed,[]);

// Generated TS declares explicit required fields, including nullable fields.
// Verify omissions separately from serde's permissive missing Option behavior.
let omittedChecks=0,transportChecks=0;
function mustReject(value,label){
 const messages=[],closures=[];const connection=connect(x=>messages.push(x),x=>closures.push(x));const socket=Socket.latest;
 socket.receive(value);assert.deepEqual(messages,[],label);assert.deepEqual(closures,['invalid-server-message'],label);assert.equal(socket.closed,1,label);
 socket.receive(roots[0]);connection.send({type:'Query',request:'late',kind:'time'});assert.deepEqual(socket.sent,[],label);assert.deepEqual(messages,[],label);
}
for(const v of roots){
 const paths=[];
 function walk(x,p=[]){if(x&&typeof x==='object'&&!Array.isArray(x))for(const k of Object.keys(x)){paths.push([...p,k]);walk(x[k],[...p,k]);}else if(Array.isArray(x))x.forEach((item,i)=>walk(item,[...p,String(i)]));}
 walk(v);
 for(const p of paths){const value=structuredClone(v);let dest=value;for(const k of p.slice(0,-1))dest=dest[k];delete dest[p.at(-1)];mustReject(value,v.type+'/missing/'+p.join('.'));omittedChecks++;}
}
for(const field of ['hour','speed'])for(const bad of [NaN,Infinity,-Infinity]){mustReject({type:'Snapshot',state:{...state,[field]:bad}},field+'/'+String(bad));transportChecks++;}
for(const data of ['text',new Uint8Array([193]).buffer,new Uint8Array([0xc1]),null]){
 const messages=[],closures=[];const connection=connect(x=>messages.push(x),x=>closures.push(x));const socket=Socket.latest;
 socket.onmessage({data});assert.deepEqual(messages,[]);assert.deepEqual(closures,['invalid-server-message']);assert.equal(socket.closed,1);connection.send({type:'Query',request:'late',kind:'time'});assert.deepEqual(socket.sent,[]);transportChecks++;
}

const report={omittedChecks,transportChecks,cases:cases.length,rejected,accepted,mismatches,outsideGeneratedUnion,connection_checks:'invalid frames: no callbacks, close once, no later receive/send; normal variants preserved'};
fs.writeFileSync(out+'/wire-oracle.json',JSON.stringify(report,null,2));console.log(report);assert.deepEqual(mismatches,[]);
