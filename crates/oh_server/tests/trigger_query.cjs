const assert = require('node:assert/strict');
const fs = require('node:fs');
const {encode,decode} = require('../../../client/node_modules/@msgpack/msgpack');
async function main(){
  const [url,expectedPath,caseName] = process.argv.slice(2);
  const expected=JSON.parse(fs.readFileSync(expectedPath,'utf8')).cases[caseName];
  const dto=expected.dto, base=dto.base.base.base, t=dto.trigger;
  const socket=new WebSocket(url.replace('http:','ws:')+'ws');
  const messages=[],waiters=[];
  socket.addEventListener('message',event=>{
    const m=decode(new Uint8Array(event.data));messages.push(m);
    for(const w of [...waiters])if(w.p(m)){clearTimeout(w.timer);waiters.splice(waiters.indexOf(w),1);w.resolve(m);}
  });
  socket.binaryType='arraybuffer';
  const wait=p=>new Promise((resolve,reject)=>{const seen=messages.find(p);if(seen){resolve(seen);return;}const w={p,resolve,timer:setTimeout(()=>reject(Error('wire timeout')),5000)};waiters.push(w);});
  await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
  socket.send(encode({type:'Hello',protocol_version:'m0-v1'}));assert.equal((await wait(m=>m.type==='Welcome')).accepted,true);
  socket.send(encode({type:'Join',session:'local',nation:null}));
  const snapshot=await wait(m=>m.type==='Snapshot');assert.equal(snapshot.state.tick,String(base.state.tick));assert.equal(snapshot.state.paused,base.state.paused);
  async function query(request,kind,type){socket.send(encode({type:'Query',request,kind}));return await wait(m=>m.type===type&&m.request===request);}
  const before=await query('before','trigger','TriggerResult');assert.equal(before.supported,true);
  const causes=e=>e.causes.map(c=>typeof c==='string'?{type:c}:{type:'Explicit',source:c.Explicit});
  assert.deepEqual(before.trigger,{definitions_hash:BigInt(t.definitions_hash).toString(16).padStart(16,'0'),flags:t.flags.map(([nation,keys])=>({nation,keys})),ended:t.ended?{tick:String(t.ended.tick),date:`${t.ended.date.year}-${String(t.ended.date.month).padStart(2,'0')}-${String(t.ended.date.day).padStart(2,'0')}`,hour:t.ended.hour,causes:causes(t.ended)}:null});
  // Native JSON's large integers need exact text: supplied safe projection below overrides hash.
  for(const sequence of ['1','2']){
    socket.send(encode({type:'Command',sequence,command:sequence==='1'?{type:'Pause',paused:true}:{type:'SetSpeed',speed:5}}));
    const reply=await wait(m=>m.type==='CommandResult'&&m.sequence===sequence);
    if(t.ended){assert.equal(reply.accepted,false);assert.equal(reply.reason_key,'scenario-ended');}
    else assert.equal(reply.accepted,true);
  }
  const after=await query('after','trigger','TriggerResult');assert.deepEqual(after,before.type?{...before,request:'after'}:before);
  const world=await query('world','world','WorldResult');assert.equal(world.supported,true);assert.equal(world.world.tick,String(base.state.tick));
  const time=await query('time','time','QueryResult');assert.equal(time.supported,true);assert.equal(time.state.tick,String(base.state.tick));
  socket.close();console.log(JSON.stringify({pid:process.pid,snapshot,before,after,world,time,messages}));
}
main().catch(e=>{console.error(e);process.exit(1);});
