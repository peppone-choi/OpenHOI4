import { readFileSync } from 'node:fs';
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test } from 'vitest';
import { decode } from '@msgpack/msgpack';
import { EconomyPanel } from './components/EconomyPanel';
import { EconomyResponses } from './economyResponses';
import { Localization, translatorFor } from './i18n';
import { isServerMessage } from './network';
import type { EconomyView, ServerMessage } from './proto/protocol';

const fixture = JSON.parse(readFileSync(new URL('../../target/wp14/economy-wire-fixtures.json', import.meta.url), 'utf8'))[0];
const result = fixture.expected as Extract<ServerMessage, {type: 'EconomyResult'}>;
const view = result.economy!;
const render = (economy: EconomyView | null, status: 'ready'|'loading'|'stale'|'unsupported'|'disconnected' = 'ready', language: 'en'|'ko' = 'en') => renderToStaticMarkup(
  <Localization.Provider value={{language, t:translatorFor(language)}}>
    <EconomyPanel display={{view:economy,status,reasonKey:null}} world={null} nationId={null} onNationSelect={()=>{}}/>
  </Localization.Provider>
);

test('actual Rust fixture renders fixed values, complete allocation roles and source ledgers in both languages', () => {
  expect(decode(Uint8Array.from(fixture.bytes))).toEqual(result);
  expect(isServerMessage(result)).toBe(true);
  for (const language of ['en','ko'] as const) {
    const html = render(view, 'ready', language);
    for (const role of ['consumer','construction','military','export']) expect(html).toContain(`data-allocation="${role}"`);
    for (const index of [0,1,2,3]) expect(html).toContain(view.nations[0].ledger.allocation[index].value);
    expect(html).toContain(view.nations[0].political_capital.value);
    expect(html).toContain('data-testid="economy-construction"');
    expect(html).toContain(view.nations[0].projects[0].progress.value);
    expect(html).not.toContain('Unknown message');
    expect(html).not.toContain('알 수 없는 메시지');
  }
});

test('large integers and raw fixed bits survive without Number conversion; array-empty views are safe', () => {
  const huge = structuredClone(view);
  huge.nations[0].capacity = '9223372036854775807';
  huge.nations[0].reserved = '9007199254740993';
  huge.nations[0].construction.tick = '18446744073709551615';
  const html = render(huge);
  for (const value of ['9223372036854775807','9007199254740993','18446744073709551615', huge.nations[0].ledger.total_ic.bits]) expect(html).toContain(value);
  const empty = {...view,nations:[],pending:[],industrial_scores:[]};
  expect(render(empty)).toContain('No economic data');
  for (const status of ['loading','unsupported','disconnected'] as const) expect(render(null,status)).not.toContain('economy-nation-body');
  expect(render(view,'stale')).toContain('Last verified data');
});

test('allocation projection keeps consumer/construction/military/export indices and does not invent an absent country', () => {
  const distinct=structuredClone(view);
  distinct.nations[0].ledger.allocation=[1,2,3,4].map(value=>({value:String(value),bits:String(BigInt(value)*65536n),fractional_bits:16})) as typeof distinct.nations[0]['ledger']['allocation'];
  const html=render(distinct);
  for(const [role,value] of [['consumer','1'],['construction','2'],['military','3'],['export','4']]){
    const row=html.match(new RegExp(`data-allocation="${role}"[^]*?</tr>`))![0];
    expect(row).toContain(`data-bits="${BigInt(value)*65536n}"`);
    expect(row.endsWith(`>${value}</abbr></td></tr>`)).toBe(true);
  }
  const absent=renderToStaticMarkup(<EconomyPanel display={{view,status:'ready',reasonKey:null}} world={null} nationId={65535} onNationSelect={()=>{}}/>);
  expect(absent).not.toContain('data-testid="economy-nation-body"');
  const arrays=structuredClone(view);
  for(const nation of arrays.nations){nation.laws=[];nation.projects=[];nation.ledger.laws=[];nation.ledger.resources=[];nation.ledger.contributions=[];nation.ledger.multipliers=[];nation.ledger.state_flows=[];nation.construction.entries=[];}
  expect(render(arrays)).toContain('No economic data');
});

