from pathlib import Path
p=Path('E:/openhoi/.orchestrator/wt/WP-12-verify2/target/wp12-verify2/wire.mjs')
s=p.read_text(encoding='utf-8')
new="""
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
"""
s=s.replace('const report={cases:',new+'\nconst report={omittedChecks,transportChecks,cases:')
p.write_text(s,encoding='utf-8')
