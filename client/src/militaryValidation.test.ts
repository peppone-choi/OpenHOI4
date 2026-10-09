import {readFileSync} from 'node:fs';
import {describe, it, expect} from 'vitest';
import {decode} from '@msgpack/msgpack';
import {isServerMessage} from './network';
import {isMilitaryCommand} from './militaryValidation';
const fixture = JSON.parse(readFileSync(new URL('../../target/wp16/military-wire-fixture.json', import.meta.url), 'utf8'));
describe('actual military authority transport', () => {
  it('decodes and validates actual Rust MessagePack bytes in the client', () => {
    const wire = JSON.parse(readFileSync(new URL('../../target/wp16/military-wire-bytes.json', import.meta.url), 'utf8'));
    const message = decode(Uint8Array.from(wire.bytes));
    expect(message).toEqual(wire.expected);
    expect(isServerMessage(message)).toBe(true);
  });
  it('accepts the real Rust Ready projection and pending authority', () => {
    expect(isServerMessage(fixture)).toBe(true);
    expect(fixture.military.jobs[0].reserved_manpower).toBe('8');
    expect(fixture.military.jobs[0].status).toBe('Ready');
    expect(fixture.military.jobs[0].start_tick).toBe('24');
    expect(fixture.military.jobs[0].normal.equipment[0].count).toBe('8');
    expect(fixture.military.pending[0].command.type).toBe('SetPriority');
    expect(isServerMessage({type: 'MilitaryResult', request: 'none', supported: false, reason_key: 'unsupported-query', military: null})).toBe(true);
  });
  it('rejects noncanonical or widened integers and malformed authority bodies', () => {
    const reject = (change: (v: any) => void) => {const v = structuredClone(fixture); change(v); expect(isServerMessage(v)).toBe(false);};
    reject(v => v.military.jobs[0].reserved_manpower = '-9223372036854775808');
    reject(v => v.military.jobs[0].id = '00');
    reject(v => v.military.armies[0].id = '18446744073709551616');
    reject(v => v.military.next_division_id = '4294967296');
    reject(v => v.military.jobs[0].equipment[0].count = 0);
    reject(v => v.military.jobs[0].status = 'Complete');
    reject(v => v.military.jobs[0].normal.speed_kmh.fractional_bits = 16);
    reject(v => v.military.jobs[0].normal.strength.bits = '01');
    reject(v => v.military.jobs[0].normal.strength.value = '999');
    reject(v => v.military.jobs.push(v.military.jobs[0]));
    reject(v => v.military.armies.reverse());
    reject(v => v.military.pending[0].command.extra = true);
    reject(v => delete v.military.jobs[0].start_tick);
    reject(v => v.military.extra = true);
    reject(v => v.supported = false);
    reject(v => v.reason_key = 'wrong');
  });
  it('requires explicit deployment and rejects malformed IDs with valid extrema controls', () => {
    expect(isMilitaryCommand({type: 'Cancel', job: '18446744073709551615'})).toBe(true);
    expect(isMilitaryCommand({type: 'Deploy', job: '0', army: '0', province: 65535, allow_understrength: true})).toBe(true);
    for (const id of ['-9223372036854775808', '', '00', '+1', '1.0', '18446744073709551616']) {
      expect(isMilitaryCommand({type: 'Cancel', job: id})).toBe(false);
      expect(isMilitaryCommand({type: 'SetPriority', army: id, priority: 0})).toBe(false);
    }
    expect(isMilitaryCommand({type: 'Deploy', job: '0', army: '0', province: 10})).toBe(false);
    expect(isMilitaryCommand({type: 'SetPriority', army: '0', priority: -32768})).toBe(false);
    expect(isMilitaryCommand({type: 'Train', template: 'example', nation: 2})).toBe(false);
  });
});
