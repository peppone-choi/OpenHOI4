// Actual separate-process acceptance. Each socket has its own simulation.
// Usage from repo root: node <this file> <loopback HTTP URL> <expected.json> <fresh|restored>
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { encode, decode } = require(process.cwd() + '/client/node_modules/@msgpack/msgpack');

async function country(url, expected, mode, nation) {
  const messages = [], waiters = [];
  const ws = new WebSocket(new URL('ws', url).href.replace('http:', 'ws:'));
  ws.binaryType = 'arraybuffer';
  ws.addEventListener('message', event => {
    const message = decode(new Uint8Array(event.data));
    messages.push(message);
    for (const waiter of [...waiters]) if (waiter.predicate(message)) {
      clearTimeout(waiter.timer);
      waiters.splice(waiters.indexOf(waiter), 1);
      waiter.resolve(message);
    }
  });
  const wait = predicate => new Promise((resolve, reject) => {
    const known = messages.find(predicate);
    if (known) return resolve(known);
    const waiter = { predicate, resolve, timer: setTimeout(() => {
      waiters.splice(waiters.indexOf(waiter), 1);
      reject(Error(`N${nation}: wire timeout`));
    }, 5000) };
    waiters.push(waiter);
  });
  const send = message => ws.send(encode(message));
  try {
    await new Promise((resolve, reject) => {
      ws.addEventListener('open', resolve, { once: true });
      ws.addEventListener('error', reject, { once: true });
    });
    send({ type: 'Hello', protocol_version: 'm0-v1' });
    const welcome = await wait(message => message.type === 'Welcome');
    assert.equal(welcome.accepted, true);
    assert.equal(welcome.packs[0].id, 'testland_m2_production');
    send({ type: 'Join', session: 'local', nation: 'N' + String(nation).padStart(2, '0') });
    const snapshot = await wait(message => message.type === 'Snapshot');
    let sequence = 0;
    const command = async (type, action, accepted) => {
      const id = String(++sequence);
      send({ type, sequence: id, command: action });
      const result = await wait(message => message.type === 'CommandResult' && message.sequence === id);
      assert.equal(result.accepted, accepted);
      return result;
    };
    if (mode === 'restored') {
      assert.equal(snapshot.state.tick, expected.tick);
      assert.equal(snapshot.state.paused, true);
    } else {
      await command('Command', { type: 'Pause', paused: true }, true);
    }
    const query = async request => {
      send({ type: 'Query', request, kind: 'production' });
      const result = await wait(message => message.type === 'ProductionResult' && message.request === request);
      assert.equal(result.supported, true);
      return result.production;
    };
    let before = await query('initial');
    assert.equal(before.nations.length, 6);
    assert.equal(before.models.length, 2);
    const own = before.nations.find(row => row.nation === nation);
    assert.ok(own.allowed_models.length > 0);
    assert.ok(BigInt(own.military_ic.bits) > 0n);
    for (const model of before.models) {
      const recipe = model.model === 'm2_equipment_1' ? 65536n : 131072n;
      assert.equal(BigInt(model.unit_cost.bits), recipe);
      assert.equal(BigInt(model.resources.find(row => row.resource === 'steel').per_item.bits), recipe);
    }
    if (mode === 'fresh') {
      assert.equal(before.lines.length, 0);
      await command('ProductionCommand', { type: 'Create', model: own.allowed_models[0], requested_ic_bits: own.military_ic.bits }, true);
      before = await query('created');
      assert.equal(before.lines.length, 1);
      assert.equal(before.lines[0].nation, nation);
      assert.equal(before.lines[0].requested_ic.bits, own.military_ic.bits);
    } else {
      assert.equal(before.state_hash, BigInt(expected.paused_hash).toString(16).padStart(16, '0'));
      assert.equal(before.lines.length, 6);
      assert.equal(before.next_line_id, '6');
      for (const line of before.lines) {
        const stored = expected.production.lines[line.id];
        assert.equal(line.nation, stored.nation);
        assert.equal(line.model, stored.model);
        assert.equal(line.requested_ic.bits, String(stored.requested_ic.bits));
        assert.equal(line.efficiency.bits, String(stored.efficiency.bits));
        assert.equal(line.carry.bits, String(stored.carry.bits));
      }
      for (const row of before.nations) for (const stock of row.stock) {
        assert.equal(stock.available, String(expected.production.stock[row.nation][stock.model]));
      }
      for (const row of before.day.nations) {
        const steel = row.flows.find(flow => flow.resource === 'steel');
        assert.ok(BigInt(steel.debited_raw) > 0n);
        assert.ok(BigInt(steel.debited_raw) <= BigInt(steel.flow) * 65536n);
      }
    }
    const rejected = [];
    rejected.push(await command('ProductionCommand', { type: 'Create', model: 'missing', requested_ic_bits: '0' }, false));
    rejected.push(await command('ProductionCommand', { type: 'Create', model: own.allowed_models[0], requested_ic_bits: '9223372036854775807' }, false));
    const disallowed = before.models.find(model => !own.allowed_models.includes(model.model));
    if (disallowed) rejected.push(await command('ProductionCommand', { type: 'Create', model: disallowed.model, requested_ic_bits: '0' }, false));
    if (mode === 'restored') {
      const foreign = before.lines.find(line => line.nation !== nation);
      rejected.push(await command('ProductionCommand', { type: 'SetIC', line: foreign.id, requested_ic_bits: '0' }, false));
    }
    assert.deepEqual(await query('after-rejections'), before);
    const ownLine = before.lines.find(line => line.nation === nation);
    const lowered = (BigInt(ownLine.requested_ic.bits) / 2n).toString();
    const accepted = await command('ProductionCommand', { type: 'SetIC', line: ownLine.id, requested_ic_bits: lowered }, true);
    const after = await query('accepted');
    assert.equal(after.lines.find(line => line.id === ownLine.id).requested_ic.bits, lowered);
    assert.notEqual(after.state_hash, before.state_hash);
    send({ type: 'ProductionCommand', sequence: accepted.sequence, command: { type: 'Cancel', line: ownLine.id } });
    const duplicate = await wait(message => message.type === 'CommandResult' && message.sequence === accepted.sequence && !message.accepted);
    assert.equal(duplicate.reason_key, 'invalid-sequence');
    assert.deepEqual(await query('after-duplicate'), after);
    const closed = new Promise((resolve, reject) => ws.addEventListener('close', event => {
      if (event.wasClean && event.code === 1000) resolve(); else reject(Error('unclean close ' + event.code));
    }, { once: true }));
    ws.close(1000, 'o2-acceptance');
    await closed;
    return { nation, mode, before_hash: before.state_hash, after_hash: after.state_hash, rejected: rejected.length, clean_close: 1000 };
  } finally {
    for (const waiter of waiters) clearTimeout(waiter.timer);
    if (ws.readyState === WebSocket.OPEN) ws.close(1000, 'acceptance-exit');
  }
}

async function main() {
  const [url, file, mode] = process.argv.slice(2);
  const address = new URL(url);
  assert.equal(address.hostname, '127.0.0.1');
  assert.ok(['fresh', 'restored'].includes(mode));
  const expected = JSON.parse(fs.readFileSync(file, 'utf8'));
  const response = await fetch(url);
  assert.equal(response.status, 200);
  const html = await response.text();
  const bundle = html.match(/src="([^"]+\.js)"/)[1];
  const served = Buffer.from(await (await fetch(new URL(bundle, url))).arrayBuffer());
  assert.deepEqual(served, fs.readFileSync(process.cwd() + '/client/dist' + bundle));
  const countries = [];
  for (let nation = 1; nation <= 6; nation++) countries.push(await country(url, expected, mode, nation));
  console.log(JSON.stringify({ pid: process.pid, mode, served_bundle_matches_source_build: true, countries }));
}
main().catch(error => { console.error(error); process.exitCode = 1; });
