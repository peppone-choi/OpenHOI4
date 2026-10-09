import {test as base,expect,type Page,type WebSocketRoute} from '@playwright/test';
import {spawn,type ChildProcess} from 'node:child_process';
import {createServer} from 'node:net';
import {cpSync,mkdirSync,mkdtempSync,readFileSync,writeFileSync,createWriteStream} from 'node:fs';
import {tmpdir} from 'node:os';
import {resolve} from 'node:path';
import {encode,decode} from '@msgpack/msgpack';
import type {ServerMessage,MilitaryCommand} from '../src/proto/protocol';
import {upgradeGate} from './support/upgradeGate';

type Reply=Extract<ServerMessage,{type:'MilitaryResult'}>;
type Hosts={active:string;legacy:string;fixture:string};
async function freePort(){const server=createServer();await new Promise<void>(done=>server.listen(0,'127.0.0.1',done));const address=server.address();if(!address||typeof address==='string')throw Error('port');const port=address.port;await new Promise<void>(done=>server.close(()=>done()));return port;}
const test=base.extend<{}, {hosts:Hosts}>({hosts:[async({},use,info)=>{
  const fixture=resolve(process.env.OH_MILITARY_FIXTURE_ROOT??'../target/wp22/military-ready');
  const pack=JSON.parse(readFileSync(resolve(fixture,'expected.json'),'utf8')).pack as string;
  const binary=resolve(process.env.OH_SERVER_EXECUTABLE??'../target/debug/oh_server');
  const output=resolve(info.project.outputDir,`military-hosts-${info.workerIndex}`);mkdirSync(output,{recursive:true});
  const legacy=mkdtempSync(resolve(tmpdir(),'openhoi-wp22-legacy-'));cpSync(resolve('../data/packs/testland_m2'),resolve(legacy,'testland'),{recursive:true});
  const processes:ChildProcess[]=[];
  const records:Array<{pid:number|undefined;args:string[];exitCode:number|null;signal:NodeJS.Signals|null;alive:boolean}>=[];
  const ledger=()=>writeFileSync(resolve(output,'pid-ledger.json'),JSON.stringify({owner:process.pid,records},null,2));
  const start=async(root:string,scenario:string,save?:string)=>{
    const port=await freePort(),args=['--port',String(port),'--pack-root',root,'--scenario',scenario,...(save?['--load-save',save]:[])];
    const child=spawn(binary,args,{cwd:resolve('..'),stdio:['ignore','pipe','pipe']});processes.push(child);
    const row={pid:child.pid,args,exitCode:null as number|null,signal:null as NodeJS.Signals|null,alive:true};records.push(row);ledger();
    child.stdout!.pipe(createWriteStream(resolve(output,`server-${port}.stdout`)));child.stderr!.pipe(createWriteStream(resolve(output,`server-${port}.stderr`)));
    child.on('exit',(code,signal)=>{row.exitCode=code;row.signal=signal;row.alive=false;ledger();});
    const url=`http://127.0.0.1:${port}`;
    await expect.poll(async()=>{if(child.exitCode!==null)throw Error('host exited');try{return (await fetch(url)).status;}catch{return 0;}}).toBe(200);
    return url;
  };
  try{
    const active=await start(resolve(pack,'..'),'m1',resolve(fixture,'ready.ohsave'));
    const old=await start(legacy,'m2_initial');
    await use({active,legacy:old,fixture});
  }finally{
    for(const child of processes){
      if(child.exitCode!==null||child.signalCode!==null)continue;
      const exited=new Promise<void>(done=>child.once('exit',()=>done()));child.kill('SIGINT');
      const timer=setTimeout(()=>child.kill('SIGKILL'),5000);try{await exited;}finally{clearTimeout(timer);}
    }
    for(const row of records){if(row.pid){let alive=true;try{process.kill(row.pid,0);}catch{alive=false;}row.alive=alive;}expect(row.alive).toBe(false);expect(row.exitCode).toBe(0);}
    ledger();
  }
},{scope:'worker'}]});

