const assert=require('node:assert/strict');
const {encode,decode}=require(process.cwd()+'/client/node_modules/@msgpack/msgpack');
async function main(){
  const url=process.argv[2],messages=[],waiters=[];const ws=new WebSocket(url.replace('http','ws')+'ws');ws.binaryType='arraybuffer';
  ws.addEventListener('message',event=>{const m=decode(new Uint8Array(event.data));messages.push(m);for(const w of [...waiters])if(w.p(m)){clearTimeout(w.timer);waiters.splice(waiters.indexOf(w),1);w.resolve(m);}});
  const wait=p=>new Promise((resolve,reject)=>{const m=messages.find(p);if(m){resolve(m);return;}const w={p,resolve,timer:setTimeout(()=>reject(new Error('economic wire timeout')),5000)};waiters.push(w);});
  const send=m=>ws.send(encode(m));await new Promise((resolve,reject)=>{ws.addEventListener('open',resolve,{once:true});ws.addEventListener('error',reject,{once:true});});
  send({type:'Hello',protocol_version:'m0-v1'});const welcome=await wait(m=>m.type==='Welcome');assert.equal(welcome.accepted,true);
  send({type:'Join',session:'local',nation:'NTH'});const snapshot=await wait(m=>m.type==='Snapshot');assert.equal(snapshot.state.tick,'24');assert.equal(snapshot.state.paused,true);
  send({type:'Query',request:'before',kind:'economy'});const before=await wait(m=>m.type==='EconomyResult'&&m.request==='before');assert.equal(before.supported,true);assert.equal(before.economy.pending.length,2);
  send({type:'EconomyCommand',sequence:'1',command:{type:'Reorder',projects:['91']}});const accepted=await wait(m=>m.type==='CommandResult'&&m.sequence==='1');assert.equal(accepted.accepted,true);
  const rejected=[];for(const [sequence,command] of [['2',{type:'Allocate',ratios_bits:['0','0','0','0']}],['3',{type:'Construct',project:'99',state:2,building:'industry'}],['4',{type:'ChangeLaw',law:'war'}]]){send({type:'EconomyCommand',sequence,command});const result=await wait(m=>m.type==='CommandResult'&&m.sequence===sequence);assert.equal(result.accepted,false);rejected.push(result);}
  send({type:'Query',request:'after',kind:'economy'});const after=await wait(m=>m.type==='EconomyResult'&&m.request==='after');assert.deepEqual(after.economy,before.economy);ws.close();console.log(JSON.stringify({pid:process.pid,welcome,snapshot,before,accepted,rejected,after,messages}));
}
main().catch(e=>{console.error(e);process.exit(1);});
