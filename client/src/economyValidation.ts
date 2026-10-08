import {isProductionCommand} from './productionValidation';
import type {FixedValue,LawSelection,EconomyResource,EconomyStateFlow,IndustryContribution,IndustryMultiplier,EconomyLedgerView,ConstructionProjectView,ConstructionEntryView,ConstructionLedgerView,EconomyNationView,EconomyView,IndustrialScoreView,EconomyCommand,AuthorityCommandView,EconomyPendingView} from './proto/protocol';
type Guard<T>=(v:unknown)=>v is T;
type Shape<T>={[K in keyof T]-?:Guard<T[K]>};
const text:Guard<string>=(v):v is string=>typeof v==='string';
const u16:Guard<number>=(v):v is number=>typeof v==='number'&&Number.isInteger(v)&&v>=0&&v<=65535;
const bool:Guard<boolean>=(v):v is boolean=>typeof v==='boolean';
const decimal:Guard<string>=(v):v is string=>text(v)&&/^(0|[1-9]\d*)$/.test(v);
const i64:Guard<string>=(v):v is string=>decimal(v)&&(v.length<19||(v.length===19&&v<='9223372036854775807'));
const u64:Guard<string>=(v):v is string=>decimal(v)&&(v.length<20||(v.length===20&&v<='18446744073709551615'));
const id:Guard<string>=(v):v is string=>text(v)&&/^[a-z0-9_]+$/.test(v);
const hash:Guard<string>=(v):v is string=>text(v)&&/^[0-9a-f]{16}$/.test(v);
function exact<T>(fields:Shape<T>):Guard<T>{return (v):v is T=>typeof v==='object'&&v!==null&&!Array.isArray(v)&&Reflect.ownKeys(v).length===Object.keys(fields).length&&Object.entries(fields).every(([k,g])=>Object.hasOwn(v,k)&&(g as Guard<unknown>)((v as Record<string,unknown>)[k]));}
const array=<T>(g:Guard<T>):Guard<T[]> => (v):v is T[]=>Array.isArray(v)&&Array.from({length:v.length},(_,i)=>Object.hasOwn(v,i)&&g(v[i])).every(Boolean);
const ordered=<T>(g:Guard<T>,less:(a:T,b:T)=>boolean):Guard<T[]> => (v):v is T[]=>array(g)(v)&&v.every((item,i)=>i===0||less(v[i-1],item));
const nullable=<T>(g:Guard<T>):Guard<T|null> =>(v):v is T|null=>v===null||g(v);
const signedI64:Guard<string>=(v):v is string=>text(v)&&/^(0|-?[1-9]\d*)$/.test(v)&&v.length<=20&&BigInt(v)>=-9223372036854775808n&&BigInt(v)<=9223372036854775807n;
function fixedEncoding(v:FixedValue):boolean{
  const sign=v.value.startsWith('-')?-1n:1n;
  const [integer,fraction='']=v.value.replace('-','').split('.');
  const denominator=10n**BigInt(fraction.length),numerator=sign*BigInt(integer+fraction)*(1n<<BigInt(v.fractional_bits));
  let quotient=numerator/denominator,remainder=numerator%denominator;
  if(remainder<0n){quotient-=1n;remainder+=denominator;}
  const nearest=quotient+((2n*remainder>denominator||(2n*remainder===denominator&&quotient%2n!==0n))?1n:0n);
  return nearest===BigInt(v.bits);
}
const fixed=(fractional_bits:16|32,signed=false):Guard<FixedValue>=>{
  const guard=exact<FixedValue>({value:(v):v is string=>text(v)&&v.length<=128&&(signed?/^-?(0|[1-9]\d*)(\.\d+)?$/:/^(0|[1-9]\d*)(\.\d+)?$/).test(v),bits:signed?signedI64:i64,fractional_bits:(v):v is number=>v===fractional_bits});
  return (v):v is FixedValue=>guard(v)&&fixedEncoding(v);
};
const fx=fixed(32),qty=fixed(16),signedFx=fixed(32,true),signedQty=fixed(16,true);
const four=<T>(g:Guard<T>):Guard<[T,T,T,T]> => (v):v is [T,T,T,T]=>array(g)(v)&&v.length===4;
const law=exact<LawSelection>({category:id,law:id,name_key:text});
const resource=exact<EconomyResource>({resource:id,value:i64});
const resources=ordered(resource,(a,b)=>a.resource<b.resource);
const stateFlow=exact<EconomyStateFlow>({state:u16,population:i64,resources});
const contribution=exact<IndustryContribution>({state:u16,building:id,levels:i64,unit_ic:qty,value:qty});
const multiplier=exact<IndustryMultiplier>({source:text,factor:fx,applied:qty});
const ledger=exact<EconomyLedgerView>({tick:u64,laws:ordered(law,(a,b)=>a.category<b.category),stability:fx,capacity:i64,contributions:ordered(contribution,(a,b)=>a.state<b.state||(a.state===b.state&&a.building<b.building)),state_flows:ordered(stateFlow,(a,b)=>a.state<b.state),population:i64,resources,multipliers:array(multiplier),total_ic:qty,minimum:fx,ratios:four(fx),allocation:four(qty),consumer_residual:qty});
const dormant:Guard<string>=(v):v is string=>text(v)&&['economy-no-ownership','economy-target-conflict','economy-slot-cap','economy-zero-cap','economy-zero-factor'].includes(v);
const project=exact<ConstructionProjectView>({id:u64,state:u16,building:id,target:i64,progress:qty,dormancy:nullable(dormant)});
const entry=exact<ConstructionEntryView>({state:u16,building:id,target:i64,starting_progress:qty,cost:qty,daily_cap:qty,infrastructure:fx,owner:u16,factor_evaluated:bool,project:u64,factor:fx,consumed:qty,applied:qty,discarded:qty,completed:bool,dormancy:nullable(dormant)});
const construction=exact<ConstructionLedgerView>({tick:u64,budget:qty,entries:array(entry),unused:qty});
const unique=<T>(g:Guard<T>,key:(v:T)=>string):Guard<T[]> => (v):v is T[]=>array(g)(v)&&new Set(v.map(key)).size===v.length;
const nation=exact<EconomyNationView>({nation:u16,laws:ordered(law,(a,b)=>a.category<b.category),political_capital:qty,stability:fx,mobilization:fx,ratios:four(fx),committed:i64,reserved:i64,capacity:i64,available:i64,overcommitted:i64,projects:unique(project,p=>p.id),ledger,construction});
const score=exact<IndustrialScoreView>({nation:u16,tick:u64,input_tick:u64,weight:signedFx,input:qty,term:signedQty});
const u8:Guard<number>=(v):v is number=>typeof v==='number'&&Number.isInteger(v)&&v>=0&&v<=255;
const u32:Guard<number>=(v):v is number=>typeof v==='number'&&Number.isInteger(v)&&v>=0&&v<=4294967295;
const literal=<T extends string>(s:T):Guard<T>=>(v):v is T=>v===s;
const actionGuards={Allocate:exact<Extract<EconomyCommand,{type:'Allocate'}>>({type:literal('Allocate'),ratios_bits:four(i64)}),Construct:exact<Extract<EconomyCommand,{type:'Construct'}>>({type:literal('Construct'),project:u64,state:u16,building:id}),Cancel:exact<Extract<EconomyCommand,{type:'Cancel'}>>({type:literal('Cancel'),project:u64}),Reorder:exact<Extract<EconomyCommand,{type:'Reorder'}>>({type:literal('Reorder'),projects:unique(u64,v=>v)}),ChangeLaw:exact<Extract<EconomyCommand,{type:'ChangeLaw'}>>({type:literal('ChangeLaw'),law:id})} satisfies {[K in EconomyCommand['type']]:Guard<Extract<EconomyCommand,{type:K}>>};
const action:Guard<EconomyCommand>=(v):v is EconomyCommand=>Object.values(actionGuards).some(g=>g(v));
const commandGuards={Production:exact<Extract<AuthorityCommandView,{type:'Production'}>>({type:literal('Production'),command:isProductionCommand}),Pause:exact<Extract<AuthorityCommandView,{type:'Pause'}>>({type:literal('Pause'),paused:bool}),SetSpeed:exact<Extract<AuthorityCommandView,{type:'SetSpeed'}>>({type:literal('SetSpeed'),speed:u8}),Move:exact<Extract<AuthorityCommandView,{type:'Move'}>>({type:literal('Move'),unit:u32,destination:u16}),Stop:exact<Extract<AuthorityCommandView,{type:'Stop'}>>({type:literal('Stop'),unit:u32}),Effects:exact<Extract<AuthorityCommandView,{type:'Effects'}>>({type:literal('Effects'),program:id}),Economy:exact<Extract<AuthorityCommandView,{type:'Economy'}>>({type:literal('Economy'),command:action})} satisfies {[K in AuthorityCommandView['type']]:Guard<Extract<AuthorityCommandView,{type:K}>>};
const command:Guard<AuthorityCommandView>=(v):v is AuthorityCommandView=>Object.values(commandGuards).some(g=>g(v));
const pending=exact<EconomyPendingView>({tick:u64,nation:u16,sequence:u64,command});
const economy=exact<EconomyView>({state_hash:hash,pending:ordered(pending,(a,b)=>BigInt(a.tick)<BigInt(b.tick)||(a.tick===b.tick&&(a.nation<b.nation||(a.nation===b.nation&&BigInt(a.sequence)<BigInt(b.sequence))))),definitions_hash:hash,nations:ordered(nation,(a,b)=>a.nation<b.nation),industrial_scores:nullable(ordered(score,(a,b)=>a.nation<b.nation))});
export function isEconomyView(v:unknown):v is EconomyView{return economy(v);}
