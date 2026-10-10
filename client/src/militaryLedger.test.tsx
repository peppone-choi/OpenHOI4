import {readFileSync} from 'node:fs';
import {renderToStaticMarkup} from 'react-dom/server';
import {expect,test} from 'vitest';
import {isServerMessage} from './network';
import {MilitaryLedgerResponses} from './militaryLedgerResponses';
import {MilitaryQtyLedger} from './components/MilitaryQtyLedger';
import {Localization,translatorFor} from './i18n';
import type {ServerMessage} from './proto/protocol';
const result=JSON.parse(readFileSync(new URL('../../target/wp16/military-ledger-wire-fixture.json',import.meta.url),'utf8')) as Extract<ServerMessage,{type:'MilitaryNormalLedgerResult'}>;
const base=JSON.parse(readFileSync(new URL('../../target/wp16/military-wire-fixture.json',import.meta.url),'utf8')).military;
test('actual Rust Qty packet validates and renders seven raw-value tooltips in ko/en',()=>{
  expect(isServerMessage(result)).toBe(true);
  for(const language of ['ko','en'] as const){
    const html=renderToStaticMarkup(<Localization.Provider value={{language,t:translatorFor(language)}}><MilitaryQtyLedger display={{ledger:result.ledger,template:'example',status:'ready',reasonKey:null}}/></Localization.Provider>);
    expect(html.match(/data-qty-ledger-field=/g)).toHaveLength(7);
    for(const f of result.ledger!.fields){expect(html).toContain(`data-bits="${f.value.bits}"`);expect(html).toContain(f.entries[0].component);}
    expect(html).not.toMatch(/Unknown message|알 수 없는 메시지/);
  }
});
test('large Rust Qty strings and 16 repeated contributor occurrences remain exact',()=>{
  const worst=JSON.parse(readFileSync(new URL('../../target/wp22/ledger-worst.json',import.meta.url),'utf8')) as typeof result;
  expect(isServerMessage(worst)).toBe(true);
  for(const field of worst.ledger!.fields){expect(field.value.bits).toBe('8388608000000000000');expect(field.entries).toHaveLength(16);expect(new Set(field.entries.map(e=>e.id)).size).toBe(16);}
});
test('wire rejects extra/missing fields, malformed raw values, contributor order/IDs/reference inconsistencies',()=>{
  const changes:Array<(v:any)=>void>=[
    v=>{v.ledger.extra=1;},v=>{delete v.ledger.tick;},v=>{v.ledger.fields.reverse();},
    v=>{v.ledger.fields[0].base.bits='1';},v=>{v.ledger.fields[0].entries[0].value.value='-1';},
    v=>{v.ledger.fields[0].entries[0].value.bits='9223372036854775808';},
    v=>{v.ledger.fields[0].entries[0].position=12;},v=>{v.ledger.fields[0].entries.reverse();},
    v=>{v.ledger.fields[0].entries[0].id='forged';},v=>{v.ledger.fields[1].entries[0].component='other';},
    v=>{v.ledger.fields[0].entries[0].component='x'.repeat(65);},
    v=>{v.ledger.fields[0].value={value:'0',bits:'0',fractional_bits:16};},
    v=>{v.supported=false;},v=>{v.reason_key='unknown-template';},
  ];
  for(const change of changes){const bad=structuredClone(result);change(bad);expect(isServerMessage(bad)).toBe(false);}
});
test('definitions, template existence and final bits bind to base; provenance tick/hash never trigger refresh',()=>{
  const inbox=new MilitaryLedgerResponses(),socket={},sent:string[]=[];
  const send=(request:string)=>{sent.push(request);return true;};
  const start=()=>{inbox.begin(socket);inbox.setBase(base);inbox.select('example');inbox.issue(socket,send);inbox.accept(socket,{type:'MilitaryNormalLedgerCapabilityResult',request:sent.at(-1)!,supported:true,reason_key:null});inbox.issue(socket,send);};
  start();expect(inbox.accept(socket,{...result,request:sent.at(-1)!})?.status).toBe('ready');
  for(let i=0;i<20;i++){inbox.setBase({...base,state_hash:'1111111111111111'});expect(inbox.issue(socket,send)).toBeNull();}
  expect(sent).toHaveLength(2);
  expect(inbox.end(socket)).toBe(true);expect(inbox.display.status).toBe('stale');inbox.select(null);expect(inbox.display.template).toBeNull();expect(inbox.display.status).toBe('idle');
  for(const change of [
    (v:any)=>{v.definitions_hash='1111111111111111';},
    (v:any)=>{v.template='missing';},
    (v:any)=>{v.fields[0].value={value:'0',bits:'0',fractional_bits:16};},
  ]){
    start();const ledger=structuredClone(result.ledger)!;change(ledger);
    expect(inbox.accept(socket,{...result,ledger,request:sent.at(-1)!})?.reasonKey).toBe('military-ledger-definition-mismatch');
    inbox.select('other');expect(inbox.display.status).toBe('unavailable');expect(inbox.issue(socket,send)).toBeNull();
    inbox.setBase({...base,definitions_hash:'2222222222222222'});inbox.select('example');expect(inbox.issue(socket,send)).not.toBeNull();
  }
});
test('V -> null -> ledger reply -> V resumes with a fresh request without a definition-mismatch lock',()=>{
  const inbox=new MilitaryLedgerResponses(),socket={},sent:string[]=[];
  const send=(request:string)=>{sent.push(request);return true;};
  inbox.begin(socket);inbox.setBase(base);inbox.select('example');inbox.issue(socket,send);
  inbox.accept(socket,{type:'MilitaryNormalLedgerCapabilityResult',request:sent[0],supported:true,reason_key:null});
  const beforeLoss=inbox.issue(socket,send)!;
  inbox.setBase(null);
  expect(inbox.accept(socket,{...result,request:beforeLoss})).toBeNull();
  // App attempts a follow-up after the correlated reply releases the slot.
  const withoutBase=inbox.issue(socket,send);
  if(withoutBase)inbox.accept(socket,{...result,request:withoutBase});
  for(let i=0;i<20;i++)expect(inbox.issue(socket,send)).toBeNull();
  inbox.setBase(base);
  const recovery=inbox.issue(socket,send);
  expect(recovery).not.toBeNull();
  expect(withoutBase).toBeNull();
  expect(inbox.display.reasonKey).toBeNull();
  expect(recovery).not.toBe(beforeLoss);
  expect(inbox.accept(socket,{...result,request:beforeLoss})).toBeNull();
  expect(inbox.accept(socket,{...result,request:recovery!})?.status).toBe('ready');
  expect(inbox.display.ledger).toEqual(result.ledger);
});
test('restored V waits for the one old slot, discards its late reply and fetches fresh authority',()=>{
  const inbox=new MilitaryLedgerResponses(),socket={},sent:string[]=[];
  const send=(request:string)=>{sent.push(request);return true;};
  inbox.begin(socket);inbox.setBase(base);inbox.select('example');inbox.issue(socket,send);
  inbox.accept(socket,{type:'MilitaryNormalLedgerCapabilityResult',request:sent[0],supported:true,reason_key:null});
  const old=inbox.issue(socket,send)!;
  inbox.setBase(null);inbox.setBase(base);
  expect(inbox.issue(socket,send)).toBeNull();expect(sent).toHaveLength(2);
  expect(inbox.accept(socket,{...result,request:old})).toBeNull();
  const fresh=inbox.issue(socket,send)!;expect(fresh).not.toBe(old);expect(sent).toHaveLength(3);
  expect(inbox.accept(socket,{...result,request:fresh})?.status).toBe('ready');
});