async function observe(page:Page,hold=false){
  const requests:string[]=[],browserMessages:Array<{type:string;kind?:string;request?:string}>=[],held:Array<{route:WebSocketRoute;reply:Reply}>=[],replies:Reply[]=[],commandResults:Array<Extract<ServerMessage,{type:'CommandResult'}>>=[];
  let deltas=0,injection:(command:MilitaryCommand,sequence:string)=>void=()=>{throw Error('socket not ready');};
  let disconnect:()=>void=()=>{throw Error('socket not ready');};
  await page.routeWebSocket('**/ws',route=>{
    const server=route.connectToServer();
    injection=(command,sequence)=>server.send(Buffer.from(encode({type:'MilitaryCommand',sequence,command})));
    disconnect=()=>server.close({code:1000,reason:'test-disconnect'});
    route.onMessage(frame=>{const message=decode(frame as Uint8Array) as {type:string;kind?:string;request?:string};browserMessages.push(message);if(message.type==='Query'&&message.kind==='military')requests.push(message.request!);server.send(frame);});
    server.onMessage(frame=>{const message=decode(frame as Uint8Array) as ServerMessage;if(message.type==='Delta')deltas++;if(message.type==='CommandResult')commandResults.push(message);if(message.type==='MilitaryResult'){replies.push(message);if(hold){held.push({route,reply:message});return;}}route.send(frame);});
  });
  return {requests,browserMessages,held,replies,commandResults,deltaCount:()=>deltas,command:(command:MilitaryCommand,sequence:string)=>injection(command,sequence),disconnect:()=>disconnect()};
}
const send=(row:{route:WebSocketRoute;reply:Reply},reply=row.reply)=>row.route.send(Buffer.from(encode(reply)));
async function open(page:Page){await page.getByRole('button',{name:'Open military information',exact:true}).click();}

test('actual military host: keyboard open/close, no closed queries or UI commands, local filter and mobile',async({page,hosts},info)=>{
  const wire=await observe(page);await page.goto(hosts.active);
  await expect(page.getByTestId('production-panel')).toBeVisible();
  expect(wire.requests).toHaveLength(0);
  const toggle=page.getByRole('button',{name:'Open military information',exact:true});await toggle.focus();await page.keyboard.press('Enter');
  const panel=page.getByTestId('military-panel');await expect(panel).toHaveAttribute('data-status','ready');
  await expect(panel.getByTestId('military-templates')).toContainText('example');await expect(panel.getByTestId('military-armies').locator('article')).toHaveCount(2);
  const joins=wire.browserMessages.filter(message=>message.type==='Join').length;
  await panel.getByRole('combobox').selectOption('2');await expect(panel.getByTestId('military-armies').locator('article')).toHaveCount(1);
  expect(wire.browserMessages.filter(message=>message.type==='Join')).toHaveLength(joins);
  await page.getByRole('combobox',{name:'Language',exact:true}).selectOption('ko');await expect(panel).toContainText('군사 정보');
  await page.setViewportSize({width:390,height:844});await panel.scrollIntoViewIfNeeded();
  expect(await panel.evaluate(element=>element.scrollWidth<=element.clientWidth)).toBe(true);
  await page.screenshot({path:info.outputPath('military-mobile-ko.png'),fullPage:true});
  await panel.getByRole('button',{name:'군사 정보 닫기',exact:true}).focus();await page.keyboard.press('Enter');await expect(panel).toHaveCount(0);
  await expect(page.getByRole('button',{name:'군사 정보 열기',exact:true})).toBeFocused();
  const count=wire.requests.length,deltas=wire.deltaCount();await expect.poll(()=>wire.deltaCount()).toBeGreaterThan(deltas);
  expect(wire.requests).toHaveLength(count);expect(wire.browserMessages.filter(message=>message.type==='MilitaryCommand')).toHaveLength(0);
});

test('unreplaced native restored Ready and actual raw Train/Cancel/Deploy projections',async({page,hosts,request},info)=>{
  const wire=await observe(page);await page.goto(hosts.active);await open(page);
  const panel=page.getByTestId('military-panel');await expect(panel).toHaveAttribute('data-status','ready');
  const html=await (await request.get(hosts.active)).text(),script=html.match(/<script[^>]*src="([^"]+)"/)![1];
  expect(Buffer.from(await (await request.get(hosts.active+script)).body())).toEqual(readFileSync(resolve('dist',script.slice(1))));
  await expect(panel.locator('[data-military-job="0"]')).toContainText('Ready');
  wire.command({type:'Train',template:'example'},'9000');
  await expect.poll(()=>wire.commandResults.find(row=>row.sequence==='9000')?.reason_key).toBe('unsupported-session');
  // Raw commands use this browser's real session. Spectator queries are valid,
  // but command authority first requires the existing native Join/control UI.
  await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();
  await expect.poll(()=>wire.browserMessages.filter(message=>message.type==='Join').length).toBe(2);
  await expect(panel).toHaveAttribute('data-status','ready');
  wire.command({type:'Train',template:'example'},'9001');await expect.poll(()=>wire.commandResults.find(row=>row.sequence==='9001')?.accepted).toBe(true);
  await expect(panel.locator('[data-military-job="1"]')).toContainText('Pending');
  wire.command({type:'Cancel',job:'1'},'9002');await expect.poll(()=>wire.commandResults.find(row=>row.sequence==='9002')?.accepted).toBe(true);await expect(panel.locator('[data-military-job="1"]')).toContainText('Cancelled');
  wire.command({type:'Deploy',job:'0',army:'0',province:10,allow_understrength:true},'9003');await expect.poll(()=>wire.commandResults.find(row=>row.sequence==='9003')?.accepted).toBe(true);
  await expect(panel.locator('[data-military-job="0"]')).toContainText('Deployed');await expect(panel.locator('[data-military-division="0"]')).toBeVisible();
  expect(wire.browserMessages.filter(message=>message.type==='MilitaryCommand')).toHaveLength(0);
  writeFileSync(info.outputPath('native-replies.json'),JSON.stringify({requests:wire.requests,replies:wire.replies,commandResults:wire.commandResults},null,2));
});

