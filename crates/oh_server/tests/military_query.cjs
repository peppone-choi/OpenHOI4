// Actual paused V7 host: resource ownership, explicit deploy, integer rejection, and clean close.
const assert=require('node:assert/strict'),fs=require('node:fs');
const {encode,decode}=require(process.cwd()+'/client/node_modules/@msgpack/msgpack');
async function main(){
 const url=process.argv[2],expected=JSON.parse(fs.readFileSync(process.argv[3],'utf8')),messages=[],waiters=[];
 const template=expected.template??'example', held=expected.held_equipment??'0';
 const ws=new WebSocket(url.replace('http','ws')+'ws');ws.binaryType='arraybuffer';
 ws.addEventListener('message',event=>{const m=decode(new Uint8Array(event.data));messages.push(m);for(const w of [...waiters])if(w.p(m)){clearTimeout(w.timer);waiters.splice(waiters.indexOf(w),1);w.resolve(m);}});
 const wait=p=>new Promise((resolve,reject)=>{const m=messages.find(p);if(m){resolve(m);return;}const w={p,resolve,timer:setTimeout(()=>reject(Error('military wire timeout')),5000)};waiters.push(w);});
 const send=m=>ws.send(encode(m));await new Promise((resolve,reject)=>{ws.addEventListener('open',resolve,{once:true});ws.addEventListener('error',reject,{once:true});});
 send({type:'Hello',protocol_version:'m0-v1'});assert.equal((await wait(m=>m.type==='Welcome')).accepted,true);
 send({type:'Query',request:'not-joined',kind:'military'});const notJoined=await wait(m=>m.type==='MilitaryResult'&&m.request==='not-joined');assert.equal(notJoined.supported,false);assert.equal(notJoined.reason_key,'not-joined');assert.equal(notJoined.military,null);
 send({type:'Join',session:'local',nation:'NTH'});const snapshot=await wait(m=>m.type==='Snapshot');assert.equal(snapshot.state.tick,expected.tick);assert.equal(snapshot.state.paused,true);
 const query=async request=>{send({type:'Query',request,kind:'military'});const m=await wait(m=>m.type==='MilitaryResult'&&m.request===request);assert.equal(m.supported,true);assert.equal(m.reason_key,null);return m.military;};
 const before=await query('before');assert.equal(before.state_hash,expected.hash);assert.equal(before.jobs[0].status,'Ready');assert.equal(before.jobs[0].reserved_manpower,'8');assert.equal(before.jobs[0].start_tick,'24');assert.equal(before.jobs[0].progress_days,2);assert.deepEqual(before.jobs[0].equipment,[{model:'test_model_1',count:held}]);assert.equal(before.jobs[0].template,template);assert.equal(before.jobs[0].normal.manpower,'8');assert.equal(before.jobs[0].normal.equipment[0].count,expected.normal_equipment??'8');assert.equal(before.divisions.length,0);assert.equal(before.background[0].reserved,'10');
 const rejected=[];
 for(const [sequence,command] of [
  ['1',{type:'Cancel',job:'-9223372036854775808'}],
  ['2',{type:'Cancel',job:'00'}],
  ['3',{type:'Deploy',job:'0',army:'1',province:10,allow_understrength:true}],
  ['4',{type:'Deploy',job:'0',army:'0',province:20,allow_understrength:true}],
  ['5',{type:'Deploy',job:'0',army:'0',province:50,allow_understrength:true}],
  ['6',{type:'Deploy',job:'0',army:'0',province:10,allow_understrength:false}],
  ['7',{type:'Train',template:'absent'}],
  ['8',{type:'SetPriority',army:'18446744073709551616',priority:0}],
 ]) {send({type:'MilitaryCommand',sequence,command});const r=await wait(m=>m.type==='CommandResult'&&m.sequence===sequence);assert.equal(r.accepted,false);rejected.push(r);}
 assert.deepEqual(await query('after-rejection'),before);
 send({type:'MilitaryCommand',sequence:'9',command:{type:'Deploy',job:'0',army:'0',province:10,allow_understrength:true}});assert.equal((await wait(m=>m.type==='CommandResult'&&m.sequence==='9')).accepted,true);
 const deployed=await query('deployed');assert.equal(deployed.jobs[0].status,'Deployed');assert.equal(deployed.jobs[0].reserved_manpower,'0');assert.equal(deployed.jobs[0].division,'0');assert.equal(deployed.divisions[0].manpower,'8');assert.equal(deployed.divisions[0].province,10);assert.deepEqual(deployed.armies[0].divisions,['0']);assert.deepEqual(deployed.jobs[0].equipment,[]);assert.deepEqual(deployed.divisions[0].equipment,[{model:'test_model_1',count:held}]);
 send({type:'MilitaryCommand',sequence:'10',command:{type:'Cancel',job:'0'}});assert.equal((await wait(m=>m.type==='CommandResult'&&m.sequence==='10')).accepted,false);assert.deepEqual(await query('after-terminal'),deployed);
 send({type:'MilitaryCommand',sequence:'11',command:{type:'Train',template}});assert.equal((await wait(m=>m.type==='CommandResult'&&m.sequence==='11')).accepted,true);
 const pending=await query('pending');assert.equal(pending.jobs[1].status,'Pending');assert.equal(pending.jobs[1].reserved_manpower,'0');assert.equal(pending.jobs[1].progress_days,0);
 send({type:'MilitaryCommand',sequence:'12',command:{type:'Cancel',job:'1'}});assert.equal((await wait(m=>m.type==='CommandResult'&&m.sequence==='12')).accepted,true);
 const cancelled=await query('cancelled');assert.equal(cancelled.jobs[1].status,'Cancelled');assert.equal(cancelled.jobs[1].reserved_manpower,'0');assert.equal(cancelled.jobs[1].progress_days,0);
 send({type:'MilitaryCommand',sequence:'12',command:{type:'SetPriority',army:'0',priority:3}});const duplicate=await wait(m=>m.type==='CommandResult'&&m.sequence==='12'&&!m.accepted);assert.equal(duplicate.reason_key,'invalid-sequence');assert.deepEqual(await query('after-duplicate'),cancelled);
 const closed=new Promise((resolve,reject)=>{ws.addEventListener('close',e=>e.wasClean&&e.code===1000?resolve():reject(Error('unclean close '+e.code)),{once:true});});ws.close(1000,'military-acceptance');await closed;
 console.log(JSON.stringify({pid:process.pid,notJoined,snapshot,before,rejected,deployed,pending,cancelled,duplicate,clean_close:1000}));
}
main().catch(e=>{console.error(e);process.exit(1)});
