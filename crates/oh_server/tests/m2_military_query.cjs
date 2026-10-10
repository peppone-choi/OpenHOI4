// Mandatory separate-process loopback acceptance; not part of cargo/CI by default.
// Run from repo root after client build + server build and native checkpoint export.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {encode, decode} = require(process.cwd() + '/client/node_modules/@msgpack/msgpack');
const fixture = path.resolve(process.env.OH_M2_MILITARY_OUTPUT_DIR || 'target/wp23/checkpoint');
const output = path.resolve(process.env.OH_M2_MILITARY_WIRE_OUTPUT || 'target/wp23/wire');
const binary = path.resolve(process.env.OH_SERVER_EXECUTABLE || 'target/debug/oh_server');
const records = [], sockets = [], hosts = [];
fs.mkdirSync(output, {recursive:true});
const ledger = () => fs.writeFileSync(path.join(output,'pid-ledger.json'), JSON.stringify({owner:process.pid,records},null,2));
const sleep = ms => new Promise(resolve => setTimeout(resolve,ms));
async function freePort() {
  const server = net.createServer();await new Promise(resolve => server.listen(0,'127.0.0.1',resolve));
  const port = server.address().port;await new Promise(resolve => server.close(resolve));return port;
}
async function start(root, save) {
  const port = await freePort(), args = ['--port',String(port),'--pack-root',root,'--scenario','m2_military',...(save?['--load-save',save]:[])];
  const child = spawn(binary,args,{cwd:process.cwd(),stdio:['ignore','pipe','pipe']});hosts.push(child);
  const row = {pid:child.pid,args,exitCode:null,signal:null,alive:true};records.push(row);ledger();
  child.stdout.pipe(fs.createWriteStream(path.join(output,`server-${port}.stdout`)));
  child.stderr.pipe(fs.createWriteStream(path.join(output,`server-${port}.stderr`)));
  child.on('exit',(code,signal)=>{row.exitCode=code;row.signal=signal;row.alive=false;ledger();});
  const url = `http://127.0.0.1:${port}/`;
  for(let attempt=0;attempt<100;attempt++) {if(child.exitCode!==null)throw Error('host exited');try{if((await fetch(url)).status===200)return url;}catch{}await sleep(50);}
  throw Error('host readiness timeout');
}
async function connect(url,nation) {
  const ws = new WebSocket(url.replace('http','ws')+'ws'), messages = [], waiters=[];sockets.push(ws);ws.binaryType='arraybuffer';
  ws.addEventListener('message',event=>{const m=decode(new Uint8Array(event.data));messages.push(m);for(const w of [...waiters])if(w.p(m)){clearTimeout(w.timer);waiters.splice(waiters.indexOf(w),1);w.resolve(m);}});
  const wait=p=>new Promise((resolve,reject)=>{const found=messages.find(p);if(found){resolve(found);return;}const w={p,resolve,timer:setTimeout(()=>{waiters.splice(waiters.indexOf(w),1);reject(Error('wire timeout'));},5000)};waiters.push(w);});
  const send=m=>ws.send(encode(m));await new Promise((resolve,reject)=>{ws.addEventListener('open',resolve,{once:true});ws.addEventListener('error',reject,{once:true});});
  send({type:'Hello',protocol_version:'m0-v1'});assert.equal((await wait(m=>m.type==='Welcome')).accepted,true);
  send({type:'Join',session:'local',nation:`N${String(nation).padStart(2,'0')}`});const snapshot=await wait(m=>m.type==='Snapshot');
  let request=0,sequence=0;
  const query=async kind=>{const id=`wp23:${kind}:${++request}`;send({type:'Query',request:id,kind});const result=await wait(m=>m.request===id);assert.equal(result.supported,true);assert.equal(result.reason_key,null);return result[kind];};
  const command=async(type,command,explicit)=>{const seq=explicit??String(++sequence);send({type,sequence:seq,command});return wait(m=>m.type==='CommandResult'&&m.sequence===seq&&(explicit===undefined||!m.accepted));};
  const close=async()=>{if(ws.readyState===WebSocket.CLOSED)return;await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error('close timeout')),5000);ws.addEventListener('close',e=>{clearTimeout(timer);e.wasClean&&e.code===1000?resolve():reject(Error(`unclean close ${e.code}`));},{once:true});ws.close(1000,'wp23-acceptance');});};
  return {snapshot,query,command,close,messages};
}
const economyNation=(e,n)=>e.nations.find(row=>row.nation===n);
const stock=(p,n)=>BigInt(p.nations.find(row=>row.nation===n).stock.find(row=>row.model==='m2_equipment_1').available);
async function servedIdentity(url){const html=await(await fetch(url)).text(),match=html.match(/<script[^>]*src="([^"]+)"/);assert.ok(match);const bytes=Buffer.from(await(await fetch(new URL(match[1],url))).arrayBuffer());assert.deepEqual(bytes,fs.readFileSync(path.join(process.cwd(),'client/dist',match[1])));return {script:match[1],bytes:bytes.length};}
async function main(){
  const root=fs.mkdtempSync(path.join(os.tmpdir(),'openhoi-wp23-wire-'));fs.cpSync('data/packs/testland_m2_military',path.join(root,'testland'),{recursive:true});
  const expected=JSON.parse(fs.readFileSync(path.join(fixture,'expected.json'),'utf8'));
  try{
    const fresh=await start(root), restored=await start(root,path.join(fixture,'paused-training.ohsave'));
    const receipts={fresh:[],restored:[],served:[await servedIdentity(fresh),await servedIdentity(restored)]};
    for(let n=1;n<=6;n++){
      const client=await connect(fresh,n);assert.equal((await client.command('Command',{type:'Pause',paused:true})).accepted,true);
      const m=await client.query('military'), p=await client.query('production');assert.equal(m.templates.length,1);assert.equal(m.templates[0].template,'m2_small');assert.equal(m.templates[0].normal.manpower,'8');assert.deepEqual(m.templates[0].normal.equipment,[{model:'m2_equipment_1',count:'6'}]);assert.equal(m.templates[0].training_days,2);
      assert.equal(m.armies.length,6);assert.deepEqual(m.armies.map(a=>a.nation),[1,2,3,4,5,6]);for(const a of m.armies){assert.equal(a.division_limit,2);assert.deepEqual(a.divisions,[]);assert.equal(a.priority,0);}
      assert.deepEqual(m.jobs,[]);assert.deepEqual(m.divisions,[]);assert.equal(m.next_job_id,'0');assert.equal(m.next_division_id,'0');for(let i=1;i<=6;i++)assert.equal(stock(p,i),0n);
      receipts.fresh.push({nation:n,snapshot:client.snapshot,military:m,production:p,clean_close:1000});await client.close();
    }
    for(let n=1;n<=6;n++){
      // Each Join owns a separate simulation; mutations never contaminate another socket.
      const client=await connect(restored,n);assert.equal(client.snapshot.state.tick,expected.tick);assert.equal(client.snapshot.state.paused,true);
      const before=await client.query('military'), production=await client.query('production'), economy=await client.query('economy');assert.equal(before.state_hash,expected.paused_hash);assert.equal(before.jobs.length,6);assert.equal(before.next_job_id,'6');assert.deepEqual(before.divisions,[]);
      for(const j of before.jobs){assert.equal(j.status,'Training');assert.equal(j.reserved_manpower,'8');assert.deepEqual(j.equipment,[{model:'m2_equipment_1',count:'6'}]);}
      const rejected=[];
      for(const command of [{type:'Cancel',job:String(n%6)},{type:'Train',template:'missing'}]){const r=await client.command('MilitaryCommand',command);assert.equal(r.accepted,false);rejected.push(r);}
      assert.deepEqual(await client.query('military'),before);assert.deepEqual(await client.query('production'),production);assert.deepEqual(await client.query('economy'),economy);
      const accepted=await client.command('MilitaryCommand',{type:'Cancel',job:String(n-1)});assert.equal(accepted.accepted,true);
      const after=await client.query('military'), pafter=await client.query('production'), eafter=await client.query('economy');const job=after.jobs[n-1];assert.equal(job.status,'Cancelled');assert.equal(job.reserved_manpower,'0');assert.deepEqual(job.equipment,[]);assert.equal(job.progress_days,0);assert.equal(job.start_tick,null);assert.equal(after.next_job_id,'6');assert.deepEqual(after.pending,[]);assert.deepEqual(after.divisions,[]);assert.equal(stock(pafter,n),stock(production,n)+6n);assert.deepEqual(pafter.lines,production.lines);
      const e=economyNation(economy,n), a=economyNation(eafter,n);assert.equal(BigInt(a.reserved),BigInt(e.reserved)-8n);assert.equal(BigInt(a.available),BigInt(e.available)+8n);assert.equal(a.committed,e.committed);for(let other=1;other<=6;other++)if(other!==n){assert.deepEqual(after.jobs[other-1],before.jobs[other-1]);assert.deepEqual(economyNation(eafter,other),economyNation(economy,other));assert.equal(stock(pafter,other),stock(production,other));}
      const duplicate=await client.command('MilitaryCommand',{type:'Cancel',job:String(n-1)},accepted.sequence);assert.equal(duplicate.accepted,false);assert.equal(duplicate.reason_key,'invalid-sequence');assert.deepEqual(await client.query('military'),after);assert.deepEqual(await client.query('production'),pafter);assert.deepEqual(await client.query('economy'),eafter);
      const terminal=await client.command('MilitaryCommand',{type:'Cancel',job:String(n-1)});assert.equal(terminal.accepted,false);assert.deepEqual(await client.query('military'),after);assert.deepEqual(await client.query('production'),pafter);assert.deepEqual(await client.query('economy'),eafter);
      receipts.restored.push({nation:n,before,production,economy,rejected,accepted,after,pafter,eafter,duplicate,terminal,clean_close:1000});await client.close();
    }
    fs.writeFileSync(path.join(output,'receipts.json'),JSON.stringify(receipts,null,2));console.log('WP23 loopback: six fresh + six restored independent sessions, conservation, rejections, served JS, close1000 PASS');
  }finally{
    for(const ws of sockets)if(ws.readyState===WebSocket.OPEN){const closed=new Promise(resolve=>ws.addEventListener('close',resolve,{once:true}));ws.close(1000,'finally');await closed;}
    for(const child of hosts)if(child.exitCode===null&&child.signalCode===null){const exited=new Promise(resolve=>child.once('exit',resolve));child.kill('SIGINT');const timer=setTimeout(()=>child.kill('SIGKILL'),5000);try{await exited;}finally{clearTimeout(timer);}}
    for(const row of records){try{process.kill(row.pid,0);row.alive=true;}catch{row.alive=false;}assert.equal(row.alive,false);assert.equal(row.exitCode,0,'forced cleanup is not PASS');}ledger();fs.rmSync(root,{recursive:true});
  }
}
main().catch(error=>{console.error(error);process.exitCode=1;});
