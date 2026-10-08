import type * as P from './proto/protocol';
type Guard<T>=(v:unknown)=>v is T;
const text:Guard<string>=(v):v is string=>typeof v==='string';
const bool:Guard<boolean>=(v):v is boolean=>typeof v==='boolean';
const id:Guard<string>=(v):v is string=>text(v)&&/^[a-z0-9_]+$/.test(v);
const u16:Guard<number>=(v):v is number=>typeof v==='number'&&Number.isInteger(v)&&v>=0&&v<=65535;
const u32:Guard<number>=(v):v is number=>typeof v==='number'&&Number.isInteger(v)&&v>=0&&v<=4294967295;
const integer=(max:bigint):Guard<string>=>(v):v is string=>text(v)&&v.length<=39&&/^(0|[1-9]\d*)$/.test(v)&&BigInt(v)<=max;
const i64=integer(9223372036854775807n),u64=integer(18446744073709551615n),wide=integer((1n<<127n)-1n);
const hash:Guard<string>=(v):v is string=>text(v)&&/^[0-9a-f]{16}$/.test(v);
const exact=<T>(f:{[K in keyof T]-?:Guard<T[K]>}):Guard<T>=>(v):v is T=>typeof v==='object'&&v!==null&&!Array.isArray(v)&&Reflect.ownKeys(v).length===Object.keys(f).length&&Object.entries(f).every(([k,g])=>Object.hasOwn(v,k)&&(g as Guard<unknown>)((v as Record<string,unknown>)[k]));
const array=<T>(g:Guard<T>):Guard<T[]>=>(v):v is T[]=>Array.isArray(v)&&Array.from({length:v.length},(_,i)=>Object.hasOwn(v,i)&&g(v[i])).every(Boolean);
const sorted=<T>(g:Guard<T>,less:(a:T,b:T)=>boolean):Guard<T[]>=>(v):v is T[]=>array(g)(v)&&v.every((x,i)=>i===0||less(v[i-1],x));
const nullable=<T>(g:Guard<T>):Guard<T|null>=>(v):v is T|null=>v===null||g(v);
const literal=<T extends string>(s:T):Guard<T>=>(v):v is T=>v===s;
/** Decimal-to-fixed transport encoding, not a production formula. */
export function quantityBits(value:string):string|null {
  if(value.length>128||!/^(0|[1-9]\d*)(\.\d+)?$/.test(value))return null;
  const [whole,fraction='']=value.split('.');const denominator=10n**BigInt(fraction.length);
  const numerator=BigInt(whole+fraction)*65536n;let result=numerator/denominator;const remainder=numerator%denominator;
  if(remainder*2n>denominator||(remainder*2n===denominator&&result%2n!==0n))result++;
  return result<=9223372036854775807n?result.toString():null;
}
const fixed=(bits:16|32):Guard<P.FixedValue>=>{
  const shape=exact<P.FixedValue>({value:(v):v is string=>text(v)&&v.length<=128&&/^(0|[1-9]\d*)(\.\d+)?$/.test(v),bits:i64,fractional_bits:(v):v is number=>v===bits});
  return (v):v is P.FixedValue=>{
    if(!shape(v))return false;
    const [whole,fraction='']=v.value.split('.');const denominator=10n**BigInt(fraction.length),n=BigInt(whole+fraction)*(1n<<BigInt(bits));
    let raw=n/denominator;const r=n%denominator;if(r*2n>denominator||(r*2n===denominator&&raw%2n!==0n))raw++;
    return raw===BigInt(v.bits);
  };
};
const qty=fixed(16),fx=fixed(32);
const commands={
  Create:exact<Extract<P.ProductionCommand,{type:'Create'}>>({type:literal('Create'),model:id,requested_ic_bits:i64}),
  SetIC:exact<Extract<P.ProductionCommand,{type:'SetIC'}>>({type:literal('SetIC'),line:u64,requested_ic_bits:i64}),
  Pause:exact<Extract<P.ProductionCommand,{type:'Pause'}>>({type:literal('Pause'),line:u64,paused:bool}),
  Switch:exact<Extract<P.ProductionCommand,{type:'Switch'}>>({type:literal('Switch'),line:u64,model:id}),
  Cancel:exact<Extract<P.ProductionCommand,{type:'Cancel'}>>({type:literal('Cancel'),line:u64}),
} satisfies {[K in P.ProductionCommand['type']]:Guard<Extract<P.ProductionCommand,{type:K}>>};
export const isProductionCommand:Guard<P.ProductionCommand>=(v):v is P.ProductionCommand=>Object.values(commands).some(g=>g(v));
const line=exact<P.ProductionLineView>({id:u64,nation:u16,model:id,requested_ic:qty,paused:bool,efficiency:fx,carry:qty});
const resource=exact<P.ProductionResourceView>({resource:id,required:qty,reserved:qty,debited:qty});
const dayLine=exact<P.ProductionLineDayView>({starting:line,effective_ic:qty,stability_factor:fx,planned:qty,fulfillment:fx,actual:qty,output:i64,ending_efficiency:fx,ending_carry:qty,resources:sorted(resource,(a,b)=>a.resource<b.resource)});
const flow=exact<P.ProductionFlowView>({resource:id,flow:i64,required_raw:wide,reserved_raw:wide,debited_raw:wide,unused_raw:wide});
const nationDay=exact<P.ProductionNationDayView>({nation:u16,budget:qty,stability:fx,unused_ic:qty,flows:sorted(flow,(a,b)=>a.resource<b.resource),lines:sorted(dayLine,(a,b)=>BigInt(a.starting.id)<BigInt(b.starting.id))});
const day=exact<P.ProductionDayView>({tick:u64,nations:sorted(nationDay,(a,b)=>a.nation<b.nation)});
const modelResource=exact<P.ProductionModelResourceView>({resource:id,per_item:qty});
const model=exact<P.ProductionModelView>({model:id,name_key:text,family:id,generation:u32,unit_cost:qty,resources:sorted(modelResource,(a,b)=>a.resource<b.resource)});
const stock=exact<P.ProductionStockView>({model:id,available:i64});
const nation=exact<P.ProductionNationView>({nation:u16,allowed_models:sorted(id,(a,b)=>a<b),stock:sorted(stock,(a,b)=>a.model<b.model),military_ic:qty});
const discard=exact<P.ProductionDiscardView>({tick:u64,line:u64,nation:u16,model:id,carry:qty});
const pending=exact<P.ProductionPendingView>({tick:u64,nation:u16,sequence:u64,command:isProductionCommand});
const view=exact<P.ProductionView>({state_hash:hash,definitions_hash:hash,next_line_id:u64,models:sorted(model,(a,b)=>a.model<b.model),nations:sorted(nation,(a,b)=>a.nation<b.nation),lines:sorted(line,(a,b)=>BigInt(a.id)<BigInt(b.id)),day:nullable(day),discards:array(discard),pending:sorted(pending,(a,b)=>BigInt(a.tick)<BigInt(b.tick)||(a.tick===b.tick&&(a.nation<b.nation||(a.nation===b.nation&&BigInt(a.sequence)<BigInt(b.sequence)))))});
export const isProductionView:Guard<P.ProductionView>=(v):v is P.ProductionView=>view(v);
