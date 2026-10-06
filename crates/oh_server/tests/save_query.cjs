// Native server wire test; no browser/client fixture or gameplay rules.
const assert = require('node:assert/strict');
const {encode,decode} = require(process.cwd() + '/client/node_modules/@msgpack/msgpack');
async function main() {
  const url = process.argv[2];
  const messages = [], waiters = [];
  const socket = new WebSocket(url.replace('http','ws') + 'ws');
  socket.binaryType = 'arraybuffer';
  socket.addEventListener('message', event => {
    const message = decode(new Uint8Array(event.data));
    messages.push(message);
    for (const waiter of [...waiters]) if (waiter.predicate(message)) {
      clearTimeout(waiter.timer); waiters.splice(waiters.indexOf(waiter),1); waiter.resolve(message);
    }
  });
  const wait = predicate => new Promise((resolve,reject) => {
    const seen = messages.find(predicate); if (seen) {resolve(seen);return;}
    const waiter = {predicate,resolve,timer:setTimeout(()=>reject(new Error('wire wait timeout')),5000)};
    waiters.push(waiter);
  });
  await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
  socket.send(encode({type:'Hello',protocol_version:'m0-v1'}));
  const welcome = await wait(m=>m.type==='Welcome');assert.equal(welcome.accepted,true);
  socket.send(encode({type:'Create',scenario:'m1',seed:'999',mode:'single'}));
  assert.equal((await wait(m=>m.type==='Notice')).key,'unsupported-create');
  socket.send(encode({type:'Join',session:'local',nation:null}));
  const snapshot = await wait(m=>m.type==='Snapshot');
  assert.deepEqual(snapshot.state,{date:'2000-03-01',hour:0,tick:'48',paused:true,speed:5});
  socket.send(encode({type:'Query',request:'saved-world',kind:'world'}));
  const query = await wait(m=>m.type==='WorldResult'&&m.request==='saved-world');assert.equal(query.supported,true);
  assert.equal(query.world.tick,'48');assert.equal(query.world.nations.length,2);assert.equal(query.world.provinces.length,6);
  const ledger = query.world.states[0].infrastructure;
  assert.equal(ledger.tick,'48');assert.equal(ledger.entries.length,4);
  assert.equal(ledger.entries[1].source_key,'a.raw');assert.notEqual(ledger.final_value,'3');
  assert.equal(ledger.final_value,ledger.entries[3].accumulated);
  socket.send(encode({type:'Command',sequence:'1',command:{type:'SetSpeed',speed:2}}));
  const command = await wait(m=>m.type==='CommandResult'&&m.sequence==='1');assert.equal(command.accepted,true);
  // State remains at tick48: commands pump while the restored game is paused.
  const state = await wait(m=>m.type==='Snapshot'&&m.state.speed===2);assert.equal(state.state.tick,'48');assert.equal(state.state.paused,true);
  socket.close();console.log(JSON.stringify({pid:process.pid,welcome,snapshot,query,command,state,messages}));
}
main().catch(error=>{console.error(error);process.exit(1);});