test('delayed native reply survives continuous Delta; closed/open epochs discard old/future/duplicate replies',async({page,hosts})=>{
  const wire=await observe(page,true);await page.goto(hosts.active);await open(page);
  await expect.poll(()=>wire.held.length).toBe(1);const first=wire.held[0];
  const deltas=wire.deltaCount();await expect.poll(()=>wire.deltaCount()).toBeGreaterThan(deltas+2);expect(wire.requests).toHaveLength(1);
  send(first,{...first.reply,request:'military:999999'});await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','loading');
  send(first);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');await expect.poll(()=>wire.held.length).toBe(2);
  await page.getByTestId('military-panel').getByRole('button',{name:'Close military information',exact:true}).click();send(wire.held[1]);
  await open(page);await expect.poll(()=>wire.held.length).toBe(3);
  send(first);send(wire.held[1]);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','loading');
  send(wire.held[2]);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
});

test('real CONNECTING upgrade refusal retries after OPEN without phantom pending',async({page,hosts})=>{
  const gate=await upgradeGate(hosts.active);gate.hold();
  try{await page.goto(gate.url);await open(page);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','loading');await expect.poll(()=>gate.count()).toBe(1);gate.release();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');}
  finally{await page.close();await gate.close();}
});

test('disconnect marks verified data stale; reconnect clears details and old socket replies',async({page,hosts})=>{
  const wire=await observe(page,true);await page.goto(hosts.active);await open(page);await expect.poll(()=>wire.held.length).toBe(1);send(wire.held[0]);
  const panel=page.getByTestId('military-panel');await expect(panel).toHaveAttribute('data-status','ready');wire.disconnect();await expect(panel).toHaveAttribute('data-status','stale');
  const count=wire.held.length;await page.getByRole('button',{name:'Reconnect',exact:true}).click();await expect(panel.locator('[data-military-job]')).toHaveCount(0);await expect.poll(()=>wire.held.length).toBeGreaterThan(count);await expect(panel).toHaveAttribute('data-status','loading');
  const current=wire.held.at(-1)!;send(current);await expect(panel).toHaveAttribute('data-status','ready');
});

test('actual army-free M2 uses generic unsupported fallback and renders mobile state',async({page,hosts},info)=>{
  const frames:ServerMessage[]=[];page.on('websocket',socket=>socket.on('framereceived',event=>{if(event.payload instanceof Buffer)frames.push(decode(event.payload) as ServerMessage);}));
  await page.goto(hosts.legacy);await open(page);const panel=page.getByTestId('military-panel');await expect(panel).toHaveAttribute('data-status','unsupported');
  expect(frames.some(message=>message.type==='QueryResult'&&message.request.startsWith('military:')&&!message.supported)).toBe(true);
  await page.setViewportSize({width:390,height:844});await expect(panel).toContainText('unavailable');await page.screenshot({path:info.outputPath('military-unsupported.png'),fullPage:true});
});

test('actual control identity change clears previous detail and display filter before new reply',async({page,hosts})=>{
  const wire=await observe(page,true);await page.goto(hosts.active);await open(page);await expect.poll(()=>wire.held.length).toBe(1);send(wire.held[0]);
  const panel=page.getByTestId('military-panel');await expect(panel).toHaveAttribute('data-status','ready');await panel.getByRole('combobox').selectOption('2');
  const count=wire.held.length;await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();
  await expect(panel.locator('[data-military-job]')).toHaveCount(0);await expect.poll(()=>wire.held.length).toBeGreaterThan(count);await expect(panel).toHaveAttribute('data-status','loading');
  send(wire.held.at(-1)!);await expect(panel).toHaveAttribute('data-status','ready');await expect(panel.getByRole('combobox')).toHaveValue('');
  expect(wire.browserMessages.filter(message=>message.type==='Join')).toHaveLength(2);
});

test('malformed military response uses existing transport guard and preserves stale verified detail',async({page,hosts})=>{
  const wire=await observe(page,true);await page.goto(hosts.active);await open(page);await expect.poll(()=>wire.held.length).toBe(1);send(wire.held[0]);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
  const invalid=structuredClone(wire.held[0].reply) as any;delete invalid.military.jobs[0].reserved_manpower;send(wire.held[0],invalid);
  await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','stale');await expect(page.getByTestId('military-panel').locator('[data-military-job="0"]')).toBeVisible();
});
