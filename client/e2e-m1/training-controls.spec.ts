import {test as base,expect,type Page} from '@playwright/test';
import {spawn,type ChildProcess} from 'node:child_process';
import {createServer} from 'node:net';
import {cpSync,mkdirSync,mkdtempSync,readFileSync,writeFileSync,createWriteStream} from 'node:fs';
import {tmpdir} from 'node:os';
import {resolve} from 'node:path';
import {decode,encode} from '@msgpack/msgpack';
import {upgradeGate} from './support/upgradeGate';
import type {ClientMessage,ServerMessage} from '../src/proto/protocol';
type Hosts={fresh:string;restored:string};
async function freePort(){const server=createServer();await new Promise<void>(done=>server.listen(0,'127.0.0.1',done));const address=server.address();if(!address||typeof address==='string')throw Error('port');const port=address.port;await new Promise<void>(done=>server.close(()=>done()));return port;}
const test=base.extend<{}, {hosts:Hosts}>({hosts:[async({},use,info)=>{
  const binary=resolve(process.env.OH_SERVER_EXECUTABLE??'../target/debug/oh_server');
  const output=resolve(info.project.outputDir,`training-hosts-${info.workerIndex}`);mkdirSync(output,{recursive:true});
  const root=mkdtempSync(resolve(tmpdir(),'openhoi-training-ui-'));cpSync(resolve('../data/packs/testland_m2_military'),resolve(root,'testland'),{recursive:true});
  const checkpoint=resolve(process.env.OH_M2_MILITARY_OUTPUT_DIR??'../target/wp23/checkpoint');
  const children:ChildProcess[]=[],records:Array<{pid:number|undefined;exitCode:number|null;alive:boolean}>=[];
  const ledger=()=>writeFileSync(resolve(output,'pid-ledger.json'),JSON.stringify(records,null,2));
  const start=async(save?:string)=>{
    const port=await freePort(),args=['--port',String(port),'--pack-root',root,'--scenario','m2_military',...(save?['--load-save',save]:[])];
    const child=spawn(binary,args,{cwd:resolve('..'),stdio:['ignore','pipe','pipe']});children.push(child);
    const row={pid:child.pid,exitCode:null as number|null,alive:true};records.push(row);ledger();
    child.stdout!.pipe(createWriteStream(resolve(output,`server-${port}.stdout`)));child.stderr!.pipe(createWriteStream(resolve(output,`server-${port}.stderr`)));
    child.on('exit',code=>{row.exitCode=code;row.alive=false;ledger();});
    const url=`http://127.0.0.1:${port}`;
    await expect.poll(async()=>{if(child.exitCode!==null)throw Error('host exited');try{return(await fetch(url)).status;}catch{return 0;}}).toBe(200);
    return url;
  };
  try{await use({fresh:await start(),restored:await start(resolve(checkpoint,'paused-training.ohsave'))});}
  finally{
    for(const child of children){if(child.exitCode!==null||child.signalCode!==null)continue;const exited=new Promise<void>(done=>child.once('exit',()=>done()));child.kill('SIGINT');const timer=setTimeout(()=>child.kill('SIGKILL'),5000);try{await exited;}finally{clearTimeout(timer);}}
    for(const row of records){if(row.pid){try{process.kill(row.pid,0);row.alive=true;}catch{row.alive=false;}}expect(row.alive).toBe(false);expect(row.exitCode).toBe(0);}ledger();
  }
},{scope:'worker'}]});
function observe(page:Page){const sent:ClientMessage[]=[],received:ServerMessage[]=[];page.on('websocket',socket=>{socket.on('framesent',e=>{if(e.payload instanceof Buffer)sent.push(decode(e.payload) as ClientMessage);});socket.on('framereceived',e=>{if(e.payload instanceof Buffer)received.push(decode(e.payload) as ServerMessage);});});return {sent,received};}
async function player(page:Page,url:string){await page.goto(url);await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();await expect(page.getByTestId('production-panel')).toBeVisible();await page.getByRole('button',{name:'Open military information',exact:true}).click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');}
test('native buttons start and cancel authoritative Pending training',async({page,hosts},info)=>{
  const wire=observe(page);await player(page,hosts.fresh);
  const html=await(await fetch(hosts.fresh)).text(),script=html.match(/<script[^>]*src="([^"]+)"/)![1];expect(Buffer.from(await(await fetch(hosts.fresh+script)).arrayBuffer())).toEqual(readFileSync(resolve('dist',script.slice(1))));
  const train=page.locator('[data-military-template="m2_small"]').getByRole('button',{name:'Start training',exact:true});
  await expect(train).toBeEnabled();await train.click();
  const job=page.locator('[data-military-job="0"]');await expect(job).toContainText('Pending');
  await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
  await job.getByRole('button',{name:'Cancel training',exact:true}).click();await expect(job).toContainText('Cancelled');
  await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
  expect(wire.sent.filter(m=>m.type==='MilitaryCommand').map(m=>m.type==='MilitaryCommand'?m.command.type:'')).toEqual(['Train','Cancel']);
  writeFileSync(info.outputPath('native-training-ui-wire.json'),JSON.stringify(wire,null,2));
});

