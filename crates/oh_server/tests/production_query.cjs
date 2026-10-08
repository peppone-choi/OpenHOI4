// Separate-process production wire acceptance: run against the generated V6 paused fixture.
const assert=require('node:assert/strict'),fs=require('node:fs');
const {encode,decode}=require(process.cwd()+'/client/node_modules/@msgpack/msgpack');
async function main(){
 const url=process.argv[2],expected=JSON.parse(fs.readFileSync(process.argv[3],'utf8')),messages=[],waiters=[];
 const ws=new WebSocket(url.replace('http','ws')+'ws');ws.binaryType='arraybuffer';
 ws.addEventListener('message',event=>{const m=decode(new Uint8Array(event.data));messages.push(m);for(const w of [...waiters])if(w.p(m)){clearTimeout(w.timer);waiters.splice(waiters.indexOf(w),1);w.resolve(m);}});
 const wait=p=>new Promise((resolve,reject)=>{const m=messages.find(p);if(m){resolve(m);return;}const w={p,resolve,timer:setTimeout(()=>reject(Error('production wire timeout')),5000)};waiters.push(w);});
 const send=m=>ws.send(encode(m));await new Promise((resolve,reject)=>{ws.addEventListener('open',resolve,{once:true});ws.addEventListener('error',reject,{once:true});});
 send({type:'Hello',protocol_version:'m0-v1'});const welcome=await wait(m=>m.type==='Welcome');assert.equal(welcome.accepted,true);
 send({type:'Join',session:'local',nation:'NTH'});const snapshot=await wait(m=>m.type==='Snapshot');assert.equal(snapshot.state.tick,'24');assert.equal(snapshot.state.paused,true);
 const query=async request=>{send({type:'Query',request,kind:'production'});const v=await wait(m=>m.type==='ProductionResult'&&m.request===request);assert.equal(v.supported,true);return v.production;};
 const before=await query('before');assert.equal(before.state_hash,expected.paused.hash);assert.equal(before.lines[0].carry.bits,'16384');assert.equal(before.lines[0].efficiency.bits,'1107296256');assert.equal(before.pending[0].tick,'60');
 send({type:'Join',session:'local',nation:'STH'});assert.equal((await wait(m=>m.type==='Notice')).key,'already-joined');
 const rejected=[];
 for(const [sequence,command] of [['1',{type:'Create',model:'missing',requested_ic_bits:'65536'}],['2',{type:'Create',model:'test_model_1',requested_ic_bits:'9223372036854775807'}],['3',{type:'SetIC',line:'99',requested_ic_bits:'0'}],['4',{type:'SetIC',line:'0',requested_ic_bits:'01'}]]){
  send({type:'ProductionCommand',sequence,command});const result=await wait(m=>m.type==='CommandResult'&&m.sequence===sequence);assert.equal(result.accepted,false);rejected.push(result);
 }
 assert.deepEqual(await query('after-rejections'),before);
 send({type:'ProductionCommand',sequence:'5',command:{type:'SetIC',line:'0',requested_ic_bits:'32768'}});assert.equal((await wait(m=>m.type==='CommandResult'&&m.sequence==='5')).accepted,true);
 const after=await query('accepted');assert.equal(after.lines[0].requested_ic.bits,'32768');assert.equal(after.lines[0].carry.bits,'16384');assert.equal(after.pending[0].tick,'60');
 send({type:'ProductionCommand',sequence:'5',command:{type:'Cancel',line:'0'}});const duplicate=await wait(m=>m.type==='CommandResult'&&m.sequence==='5'&&!m.accepted);assert.equal(duplicate.reason_key,'invalid-sequence');assert.deepEqual(await query('after-duplicate'),after);
 const closed=new Promise((resolve,reject)=>{ws.addEventListener('close',e=>e.wasClean&&e.code===1000?resolve():reject(Error('unclean close '+e.code)),{once:true});});ws.close(1000,'production-acceptance');await closed;
 console.log(JSON.stringify({pid:process.pid,welcome,snapshot,before,rejected,after,duplicate,clean_close:1000}));
}
main().catch(e=>{console.error(e);process.exit(1)});
