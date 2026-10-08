import {readFileSync} from 'node:fs';
import {describe,it,expect} from 'vitest';
import {isServerMessage} from './network';
import {isProductionCommand,quantityBits} from './productionValidation';
const fixture=JSON.parse(readFileSync(new URL('../../target/wp15/production-wire-fixture.json',import.meta.url),'utf8'));
describe('production authority transport',()=>{
  it('accepts a real Rust daily projection with raw carry and efficiency',()=>{
    expect(isServerMessage(fixture)).toBe(true);
    expect(fixture.production.lines[0].carry.bits).toBe('16384');
    expect(fixture.production.lines[0].efficiency.bits).toBe('1107296256');
  });
  it('rejects malformed, duplicate, widened or missing authority fields',()=>{
    const reject=(change:(v:any)=>void)=>{const v=structuredClone(fixture);change(v);expect(isServerMessage(v)).toBe(false);};
    reject(v=>delete v.production.lines[0].carry);reject(v=>v.production.lines[0].carry.bits='01');
    reject(v=>v.production.lines[0].efficiency.fractional_bits=16);reject(v=>v.production.lines.push(v.production.lines[0]));
    reject(v=>v.production.nations[0].stock[0].available=1);reject(v=>v.production.nations[0].stock[0].available='9223372036854775808');
    reject(v=>v.production.lines[0].carry.value='NaN');reject(v=>v.production.extra=true);reject(v=>v.supported=false);
    reject(v=>v.production.day.nations[0].flows[0].unused_raw='170141183460469231731687303715884105728');
  });
  it('encodes decimal IC without a production formula and rejects unsafe commands',()=>{
    expect(quantityBits('0.25')).toBe('16384');expect(quantityBits('1.5')).toBe('98304');
    for(const s of ['-1','NaN','1e5','01','999999999999999999999'])expect(quantityBits(s)).toBeNull();
    expect(isProductionCommand({type:'Create',model:'test_model_1',requested_ic_bits:'65536'})).toBe(true);
    expect(isProductionCommand({type:'Cancel',line:'0'})).toBe(true);
    expect(isProductionCommand({type:'Cancel',line:'18446744073709551616'})).toBe(false);
    expect(isProductionCommand({type:'Pause',line:'0',paused:'false'})).toBe(false);
  });
});
