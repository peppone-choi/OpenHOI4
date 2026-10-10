import type * as P from './proto/protocol';
type Guard<T> = (v: unknown) => v is T;
const text: Guard<string> = (v): v is string => typeof v === 'string';
const bool: Guard<boolean> = (v): v is boolean => typeof v === 'boolean';
const id: Guard<string> = (v): v is string => text(v) && /^[a-z0-9_]+$/.test(v);
const integer = (max: bigint): Guard<string> => (v): v is string => text(v) && v.length <= 20 && /^(0|[1-9]\d*)$/.test(v) && BigInt(v) <= max;
const u64 = integer(18446744073709551615n), divisionId = integer(4294967295n), count = integer(9223372036854775807n);
const number = (max: number): Guard<number> => (v): v is number => typeof v === 'number' && Number.isInteger(v) && v >= 0 && v <= max;
const u16 = number(65535), u32 = number(4294967295);
const hash: Guard<string> = (v): v is string => text(v) && /^[0-9a-f]{16}$/.test(v);
const exact = <T>(fields: {[K in keyof T]-?: Guard<T[K]>}): Guard<T> => (v): v is T => typeof v === 'object' && v !== null && !Array.isArray(v) && Reflect.ownKeys(v).length === Object.keys(fields).length && Object.entries(fields).every(([k, g]) => Object.hasOwn(v, k) && (g as Guard<unknown>)((v as Record<string, unknown>)[k]));
const array = <T>(g: Guard<T>): Guard<T[]> => (v): v is T[] => Array.isArray(v) && Array.from({length: v.length}, (_, i) => Object.hasOwn(v, i) && g(v[i])).every(Boolean);
const sorted = <T>(g: Guard<T>, less: (a: T, b: T) => boolean): Guard<T[]> => (v): v is T[] => array(g)(v) && v.every((x, i) => i === 0 || less(v[i-1], x));
const nullable = <T>(g: Guard<T>): Guard<T | null> => (v): v is T | null => v === null || g(v);
const literal = <T extends string>(s: T): Guard<T> => (v): v is T => v === s;
// Validate fixed-point transport encoding only; game calculations remain in Rust.
const fixed = (bits: 16 | 32): Guard<P.FixedValue> => {
  const shape = exact<P.FixedValue>({value: (v): v is string => text(v) && v.length <= 128 && /^(0|[1-9]\d*)(\.\d+)?$/.test(v), bits: count, fractional_bits: (v): v is number => v === bits});
  return (v): v is P.FixedValue => {
    if (!shape(v)) return false;
    const [whole, fraction = ''] = v.value.split('.'), denominator = 10n ** BigInt(fraction.length), n = BigInt(whole + fraction) * (1n << BigInt(bits));
    let raw = n / denominator; const r = n % denominator;
    if (r * 2n > denominator || (r * 2n === denominator && raw % 2n !== 0n)) raw++;
    return raw === BigInt(v.bits);
  };
};
const qty = fixed(16), fx = fixed(32);
export const normalQtyFields=['strength','soft_fire','hard_fire','defense','breakthrough','frontage','supply_use'] as const;
const boundedId:Guard<string>=(v):v is string=>id(v)&&v.length<=64;
const role:Guard<string>=(v):v is string=>v==='combat'||v==='support';
const entry=exact<P.MilitaryNormalLedgerEntry>({id:text,role,position:number(11),component:boundedId,value:qty,accumulated:qty});
const fieldShape=exact<P.MilitaryNormalLedgerField>({field:text,base:qty,value:qty,entries:array(entry)});
const ledgerShape=exact<P.MilitaryNormalLedgerView>({definitions_hash:hash,state_hash:hash,tick:u64,template:boundedId,fields:array(fieldShape)});
export const isMilitaryNormalLedgerView:Guard<P.MilitaryNormalLedgerView>=(v):v is P.MilitaryNormalLedgerView=>{
  if(!ledgerShape(v)||v.fields.length!==normalQtyFields.length)return false;
  let reference:string[]|null=null;
  return v.fields.every((field,i)=>{
    if(field.field!==normalQtyFields[i]||field.base.bits!=='0'||field.entries.length===0||field.entries.length>16)return false;
    let combat=0,support=0,inSupport=false,previousCombat='',previousSupport='';
    const ids=new Set<string>();
    for(const e of field.entries){
      if(e.role==='support')inSupport=true;
      else if(inSupport)return false;
      const position=e.role==='combat'?combat++:support++;
      if(e.position!==position||combat>12||support>4||e.id!==`${field.field}:${e.role}:${e.position}`||ids.has(e.id))return false;
      ids.add(e.id);
      const previous=e.role==='combat'?previousCombat:previousSupport;
      if(e.component<previous)return false;
      if(e.role==='combat')previousCombat=e.component;else previousSupport=e.component;
    }
    if(!combat||field.entries.at(-1)!.accumulated.bits!==field.value.bits)return false;
    const refs=field.entries.map(e=>`${e.role}:${e.position}:${e.component}`);
    if(reference&&!refs.every((r,j)=>r===reference![j]))return false;
    if(reference&&refs.length!==reference.length)return false;
    reference=refs;return true;
  });
};
const commands = {
  Train: exact<Extract<P.MilitaryCommand, {type: 'Train'}>>({type: literal('Train'), template: id}),
  Cancel: exact<Extract<P.MilitaryCommand, {type: 'Cancel'}>>({type: literal('Cancel'), job: u64}),
  Deploy: exact<Extract<P.MilitaryCommand, {type: 'Deploy'}>>({type: literal('Deploy'), job: u64, army: u64, province: u16, allow_understrength: bool}),
  SetPriority: exact<Extract<P.MilitaryCommand, {type: 'SetPriority'}>>({type: literal('SetPriority'), army: u64, priority: u16}),
} satisfies {[K in P.MilitaryCommand['type']]: Guard<Extract<P.MilitaryCommand, {type: K}>>};
export const isMilitaryCommand: Guard<P.MilitaryCommand> = (v): v is P.MilitaryCommand => Object.values(commands).some(g => g(v));
const equipment = exact<P.MilitaryEquipmentView>({model: id, count});
const equipmentList = sorted(equipment, (a, b) => a.model < b.model);
const remainder = exact<P.MilitaryRemainderView>({field: id, numerator_remainder: count});
const normal = exact<P.MilitaryNormalView>({manpower: count, equipment: equipmentList, strength: qty, soft_fire: qty, hard_fire: qty, defense: qty, breakthrough: qty, frontage: qty, supply_use: qty, organization: qty, armor: qty, piercing: qty, speed_kmh: fx, weighted_remainders: sorted(remainder, (a, b) => a.field < b.field)});
const army = exact<P.MilitaryArmyView>({id: u64, nation: u16, general: id, division_limit: u32, priority: u16, divisions: sorted(divisionId, (a, b) => BigInt(a) < BigInt(b))});
const division = exact<P.MilitaryDivisionView>({id: divisionId, nation: u16, army: u64, province: u16, template: id, normal, manpower: count, equipment: equipmentList});
const status: Guard<P.MilitaryJobStatus> = (v): v is P.MilitaryJobStatus => text(v) && ['Pending', 'Training', 'Ready', 'Cancelled', 'Deployed'].includes(v);
const days: Guard<number> = (v): v is number => u32(v) && v > 0;
const job = exact<P.MilitaryJobView>({id: u64, nation: u16, template: id, normal, training_days: days, status, progress_days: u32, start_tick: nullable(u64), reserved_manpower: count, equipment: equipmentList, division: nullable(divisionId)});
const background = exact<P.MilitaryBackgroundView>({nation: u16, committed: count, reserved: count});
const template = exact<P.MilitaryTemplateView>({template: id, training_days: days, normal});
const pending = exact<P.MilitaryPendingView>({tick: u64, nation: u16, sequence: u64, command: isMilitaryCommand});
export const isMilitaryView = exact<P.MilitaryView>({state_hash: hash, definitions_hash: hash, next_job_id: u64, next_division_id: divisionId, templates: sorted(template, (a, b) => a.template < b.template), armies: sorted(army, (a, b) => BigInt(a.id) < BigInt(b.id)), divisions: sorted(division, (a, b) => BigInt(a.id) < BigInt(b.id)), jobs: sorted(job, (a, b) => BigInt(a.id) < BigInt(b.id)), background: sorted(background, (a, b) => a.nation < b.nation), pending: sorted(pending, (a, b) => BigInt(a.tick) < BigInt(b.tick) || (a.tick === b.tick && (a.nation < b.nation || (a.nation === b.nation && BigInt(a.sequence) < BigInt(b.sequence))))) });