function latest<K extends ServerMessage['type']>(wire:ReturnType<typeof observe>,type:K){return wire.received.filter(m=>m.type===type).at(-1) as Extract<ServerMessage,{type:K}>;}
test('native restored Training cancel returns eight manpower and six equipment; foreign and terminal controls cannot send',async({page,hosts},info)=>{
  const wire=observe(page);await player(page,hosts.restored);
  const before=latest(wire,'EconomyResult').economy!.nations.find(n=>n.nation===1)!;
  const beforeStock=latest(wire,'ProductionResult').production!.nations.find(n=>n.nation===1)!.stock.find(s=>s.model==='m2_equipment_1')!.available;
  const own=page.locator('[data-military-job="0"]'),foreign=page.locator('[data-military-job="1"]');
  await expect(own).toContainText('Training');await expect(foreign.getByRole('button',{name:'Cancel training',exact:true})).toBeDisabled();
  await own.getByRole('button',{name:'Cancel training',exact:true}).click();await expect(own).toContainText('Cancelled');
  await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
  await expect(own.getByRole('button',{name:'Cancel training',exact:true})).toHaveCount(0);
  await expect.poll(()=>latest(wire,'EconomyResult').economy!.nations.find(n=>n.nation===1)!.reserved).toBe((BigInt(before.reserved)-8n).toString());
  const after=latest(wire,'EconomyResult').economy!.nations.find(n=>n.nation===1)!;expect(BigInt(after.available)).toBe(BigInt(before.available)+8n);expect(after.committed).toBe(before.committed);
  await expect.poll(()=>latest(wire,'ProductionResult').production!.nations.find(n=>n.nation===1)!.stock.find(s=>s.model==='m2_equipment_1')!.available).toBe((BigInt(beforeStock)+6n).toString());
  const military=latest(wire,'MilitaryResult').military!;expect(military.jobs[0].reserved_manpower).toBe('0');expect(military.jobs[0].equipment).toEqual([]);expect(military.jobs[1].reserved_manpower).toBe('8');
  expect(wire.sent.filter(m=>m.type==='MilitaryCommand')).toHaveLength(1);writeFileSync(info.outputPath('native-cancel-conservation.json'),JSON.stringify({before,after,wire},null,2));
});
test('spectator and foreign display filter cannot authorize; own filter restores fresh authority',async({page,hosts})=>{
  const wire=observe(page);await page.goto(hosts.restored);await page.getByRole('button',{name:'Open military information',exact:true}).click();
  const panel=page.getByTestId('military-panel');await expect(panel).toHaveAttribute('data-status','ready');
  await expect(panel.getByRole('button',{name:'Start training',exact:true})).toBeDisabled();expect(wire.sent.some(m=>m.type==='MilitaryCommand')).toBe(false);
  await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();await expect(panel.getByRole('button',{name:'Start training',exact:true})).toBeEnabled();
  await panel.locator('select').selectOption('2');await expect(panel).toHaveAttribute('data-status','ready');await expect(panel.getByRole('button',{name:'Start training',exact:true})).toBeDisabled();
  await expect(panel.getByRole('button',{name:'Cancel training',exact:true})).toBeDisabled();
  await panel.locator('select').selectOption('1');await expect(panel.getByRole('button',{name:'Start training',exact:true})).toBeEnabled();expect(wire.sent.some(m=>m.type==='MilitaryCommand')).toBe(false);
});
test('real CONNECTING upgrade holds revoke command authority until fresh joined military data',async({page,hosts})=>{
  const gate=await upgradeGate(hosts.fresh),wire=observe(page);
  try{await page.goto(gate.url);await page.getByRole('button',{name:'Open military information',exact:true}).click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
    gate.hold();await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();await expect.poll(()=>gate.count()).toBe(1);
    await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','loading');expect(wire.sent.some(m=>m.type==='MilitaryCommand')).toBe(false);
    gate.release();const train=page.getByRole('button',{name:'Start training',exact:true});await expect(train).toBeEnabled();await train.click();await expect(page.locator('[data-military-job="0"]')).toContainText('Pending');
  }finally{await gate.close();}
});
async function bridge(page:Page){
  const heldBases:Array<{send:(frame:Buffer)=>void;frame:Buffer}>=[],heldResults:Array<{send:(frame:Buffer)=>void;frame:Buffer}>=[];let hold=false,disconnect=()=>{},inject=(_m:ServerMessage)=>{};
  await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();disconnect=()=>server.close({code:1000,reason:'test-close'});inject=m=>route.send(Buffer.from(encode(m)));
    route.onMessage(frame=>server.send(frame));server.onMessage(frame=>{const m=decode(frame as Uint8Array) as ServerMessage;const row={send:(f:Buffer)=>route.send(f),frame:Buffer.from(frame as Uint8Array)};
      if(hold&&m.type==='MilitaryResult'){heldBases.push(row);return;}if(hold&&m.type==='CommandResult'){heldResults.push(row);return;}route.send(frame);
    });});
  return {heldBases,heldResults,hold:()=>{hold=true;},disconnect:()=>disconnect(),inject:(m:ServerMessage)=>inject(m)};
}
const releaseFrame=(row:{send:(frame:Buffer)=>void;frame:Buffer})=>row.send(row.frame);
test('injected timing: ACK barrier discards pre-command view, duplicate clicks and unissued/duplicate results',async({page,hosts})=>{
  const proxy=await bridge(page),wire=observe(page);await player(page,hosts.fresh);proxy.hold();await expect.poll(()=>proxy.heldBases.length).toBe(1);
  const train=page.getByRole('button',{name:'Start training',exact:true});await train.evaluate(el=>{(el as HTMLButtonElement).click();(el as HTMLButtonElement).click();});
  const feedback=page.getByTestId('military-command-feedback');await expect(feedback).toHaveAttribute('data-status','pending');await expect.poll(()=>proxy.heldResults.length).toBe(1);
  expect(wire.sent.filter(m=>m.type==='MilitaryCommand')).toHaveLength(1);proxy.inject({type:'CommandResult',sequence:'999999',accepted:true,reason_key:null});await expect(feedback).toHaveAttribute('data-status','pending');
  const ack=proxy.heldResults.shift()!;releaseFrame(ack);await expect(feedback).toHaveAttribute('data-status','refreshing');
  releaseFrame(proxy.heldBases.shift()!);await expect.poll(()=>proxy.heldBases.length).toBe(1);await expect(feedback).toHaveAttribute('data-status','refreshing');await expect(train).toBeDisabled();
  releaseFrame(proxy.heldBases.shift()!);await expect(feedback).toHaveAttribute('data-status','success');await expect(train).toBeEnabled();await expect(page.locator('[data-military-job="0"]')).toContainText('Pending');
  releaseFrame(ack);await expect(feedback).toHaveAttribute('data-status','success');expect(wire.sent.filter(m=>m.type==='MilitaryCommand')).toHaveLength(1);
});
test('injected close: stale commands disabled; manual reconnect needs fresh scope and never retries previous command',async({page,hosts})=>{
  const proxy=await bridge(page),wire=observe(page);await player(page,hosts.fresh);const train=page.getByRole('button',{name:'Start training',exact:true});await train.click();
  await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');proxy.disconnect();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','stale');await expect(train).toBeDisabled();
  await page.getByRole('button',{name:'Reconnect',exact:true}).click();await expect(train).toBeEnabled();await expect(page.locator('[data-military-job]')).toHaveCount(0);
  expect(wire.sent.filter(m=>m.type==='MilitaryCommand')).toHaveLength(1);await train.click();await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');expect(wire.sent.filter(m=>m.type==='MilitaryCommand')).toHaveLength(2);
});
test('native time controls progress Training to Ready; ko/en keyboard and mobile cancel remains authoritative',async({page,hosts},info)=>{
  const wire=observe(page);await player(page,hosts.restored);await page.getByTestId('speed-3').click();await page.getByTestId('pause').click();
  const job=page.locator('[data-military-job="0"]');await expect(job).toContainText('Ready',{timeout:15000});await page.getByTestId('pause').click();
  await page.getByRole('combobox',{name:'Language',exact:true}).selectOption('ko');await page.setViewportSize({width:390,height:844});
  const cancel=job.getByRole('button',{name:'훈련 취소',exact:true});await cancel.focus();await page.keyboard.press('Enter');await expect(job).toContainText('취소');
  await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
  const panel=page.getByTestId('military-panel');expect(await panel.evaluate(el=>el.scrollWidth<=el.clientWidth)).toBe(true);await page.screenshot({path:info.outputPath('training-mobile-ko.png'),fullPage:true});
  await page.getByRole('combobox',{name:'언어',exact:true}).selectOption('en');await expect(panel).toContainText('Training command completed');expect(wire.sent.filter(m=>m.type==='MilitaryCommand')).toHaveLength(1);
});
test('actual pre-ledger readonly client bundle on current server: static proxy, native WebSocket frames',async({page,hosts},info)=>{
  const directory=process.env.OH_LEGACY_CLIENT_DIST;expect(directory,'actual legacy bundle is required').toBeTruthy();const root=resolve(directory!);
  const proxy=await upgradeGate(hosts.restored,root);
  try{
    const script=readFileSync(resolve(root,'index.html'),'utf8').match(/<script[^>]*src="([^"]+)"/)![1],scriptResponse=page.waitForResponse(r=>new URL(r.url()).pathname===script);
    const wire=observe(page);await page.goto(proxy.url);expect(await(await scriptResponse).body()).toEqual(readFileSync(resolve(root,script.slice(1))));await page.getByRole('button',{name:'Open military information',exact:true}).click();
    const panel=page.getByTestId('military-panel');await expect(panel).toHaveAttribute('data-status','ready');await expect(panel.locator('[data-military-job]')).toHaveCount(6);await expect(panel.getByRole('button',{name:'Start training',exact:true})).toHaveCount(0);
    expect(wire.sent.some(m=>m.type==='MilitaryCommand')).toBe(false);expect(wire.received.some(m=>m.type==='MilitaryResult'&&m.supported)).toBe(true);writeFileSync(info.outputPath('actual-legacy-client-wire.json'),JSON.stringify(wire,null,2));
  }finally{await proxy.close();}
});

test('injected missing/unsupported base removes command authority until a fresh correlated native view',async({page,hosts})=>{
  const proxy=await bridge(page),wire=observe(page);await player(page,hosts.fresh);proxy.hold();await expect.poll(()=>proxy.heldBases.length).toBe(1);
  const old=decode(proxy.heldBases.shift()!.frame) as Extract<ServerMessage,{type:'MilitaryResult'}>;
  proxy.inject({type:'MilitaryResult',request:old.request,supported:false,reason_key:'unsupported-query',military:null});await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','unsupported');
  expect(wire.sent.some(m=>m.type==='MilitaryCommand')).toBe(false);await expect.poll(()=>proxy.heldBases.length).toBe(1);releaseFrame(proxy.heldBases.shift()!);
  await expect(page.getByRole('button',{name:'Start training',exact:true})).toBeEnabled();expect(wire.sent.some(m=>m.type==='MilitaryCommand')).toBe(false);
});
