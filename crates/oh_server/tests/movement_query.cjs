// Real executable loopback S1 acceptance. Launched by movement_wire.rs in 3 OS CI.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {encode, decode} = require(process.cwd() + '/client/node_modules/@msgpack/msgpack');
const [binary, root, ready, isolated] = process.argv.slice(2);
const hosts = [], clients = [], receipts = {cases: [], processes: []};
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
const watchdog = setTimeout(() => { console.error('S1 native deadline'); for (const h of hosts) h.kill('SIGKILL'); process.exitCode = 1; }, 60000);
async function start(save) {
  const probe = net.createServer(); await new Promise(resolve => probe.listen(0, '127.0.0.1', resolve));
  const port = probe.address().port; await new Promise(resolve => probe.close(resolve));
  const args = ['--port', String(port), '--pack-root', root, '--scenario', 'm2_military', ...(save ? ['--load-save', save] : [])];
  const child = spawn(binary, args, {stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true}); hosts.push(child);
  let stderr = ''; child.stderr.on('data', b => { stderr += b; }); child.stdout.resume();
  const row = {pid: child.pid, args}; receipts.processes.push(row);
  child.once('exit', (code, signal) => Object.assign(row, {code, signal}));
  child.once('error', error => { stderr += String(error); });
  const url = `http://127.0.0.1:${port}/`;
  for (let i = 0; i < 200; i++) {
    assert.equal(child.exitCode, null, stderr);
    try { if ((await fetch(url)).status === 200) return url; } catch {}
    await sleep(25);
  }
  throw Error(`S1 host readiness: ${stderr}`);
}
async function connect(url, nation, join = true) {
  const ws = new WebSocket(url.replace('http', 'ws') + 'ws'); ws.binaryType = 'arraybuffer';
  const messages = [], waiters = [];
  ws.addEventListener('message', event => {
    const message = decode(new Uint8Array(event.data)); messages.push(message);
    for (const w of [...waiters]) if (messages.slice(w.after).some(w.p)) {
      clearTimeout(w.timer); waiters.splice(waiters.indexOf(w), 1); w.resolve(messages.slice(w.after).find(w.p));
    }
  });
  function wait(p, after = 0) {
    const found = messages.slice(after).find(p); if (found) return Promise.resolve(found);
    return new Promise((resolve, reject) => {
      const w = {p, after, resolve, timer: setTimeout(() => { waiters.splice(waiters.indexOf(w), 1); reject(Error('S1 wire timeout')); }, 5000)};
      waiters.push(w);
    });
  }
  const send = message => ws.send(encode(message));
  await new Promise((resolve, reject) => { ws.addEventListener('open', resolve, {once: true}); ws.addEventListener('error', reject, {once: true}); });
  send({type: 'Hello', protocol_version: 'm0-v1'}); assert.equal((await wait(m => m.type === 'Welcome')).accepted, true);
  let sequence = 0, request = 0;
  const client = {ws, messages, wait, send,
    async command(type, command, explicit) {
      const seq = explicit ?? String(++sequence), after = messages.length;
      send({type, sequence: seq, command});
      const result = await wait(m => m.type === 'CommandResult' && m.sequence === seq, after);
      if (result.accepted) {
        const index = messages.indexOf(result);
        await wait(m => m.type === 'Snapshot', index + 1);
        assert.equal(messages[index + 1].type, 'Snapshot', 'ACK immediately precedes authoritative Snapshot');
      } else assert.equal(typeof result.reason_key, 'string');
      return result;
    },
    async result(kind) {
      const id = `s1:${++request}`; send({type: 'Query', request: id, kind}); return wait(m => m.request === id);
    },
    async query(kind) {
      const result = await client.result(kind);
      assert.equal(result.supported, true); assert.equal(result.reason_key, null); return result[kind];
    },
    async close() {
      if (ws.readyState === WebSocket.CLOSED) return;
      const closed = new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(Error('S1 close timeout')), 5000);
        ws.addEventListener('close', event => { clearTimeout(timer); event.code === 1000 && event.wasClean ? resolve() : reject(Error(`S1 close ${event.code}`)); }, {once: true});
      });
      ws.close(1000, 's1-complete'); await closed;
    }
  };
  clients.push(client);
  if (join) { send({type: 'Join', session: 'local', nation}); await wait(m => m.type === 'Snapshot'); }
  return client;
}
const move = (division, destination) => ({type: 'Move', division, destination});
const stop = division => ({type: 'Stop', division});
async function unchanged(client, operation) {
  const movement = await client.query('movement'), military = await client.query('military');
  assert.equal(movement.state_hash, military.state_hash);
  const result = await operation(); assert.equal(result.accepted, false); assert.ok(result.reason_key);
  assert.deepEqual(await client.query('movement'), movement); assert.deepEqual(await client.query('military'), military);
  receipts.cases.push({kind: 'atomic-rejection', result, state_hash: movement.state_hash});
}
async function arrival(client, division, destination) {
  const deadline = Date.now() + 15000;
  let movement;
  do {
    movement = await client.query('movement');
    const unit = movement.units.find(unit => unit.division === division);
    if (unit.province === destination && unit.route.length === 0) break;
    assert.ok(Date.now() < deadline, 'movement arrival deadline'); await sleep(25);
  } while (true);
  assert.equal((await client.command('Command', {type: 'Pause', paused: true})).accepted, true);
  movement = await client.query('movement'); const military = await client.query('military');
  const unit = movement.units.find(unit => unit.division === division);
  assert.equal(unit.province, destination); assert.equal(military.divisions.find(unit => unit.id === division).province, destination);
  assert.equal(unit.elapsed.bits, '0'); assert.equal(unit.current_leg_hours.bits, '0'); assert.equal(unit.remaining_hours.bits, '0');
  assert.equal(movement.state_hash, military.state_hash);
  receipts.cases.push({kind: 'arrival-after-snapshot', movement, military});
}
async function main() {
  try {
    const fresh = await start(), resumed = await start(ready), disconnected = await start(isolated);
    const empty = await connect(fresh, 'N01');
    assert.equal((await empty.command('Command', {type: 'Pause', paused: true})).accepted, true);
    const emptyView = await empty.query('movement'); assert.deepEqual(emptyView.units, []);
    receipts.cases.push({kind: 'empty-context-supported', movement: emptyView}); await empty.close();
    const unjoined = await connect(resumed, 'N01', false);
    const noQuery = await unjoined.result('movement');
    assert.deepEqual([noQuery.type, noQuery.supported, noQuery.reason_key, noQuery.movement], ['MovementResult', false, 'not-joined', null]);
    const noCommand = await unjoined.command('MovementCommand', stop('0')); assert.equal(noCommand.reason_key, 'not-joined'); await unjoined.close();
    receipts.cases.push({kind: 'not-joined', noQuery, noCommand});
    const observer = await connect(resumed, null);
    await unchanged(observer, () => observer.command('MovementCommand', stop('0'))); await observer.close();
    const client = await connect(resumed, 'N01');
    assert.equal((await client.command('MilitaryCommand', {type: 'Deploy', job: '0', army: '0', province: 1, allow_understrength: true})).accepted, true);
    const roster = await client.query('military'), division = roster.jobs.find(job => job.id === '0').division;
    const foreign = roster.divisions.find(unit => unit.nation === 2).id;
    for (const command of [move(foreign, 2), stop(foreign), move('4294967295', 2), move(division, 40), move(division, 65535), move('01', 2)]) {
      await unchanged(client, () => client.command('MovementCommand', command));
    }
    const accepted = await client.command('MovementCommand', move(division, 2)); assert.equal(accepted.accepted, true);
    const adjacent = await client.query('movement'); assert.deepEqual(adjacent.units.find(unit => unit.division === division).route, [2]);
    receipts.cases.push({kind: 'adjacent-move-accepted', accepted, movement: adjacent});
    await unchanged(client, () => client.command('MovementCommand', stop(division), accepted.sequence));
    await unchanged(client, () => client.command('MovementCommand', stop(division), '01'));
    await unchanged(client, () => client.command('Command', {type: 'Pause', paused: true}, accepted.sequence));
    assert.equal((await client.command('MovementCommand', stop(division))).accepted, true);
    const idle = (await client.query('movement')).units.find(unit => unit.division === division);
    assert.deepEqual(idle.route, []); assert.equal(idle.elapsed.bits, '0');
    assert.equal((await client.command('MovementCommand', move(division, 14))).accepted, true);
    const planned = (await client.query('movement')).units.find(unit => unit.division === division);
    assert.ok(planned.route.length >= 2); assert.equal(planned.route.at(-1), 14);
    assert.equal((await client.command('Command', {type: 'SetSpeed', speed: 4})).accepted, true);
    assert.equal((await client.command('Command', {type: 'Pause', paused: false})).accepted, true);
    assert.equal((await client.command('Command', {type: 'Pause', paused: true})).accepted, true);
    const moving = (await client.query('movement')).units.find(unit => unit.division === division);
    assert.ok(BigInt(moving.elapsed.bits) > 0n); assert.ok(BigInt(moving.elapsed.bits) <= BigInt(moving.current_leg_hours.bits));
    assert.equal(BigInt(moving.remaining_hours.bits), BigInt(planned.remaining_hours.bits) - BigInt(moving.elapsed.bits));
    assert.equal((await client.command('MovementCommand', stop(division))).accepted, true);
    const stopped = (await client.query('movement')).units.find(unit => unit.division === division);
    assert.deepEqual(stopped.route, [moving.route[0]]); assert.deepEqual(stopped.elapsed, moving.elapsed);
    assert.equal(BigInt(stopped.remaining_hours.bits), BigInt(stopped.current_leg_hours.bits) - BigInt(stopped.elapsed.bits));
    receipts.cases.push({kind: 'stop-retains-current-leg-only', planned, moving, stopped});
    assert.equal((await client.command('Command', {type: 'Pause', paused: false})).accepted, true);
    await arrival(client, division, stopped.route[0]);
    assert.equal((await client.command('MovementCommand', move(division, 14))).accepted, true);
    assert.equal((await client.command('Command', {type: 'Pause', paused: false})).accepted, true);
    await arrival(client, division, 14); await client.close();
    const isolatedClient = await connect(disconnected, 'N01');
    const isolatedId = (await isolatedClient.query('military')).divisions.find(unit => unit.nation === 1).id;
    await unchanged(isolatedClient, () => isolatedClient.command('MovementCommand', move(isolatedId, 110))); await isolatedClient.close();
  } finally {
    const closing = await Promise.allSettled(clients.map(client => client.close()));
    for (const child of hosts) {
      if (child.exitCode === null && child.signalCode === null) {
        const exited = new Promise(resolve => child.once('exit', resolve));
        // Windows uses explicit child termination; graceful Ctrl+C is covered by lifecycle.py.
        child.kill(process.platform === 'win32' ? 'SIGTERM' : 'SIGINT');
        const timer = setTimeout(() => child.kill('SIGKILL'), 5000);
        try { await exited; } finally { clearTimeout(timer); }
      }
      const row = receipts.processes.find(row => row.pid === child.pid);
      row.absent = false; try { process.kill(child.pid, 0); } catch (error) { if (error.code === 'ESRCH') row.absent = true; else throw error; }
      if (process.platform === 'win32') row.cleanup = 'owned-process termination, not a graceful lifecycle assertion';
    }
    clearTimeout(watchdog);
    const output = path.resolve('target/wp17', process.platform); fs.mkdirSync(output, {recursive: true});
    fs.writeFileSync(path.join(output, 'movement-wire.json'), JSON.stringify(receipts, null, 2));
    for (const row of receipts.processes) {
      assert.equal(row.absent, true, 'owned native process is gone');
      if (process.platform !== 'win32') assert.equal(row.code, 0, 'normal SIGINT shutdown');
    }
    for (const result of closing) if (result.status === 'rejected') throw result.reason;
  }
  console.log(JSON.stringify({S1: 'PASS', cases: receipts.cases.length, processes: receipts.processes}));
}
main().catch(error => { console.error(error); process.exitCode = 1; });
