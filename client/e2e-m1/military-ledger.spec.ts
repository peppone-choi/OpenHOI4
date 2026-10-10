import {test as base,expect,type Page,type WebSocketRoute} from '@playwright/test';
import {spawn,type ChildProcess} from 'node:child_process';
import {createServer} from 'node:net';
import {cpSync,mkdirSync,mkdtempSync,readFileSync,writeFileSync,createWriteStream} from 'node:fs';
import {tmpdir} from 'node:os';
import {resolve} from 'node:path';
import {decode,encode} from '@msgpack/msgpack';
import type {ClientMessage,ServerMessage} from '../src/proto/protocol';
import {upgradeGate} from './support/upgradeGate';
type LedgerReply=Extract<ServerMessage,{type:'MilitaryNormalLedgerResult'}>;
type Hosts={native:string;limited:string;legacy:string;restored:string;restoredHash:string};
async function freePort(){const server=createServer();await new Promise<void>(done=>server.listen(0,'127.0.0.1',done));const address=server.address();if(!address||typeof address==='string')throw Error('port');const port=address.port;await new Promise<void>(done=>server.close(()=>done()));return port;}
const test=base.extend<{}, {hosts:Hosts}>({hosts:[async({},use,info)=>{
  const binary=resolve(process.env.OH_SERVER_EXECUTABLE??'../target/debug/oh_server');
  const output=resolve(info.project.outputDir,`ledger-hosts-${info.workerIndex}`);mkdirSync(output,{recursive:true});
  const root=mkdtempSync(resolve(tmpdir(),'openhoi-ledger-'));cpSync(resolve('../data/packs/testland_m2_military'),resolve(root,'testland'),{recursive:true});
  const path=resolve(root,'testland/common/military/initial.toml');
  const source=readFileSync(path,'utf8').replace('training_days = { m2_small = 2 }','training_days = { m2_small = 2, m2_other = 2 }')
    .replace("\n'''\n",'\n[[templates]]\nid = "m2_other"\ncombat = ["m2_small_component", "m2_small_component"]\nsupport = []\nbindings = [{ family = "m2_equipment", model = "m2_equipment_1" }]\n\'\'\'\n');
  writeFileSync(path,source);
  const children:ChildProcess[]=[],records:Array<{pid:number|undefined;args:string[];exitCode:number|null;signal:NodeJS.Signals|null;alive:boolean}>=[];
  const ledger=()=>writeFileSync(resolve(output,'pid-ledger.json'),JSON.stringify({owner:process.pid,records},null,2));
  const start=async(budget?:number,executable=binary,packRoot=root,save?:string)=>{
    const port=await freePort(),args=['--port',String(port),'--pack-root',packRoot,'--scenario','m2_military',...(budget?['--military-ledger-budget-bytes',String(budget)]:[]),...(save?['--load-save',save]:[])];
    const child=spawn(executable,args,{cwd:resolve('..'),stdio:['ignore','pipe','pipe']});children.push(child);
    const row={pid:child.pid,args,exitCode:null as number|null,signal:null as NodeJS.Signals|null,alive:true};records.push(row);ledger();
    child.stdout!.pipe(createWriteStream(resolve(output,`server-${port}.stdout`)));child.stderr!.pipe(createWriteStream(resolve(output,`server-${port}.stderr`)));
    child.on('exit',(code,signal)=>{row.exitCode=code;row.signal=signal;row.alive=false;ledger();});
    const url=`http://127.0.0.1:${port}`;
    await expect.poll(async()=>{if(child.exitCode!==null)throw Error('host exited');try{return(await fetch(url)).status;}catch{return 0;}}).toBe(200);
    return url;
  };
  const oldBinary=process.env.OH_LEGACY_SERVER_EXECUTABLE;
  if(!oldBinary)throw Error('OH_LEGACY_SERVER_EXECUTABLE must name an independently built baseline binary');
  const checkpoint=resolve(process.env.OH_M2_MILITARY_OUTPUT_DIR??'../target/wp23/checkpoint');
  const restoredRoot=mkdtempSync(resolve(tmpdir(),'openhoi-ledger-restored-'));cpSync(resolve('../data/packs/testland_m2_military'),resolve(restoredRoot,'testland'),{recursive:true});
  const restoredHash=JSON.parse(readFileSync(resolve(checkpoint,'expected.json'),'utf8')).paused_hash as string;
  try{await use({native:await start(),limited:await start(256),legacy:await start(undefined,resolve(oldBinary)),restored:await start(undefined,binary,restoredRoot,resolve(checkpoint,'paused-training.ohsave')),restoredHash});}
  finally{
    for(const child of children){if(child.exitCode!==null||child.signalCode!==null)continue;const exited=new Promise<void>(done=>child.once('exit',()=>done()));child.kill('SIGINT');const timer=setTimeout(()=>child.kill('SIGKILL'),5000);try{await exited;}finally{clearTimeout(timer);}}
    for(const row of records){if(row.pid){try{process.kill(row.pid,0);row.alive=true;}catch{row.alive=false;}}expect(row.alive).toBe(false);expect(row.exitCode).toBe(0);}ledger();
  }
},{scope:'worker'}]});
function observe(page:Page){
  const sent:ClientMessage[]=[],received:ServerMessage[]=[];let deltas=0;
  page.on('websocket',socket=>{socket.on('framesent',e=>{if(e.payload instanceof Buffer)sent.push(decode(e.payload) as ClientMessage);});socket.on('framereceived',e=>{if(e.payload instanceof Buffer){const m=decode(e.payload) as ServerMessage;received.push(m);if(m.type==='Delta')deltas++;}});});
  return {sent,received,deltas:()=>deltas,queries:()=>sent.filter(m=>m.type==='Query'&&m.kind.startsWith('military-normal-ledger.v1'))};
}
async function open(page:Page){await page.getByRole('button',{name:'Open military information',exact:true}).click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');}
async function select(page:Page,id='m2_small'){await page.locator(`[data-military-template="${id}"]`).getByRole('button',{name:'Normal template contributions',exact:true}).click();}
async function intercept(page:Page,unsupported=false){
  const held:Array<{route:WebSocketRoute;reply:LedgerReply}>=[],queries:Array<{request:string;kind:string}>=[];let disconnect=()=>{};
  await page.routeWebSocket('**/ws',route=>{
    const server=route.connectToServer();disconnect=()=>server.close({code:1000,reason:'test-disconnect'});
    route.onMessage(frame=>{
      const m=decode(frame as Uint8Array) as ClientMessage;
      if(m.type==='Query'&&m.kind.startsWith('military-normal-ledger.v1')){
        queries.push({request:m.request,kind:m.kind});
        if(unsupported){route.send(Buffer.from(encode({type:'QueryResult',request:m.request,supported:false,reason_key:'unsupported-query',state:null})));return;}
      }
      server.send(frame);
    });
    server.onMessage(frame=>{const m=decode(frame as Uint8Array) as ServerMessage;if(m.type==='MilitaryNormalLedgerResult'){held.push({route,reply:m});return;}route.send(frame);});
  });
  return {held,queries,disconnect:()=>disconnect()};
}
const release=(row:{route:WebSocketRoute;reply:LedgerReply},reply=row.reply)=>row.route.send(Buffer.from(encode(reply)));

test('native query -> exact served bundle -> seven ko/en keyboard/mobile tooltips, without Delta polling',async({page,request,hosts},info)=>{
  const wire=observe(page);await page.goto(hosts.native);await open(page);expect(wire.queries()).toHaveLength(0);
  const button=page.locator('[data-military-template="m2_small"]').getByRole('button',{name:'Normal template contributions',exact:true});await button.focus();await page.keyboard.press('Enter');
  const ledger=page.getByTestId('military-qty-ledger');await expect(ledger).toHaveAttribute('data-status','ready');
  const reply=wire.received.find(m=>m.type==='MilitaryNormalLedgerResult'&&m.supported) as LedgerReply;
  const military=wire.received.find(m=>m.type==='MilitaryResult'&&m.supported) as Extract<ServerMessage,{type:'MilitaryResult'}>;
  expect(reply.ledger!.definitions_hash).toBe(military.military!.definitions_hash);expect(reply.ledger!.template).toBe('m2_small');
  await expect(ledger.locator('[data-qty-ledger-field]')).toHaveCount(7);
  for(const field of reply.ledger!.fields){const details=ledger.locator(`[data-qty-ledger-field="${field.field}"]`);await expect(details.locator('summary output')).toHaveAttribute('data-bits',field.value.bits);await details.locator('summary').focus();await page.keyboard.press('Enter');await expect(details.locator('[data-contribution-id]')).toHaveCount(field.entries.length);await expect(details).toContainText('m2_small_component');}
  expect(wire.queries().map(m=>m.type==='Query'?m.kind:'')).toEqual(['military-normal-ledger.v1','military-normal-ledger.v1:m2_small']);
  const delta=wire.deltas();await expect.poll(()=>wire.deltas()).toBeGreaterThan(delta+2);expect(wire.queries()).toHaveLength(2);
  await expect(ledger).toContainText('declared value');await ledger.screenshot({path:info.outputPath('ledger-seven-en.png')});
  await page.getByRole('combobox',{name:'Language',exact:true}).selectOption('ko');await expect(ledger).toContainText('확인된 편제 기여 내역');await expect(ledger).toContainText('전투 구성 요소');
  await page.setViewportSize({width:390,height:844});await ledger.scrollIntoViewIfNeeded();expect(await ledger.evaluate(e=>e.scrollWidth<=e.clientWidth)).toBe(true);await ledger.screenshot({path:info.outputPath('ledger-seven-mobile-ko.png')});await ledger.locator('[data-qty-ledger-field="strength"]').screenshot({path:info.outputPath('ledger-strength-mobile-ko.png')});
  const html=await(await request.get(hosts.native)).text(),script=html.match(/<script[^>]*src="([^"]+)"/)![1];expect(await(await request.get(hosts.native+script)).body()).toEqual(readFileSync(resolve('dist',script.slice(1))));
  expect(wire.sent.some(m=>m.type==='MilitaryCommand')).toBe(false);writeFileSync(info.outputPath('native-ledger-wire.json'),JSON.stringify(wire,null,2));
});
test('native response budget rejects only ledger and leaves legacy military and socket usable',async({page,hosts})=>{
  const wire=observe(page);await page.goto(hosts.limited);await open(page);await select(page);
  const ledger=page.getByTestId('military-qty-ledger');await expect(ledger).toHaveAttribute('data-status','unavailable');await expect(ledger).toContainText('response limit');await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
  const delta=wire.deltas();await expect.poll(()=>wire.deltas()).toBeGreaterThan(delta+1);
  const base=wire.received.filter(m=>m.type==='MilitaryResult'&&m.supported).at(-1) as Extract<ServerMessage,{type:'MilitaryResult'}>;expect(Object.keys(base.military!)).toHaveLength(10);expect(Object.keys(base.military!.templates[0])).toHaveLength(3);
  await select(page,'m2_other');await expect(ledger).toHaveAttribute('data-status','unavailable');expect(wire.queries()).toHaveLength(3);
});
test('old-server shaped capability fallback preserves base panel and disables only ledger queries (injected)',async({page,hosts})=>{
  const wire=await intercept(page,true);await page.goto(hosts.native);await open(page);await select(page);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','unsupported');
  await select(page,'m2_other');await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','unsupported');expect(wire.queries).toHaveLength(1);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
});
test('actual baseline server and new served client preserve military view while ledger falls back',async({page,request,hosts})=>{
  const wire=observe(page);await page.goto(hosts.legacy);await open(page);await select(page);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','unsupported');
  await select(page,'m2_other');expect(wire.queries()).toHaveLength(1);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
  expect(wire.received.some(m=>m.type==='QueryResult'&&m.request.startsWith('military-ledger:')&&m.reason_key==='unsupported-query')).toBe(true);
  expect(wire.received.some(m=>m.type==='MilitaryNormalLedgerResult')).toBe(false);
  const html=await(await request.get(hosts.legacy)).text(),script=html.match(/<script[^>]*src="([^"]+)"/)![1];expect(await(await request.get(hosts.legacy+script)).body()).toEqual(readFileSync(resolve('dist',script.slice(1))));
});
test('held A-B-A reply frees one slot but cannot display first A, duplicate or future reply (injected timing)',async({page,hosts})=>{
  const wire=await intercept(page);await page.goto(hosts.native);await open(page);await select(page);await expect.poll(()=>wire.held.length).toBe(1);const old=wire.held[0];
  await select(page,'m2_other');await select(page,'m2_small');expect(wire.held).toHaveLength(1);
  release(old,{...old.reply,request:'military-ledger:999999'});await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','loading');
  release(old);await expect.poll(()=>wire.held.length).toBe(2);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','loading');
  release(old);release(wire.held[1]);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','ready');
});
test('verified data becomes stale on disconnect, new socket clears it and probes again (injected close)',async({page,hosts})=>{
  const wire=await intercept(page);await page.goto(hosts.native);await open(page);await select(page);await expect.poll(()=>wire.held.length).toBe(1);release(wire.held[0]);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','ready');
  wire.disconnect();await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','stale');await page.getByRole('button',{name:'Reconnect',exact:true}).click();await expect(page.getByTestId('military-qty-ledger')).toHaveCount(0);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
  await select(page);await expect.poll(()=>wire.held.length).toBe(2);release(wire.held[1]);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','ready');expect(wire.queries.filter(q=>q.kind==='military-normal-ledger.v1')).toHaveLength(2);
});
test('CONNECTING gate and close/reopen admit fresh requests without phantom slots',async({page,hosts})=>{
  const gate=await upgradeGate(hosts.native);gate.hold();
  try{await page.goto(gate.url);await page.getByRole('button',{name:'Open military information',exact:true}).click();gate.release();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');await select(page);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','ready');await page.getByTestId('military-panel').getByRole('button',{name:'Close military information',exact:true}).click();await open(page);await select(page);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','ready');}
  finally{await page.close();await gate.close();}
});
test('native unknown/invalid template is a correlated domain error and a valid control still works',async({hosts},info)=>{
  const socket=new WebSocket(hosts.native.replace('http:','ws:')+'/ws');socket.binaryType='arraybuffer';
  const received:ServerMessage[]=[];socket.onmessage=e=>received.push(decode(new Uint8Array(e.data as ArrayBuffer)) as ServerMessage);
  try{
    await new Promise<void>((done,reject)=>{socket.onopen=()=>done();socket.onerror=()=>reject(Error('native socket'));});
    const send=(m:ClientMessage)=>socket.send(encode(m));
    send({type:'Hello',protocol_version:'m0-v1'});await expect.poll(()=>received.some(m=>m.type==='Welcome'&&m.accepted)).toBe(true);
    send({type:'Join',session:'local',nation:null});await expect.poll(()=>received.some(m=>m.type==='Snapshot')).toBe(true);
    let serial=100n;
    const query=async(kind:string)=>{const request=`military-ledger:${++serial}`;send({type:'Query',request,kind});await expect.poll(()=>received.some(m=>'request' in m&&m.request===request)).toBe(true);return received.find(m=>'request' in m&&m.request===request)!;};
    expect(await query('military-normal-ledger.v1')).toMatchObject({type:'MilitaryNormalLedgerCapabilityResult',supported:true});
    expect(await query('military-normal-ledger.v1:missing')).toMatchObject({type:'MilitaryNormalLedgerResult',supported:false,reason_key:'unknown-template',ledger:null});
    expect(await query('military-normal-ledger.v1:'+'x'.repeat(65))).toMatchObject({type:'MilitaryNormalLedgerResult',supported:false,reason_key:'invalid-template',ledger:null});
    expect(await query('military-normal-ledger.v1:m2_small')).toMatchObject({type:'MilitaryNormalLedgerResult',supported:true});
    const legacy=await query('military') as Extract<ServerMessage,{type:'MilitaryResult'}>;
    expect(legacy.supported).toBe(true);expect(Object.keys(legacy.military!)).toHaveLength(10);expect(Object.keys(legacy.military!.templates[0])).toHaveLength(3);
    writeFileSync(info.outputPath('native-domain-errors.json'),JSON.stringify(received,null,2));
  }finally{socket.close();}
});
test('malformed ledger uses transport disconnect and keeps last verified contribution stale (injected)',async({page,hosts})=>{
  const wire=await intercept(page);await page.goto(hosts.native);await open(page);await select(page);await expect.poll(()=>wire.held.length).toBe(1);release(wire.held[0]);await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','ready');
  const bad=structuredClone(wire.held[0].reply);bad.ledger!.fields[0].entries[0].id='forged';release(wire.held[0],bad);
  await expect(page.getByTestId('military-qty-ledger')).toHaveAttribute('data-status','stale');await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','stale');
  await expect(page.getByTestId('military-qty-ledger').locator('[data-qty-ledger-field]')).toHaveCount(7);
});
test('restored V7 Training authority serves actual Qty contributions with preserved paused hash and tick',async({page,hosts},info)=>{
  const wire=observe(page);await page.goto(hosts.restored);await open(page);await select(page);const ledger=page.getByTestId('military-qty-ledger');await expect(ledger).toHaveAttribute('data-status','ready');
  const reply=wire.received.find(m=>m.type==='MilitaryNormalLedgerResult'&&m.supported) as LedgerReply;
  const military=wire.received.find(m=>m.type==='MilitaryResult'&&m.supported) as Extract<ServerMessage,{type:'MilitaryResult'}>;
  expect(reply.ledger!.state_hash).toBe(hosts.restoredHash);expect(reply.ledger!.tick).toBe('144');expect(military.military!.state_hash).toBe(hosts.restoredHash);expect(military.military!.jobs).toHaveLength(6);
  const normal=military.military!.templates[0].normal;for(const field of reply.ledger!.fields)expect(field.value.bits).toBe(normal[field.field as keyof Pick<typeof normal,'strength'|'soft_fire'|'hard_fire'|'defense'|'breakthrough'|'frontage'|'supply_use'>].bits);
  expect(wire.sent.some(m=>m.type==='MilitaryCommand')).toBe(false);writeFileSync(info.outputPath('restored-ledger-wire.json'),JSON.stringify(wire,null,2));
});
test('malformed long v1 request never echoes past a small budget and socket still answers legacy control',async({hosts})=>{
  const socket=new WebSocket(hosts.limited.replace('http:','ws:')+'/ws');socket.binaryType='arraybuffer';
  const received:ServerMessage[]=[],sizes:number[]=[];socket.onmessage=e=>{const bytes=new Uint8Array(e.data as ArrayBuffer);received.push(decode(bytes) as ServerMessage);sizes.push(bytes.length);};
  try{
    await new Promise<void>((done,reject)=>{socket.onopen=()=>done();socket.onerror=()=>reject(Error('native socket'));});
    const send=(m:ClientMessage)=>socket.send(encode(m));send({type:'Hello',protocol_version:'m0-v1'});await expect.poll(()=>received.some(m=>m.type==='Welcome'&&m.accepted)).toBe(true);
    send({type:'Join',session:'local',nation:null});await expect.poll(()=>received.some(m=>m.type==='Snapshot')).toBe(true);
    send({type:'Query',request:'military-ledger:'+'1'.repeat(1024),kind:'military-normal-ledger.v1'});await expect.poll(()=>received.some(m=>m.type==='Notice'&&m.key==='invalid-message')).toBe(true);
    const index=received.findIndex(m=>m.type==='Notice'&&m.key==='invalid-message');expect(sizes[index]).toBeLessThanOrEqual(256);
    send({type:'Query',request:'legacy-control',kind:'military'});await expect.poll(()=>received.some(m=>m.type==='MilitaryResult'&&m.request==='legacy-control'&&m.supported)).toBe(true);
  }finally{socket.close();}
});