test('issued in-flight request remains current until consumed; socket and nation lifecycles invalidate it', () => {
  const inbox = new EconomyResponses(); const a = {}, b = {};
  inbox.begin(a,1);
  const first = inbox.issue(a,()=>true)!;
  expect(inbox.issue(a,()=>true)).toBeNull();
  expect(inbox.accept(a,{...result,request:first})).toEqual({view,status:'ready',reasonKey:null});
  const second = inbox.followUp(a,()=>true)!;
  expect(first).toBe('economy:1'); expect(second).toBe('economy:2');
  expect(inbox.accept(a,{...result,request:'economy:999'})).toBeNull();
  expect(inbox.accept(a,{...result,request:first})).toBeNull();
  expect(inbox.accept(b,{...result,request:second})).toBeNull();
  expect(inbox.accept(a,{...result,request:second})).toEqual({view,status:'ready',reasonKey:null});
  expect(inbox.accept(a,{...result,request:second})).toBeNull();
  const previousNation = inbox.issue(a,()=>true)!;
  inbox.begin(a,2);
  expect(inbox.accept(a,{...result,request:previousNation})).toBeNull();
  const previousSocket = inbox.issue(a,()=>true)!;
  inbox.begin(b,2);
  expect(inbox.issue(a,()=>true)).toBeNull();
  expect(inbox.accept(a,{...result,request:previousSocket})).toBeNull();
  const current = inbox.issue(b,()=>true)!;
  expect(inbox.end(a)).toBe(false);
  expect(inbox.accept(b,{...result,request:current})).not.toBeNull();
  expect(inbox.end(b)).toBe(true);
  expect(inbox.issue(b,()=>true)).toBeNull();
});

test('latest unsupported reply clears data and old replies cannot undo it', () => {
  const inbox = new EconomyResponses(); const socket = {};
  inbox.begin(socket,null);
  const old = inbox.issue(socket,()=>true)!;
  expect(inbox.accept(socket,{...result,request:old})).not.toBeNull();
  const current = inbox.issue(socket,()=>true)!;
  expect(inbox.accept(socket,{type:'EconomyResult',request:current,supported:false,reason_key:'unsupported-query',economy:null})).toEqual({view:null,status:'unsupported',reasonKey:'unsupported-query'});
  expect(inbox.accept(socket,{...result,request:old})).toBeNull();
});

test('frozen M0/M1 generic unsupported query is accepted only for the known economy request', () => {
  const inbox=new EconomyResponses(),socket={};inbox.begin(socket,null);
  const request=inbox.issue(socket,()=>true)!;
  expect(inbox.accept(socket,{type:'QueryResult',request:'unrelated',supported:false,reason_key:'unsupported-query',state:null})).toBeNull();
  expect(inbox.accept(socket,{type:'QueryResult',request,supported:true,reason_key:null,state:{date:'2000-01-01',hour:0,tick:'0',paused:true,speed:1}})).toBeNull();
  expect(inbox.accept(socket,{type:'QueryResult',request,supported:false,reason_key:'unsupported-query',state:null})).toEqual({view:null,status:'unsupported',reasonKey:'unsupported-query'});
});

test('continuous Delta refreshes coalesce without starving the delayed in-flight reply', () => {
  const inbox=new EconomyResponses(),socket={};inbox.begin(socket,1);
  const first=inbox.issue(socket,()=>true)!;
  for(let delta=0;delta<20;delta++)expect(inbox.issue(socket,()=>true)).toBeNull();
  expect(inbox.accept(socket,{...result,request:first})).toEqual({view,status:'ready',reasonKey:null});
  const follow=inbox.followUp(socket,()=>true)!;
  expect(follow).toBe('economy:2');
  expect(inbox.followUp(socket,()=>true)).toBeNull();
  expect(inbox.accept(socket,{...result,request:first})).toBeNull();
  expect(inbox.accept(socket,{...result,request:follow})).not.toBeNull();
  expect(inbox.followUp(socket,()=>true)).toBeNull();
  inbox.issue(socket,()=>true);inbox.issue(socket,()=>true);
  inbox.begin(socket,2);
  expect(inbox.followUp(socket,()=>true)).toBeNull();
});

test('CONNECTING send refusal cannot latch a pending query across nation or socket changes', () => {
  const inbox=new EconomyResponses(),a={},b={};let open=false;
  const sent:string[]=[];
  const transmit=(request:string)=>{if(!open)return false;sent.push(request);return true;};
  inbox.begin(a,1);
  expect(inbox.issue(a,transmit)).toBeNull();
  expect(inbox.accept(a,{...result,request:'economy:1'})).toBeNull();
  inbox.begin(a,2);
  expect(inbox.issue(a,transmit)).toBeNull();
  inbox.begin(b,4);
  expect(inbox.issue(a,transmit)).toBeNull();
  expect(inbox.issue(b,transmit)).toBeNull();
  open=true;
  const request=inbox.issue(b,transmit)!;
  expect(sent).toEqual([request]);
  expect(inbox.accept(a,{...result,request})).toBeNull();
  expect(inbox.accept(b,{...result,request})).not.toBeNull();
  expect(inbox.followUp(b,transmit)).toBeNull();
  expect(sent).toHaveLength(1);
});
