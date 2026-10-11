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

function latest<K extends ServerMessage['type']>(wire:ReturnType<typeof observe>,type:K){return wire.received.filter(m=>m.type===type).at(-1) as Extract<ServerMessage,{type:K}>;}
async function ready(page:Page,job:string){await page.getByTestId('speed-3').click();await expect(page.getByTestId('speed')).toHaveText('3');if(await page.getByTestId('pause').textContent()==='Resume')await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Pause');await expect(page.locator(`[data-military-job="${job}"]`)).toContainText('Ready',{timeout:15000});await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');}
async function choose(page:Page,job:string,army:string,province:number,under=false){const card=page.locator(`[data-military-job="${job}"]`);await card.getByLabel('Deployment army',{exact:true}).selectOption(army);await card.getByLabel('Deployment location',{exact:true}).selectOption(String(province));await card.getByLabel('Allow understrength deployment',{exact:true}).setChecked(under);return card.getByRole('button',{name:'Deploy',exact:true});}
function inputs(wire:ReturnType<typeof observe>){const m=latest(wire,'MilitaryResult').military!,w=wire.received.filter(m=>m.type==='WorldResult'&&m.request.startsWith('deploy-world:')&&m.world).at(-1) as Extract<ServerMessage,{type:'WorldResult'}>;return {army:m.armies.find(a=>a.nation===1)!.id,province:w.world!.provinces.find(p=>p.controller===1&&p.state!==null)!.id};}
test('native Ready UI click deploys held resources exactly once with current served JS',async({page,hosts},info)=>{
 const wire=observe(page);await player(page,hosts.restored);await ready(page,'0');const card=page.locator('[data-military-job="0"]');
 await expect(card.getByRole('button',{name:'Deploy',exact:true})).toBeVisible();
 const html=await(await fetch(hosts.restored)).text(),script=html.match(/<script[^>]*src="([^" ]+)"/)![1];expect(Buffer.from(await(await fetch(hosts.restored+script)).arrayBuffer())).toEqual(readFileSync(resolve('dist',script.slice(1))));
 const before=latest(wire,'MilitaryResult').military!,job=before.jobs.find(j=>j.id==='0')!,economy=latest(wire,'EconomyResult').economy!.nations.find(n=>n.nation===1)!;
 const {army,province}=inputs(wire);const deploy=await choose(page,'0',army,province);await expect(deploy).toBeEnabled();await deploy.evaluate(el=>{(el as HTMLButtonElement).click();(el as HTMLButtonElement).click();});
 await expect(card).toContainText('Deployed');await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
 const after=latest(wire,'MilitaryResult').military!,done=after.jobs.find(j=>j.id==='0')!,division=after.divisions.find(d=>d.id===done.division)!;
 expect(division).toMatchObject({army,province,manpower:job.reserved_manpower,equipment:job.equipment});expect(after.armies.find(a=>a.id===army)!.divisions).toContain(division.id);
 await expect.poll(()=>latest(wire,'EconomyResult').economy!.nations.find(n=>n.nation===1)!.committed).toBe((BigInt(economy.committed)+BigInt(job.reserved_manpower)).toString());expect(latest(wire,'EconomyResult').economy!.nations.find(n=>n.nation===1)!.reserved).toBe((BigInt(economy.reserved)-BigInt(job.reserved_manpower)).toString());expect(after.background).toEqual(before.background);
 expect(done.equipment).toEqual([]);expect(done.reserved_manpower).toBe('0');expect(wire.sent.filter(m=>m.type==='MilitaryCommand'&&m.command.type==='Deploy')).toHaveLength(1);await expect(card.getByRole('button',{name:'Deploy',exact:true})).toHaveCount(0);await expect(card.getByRole('button',{name:'Cancel training',exact:true})).toHaveCount(0);
 writeFileSync(info.outputPath('native-deploy-conservation.json'),JSON.stringify({before,after,wire},null,2));
});
test('native equipment zero requires explicit checkbox; false rejection preserves held manpower',async({page,hosts},info)=>{
 const wire=observe(page);await player(page,hosts.restored);
 await page.getByRole('button',{name:'Pause line',exact:true}).click();await expect(page.getByRole('button',{name:'Resume line',exact:true})).toBeVisible();
 for(let i=0;i<3;i++){await page.getByRole('button',{name:'Start training',exact:true}).click();await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');}
 await ready(page,'8');const {army,province}=inputs(wire);
 const before=latest(wire,'MilitaryResult').military!,job=before.jobs.find(j=>j.id==='8')!;expect(job.equipment.every(e=>e.count==='0')).toBe(true);
 await (await choose(page,'8',army,province)).click();await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','rejected');expect(latest(wire,'MilitaryResult').military!.jobs.find(j=>j.id==='8')!.reserved_manpower).toBe(job.reserved_manpower);expect(latest(wire,'MilitaryResult').military!.divisions).toHaveLength(0);
 await (await choose(page,'8',army,province,true)).click();await expect(page.locator('[data-military-job="8"]')).toContainText('Deployed');await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
 const commands=wire.sent.filter(m=>m.type==='MilitaryCommand'&&m.command.type==='Deploy');expect(commands.map(m=>m.type==='MilitaryCommand'&&m.command.type==='Deploy'?m.command.allow_understrength:null)).toEqual([false,true]);writeFileSync(info.outputPath('native-zero-equipment.json'),JSON.stringify(wire,null,2));
});

async function bridge(page:Page){
 const worlds:Array<{send:(frame:Buffer)=>void;frame:Buffer}>=[],bases:Array<{send:(frame:Buffer)=>void;frame:Buffer}>=[],acks:Array<{send:(frame:Buffer)=>void;frame:Buffer}>=[];
 let holdWorld=false,holdBase=false,holdAck=false,disconnect=()=>{},inject=(_m:ServerMessage)=>{};
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();disconnect=()=>server.close({code:1000,reason:'test-close'});inject=m=>route.send(Buffer.from(encode(m)));route.onMessage(frame=>server.send(frame));server.onMessage(frame=>{
  const m=decode(frame as Uint8Array) as ServerMessage,row={send:(f:Buffer)=>route.send(f),frame:Buffer.from(frame as Uint8Array)};
  if(holdWorld&&m.type==='WorldResult'&&m.request.startsWith('deploy-world:')){worlds.push(row);return;}
  if(holdBase&&m.type==='MilitaryResult'){bases.push(row);return;}if(holdAck&&m.type==='CommandResult'){acks.push(row);return;}route.send(frame);
 });});
 return {worlds,bases,acks,holdWorld:()=>{holdWorld=true;},holdAll:()=>{holdWorld=true;holdBase=true;holdAck=true;},disconnect:()=>disconnect(),inject:(m:ServerMessage)=>inject(m)};
}
const releaseFrame=(row:{send:(frame:Buffer)=>void;frame:Buffer})=>row.send(row.frame);
test('injected world timing: scope change clears drafts; retained/unissued/old/missing world never authorizes',async({page,hosts},info)=>{
 const proxy=await bridge(page),wire=observe(page);await player(page,hosts.restored);await ready(page,'0');const {army,province}=inputs(wire);await choose(page,'0',army,province,true);
 proxy.holdWorld();const panel=page.getByTestId('military-panel'),filter=panel.getByLabel('Displayed nation',{exact:true});await filter.selectOption('2');await expect.poll(()=>proxy.worlds.length).toBe(1);const old=proxy.worlds.shift()!;
 await filter.selectOption('1');await expect.poll(()=>proxy.worlds.length).toBe(1);await expect(panel).toHaveAttribute('data-status','ready');const card=page.locator('[data-military-job="0"]'),deploy=card.getByRole('button',{name:'Deploy',exact:true});
 await expect(card.getByLabel('Deployment army',{exact:true})).toHaveValue('');await expect(card.getByLabel('Deployment location',{exact:true})).toHaveValue('');await expect(card.getByLabel('Allow understrength deployment',{exact:true})).not.toBeChecked();await expect(deploy).toBeDisabled();
 releaseFrame(old);proxy.inject({...latest(wire,'WorldResult'),request:'world'});await expect(deploy).toBeDisabled();
 const current=proxy.worlds.shift()!,message=decode(current.frame) as Extract<ServerMessage,{type:'WorldResult'}>;proxy.inject({...message,world:null});await expect(deploy).toBeDisabled();releaseFrame(current);await expect(deploy).toBeDisabled();
 await filter.selectOption('');await expect.poll(()=>proxy.worlds.length).toBe(1);releaseFrame(proxy.worlds.shift()!);await (await choose(page,'0',army,province)).click();await expect(card).toContainText('Deployed');await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','refreshing');
 await expect.poll(()=>proxy.worlds.length).toBe(1);releaseFrame(proxy.worlds.shift()!);await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');writeFileSync(info.outputPath('injected-world-scope.json'),JSON.stringify({scope:'injected timing/missing responses; native command outcome',wire},null,2));
});
test('injected ACK timing: pre-ACK military and fresh world alone cannot clear Deploy pending',async({page,hosts},info)=>{
 const proxy=await bridge(page),wire=observe(page);await player(page,hosts.restored);await ready(page,'0');const {army,province}=inputs(wire);const deploy=await choose(page,'0',army,province);proxy.holdAll();
 await page.getByTestId('speed-1').click();await expect.poll(()=>proxy.acks.length).toBe(1);releaseFrame(proxy.acks.shift()!);await expect.poll(()=>proxy.bases.length).toBe(1);
 await deploy.click();await expect.poll(()=>proxy.acks.length).toBe(1);const ack=proxy.acks.shift()!;releaseFrame(ack);const feedback=page.getByTestId('military-command-feedback');await expect(feedback).toHaveAttribute('data-status','refreshing');
 await expect.poll(()=>proxy.worlds.length).toBe(1);releaseFrame(proxy.worlds.shift()!);await expect(feedback).toHaveAttribute('data-status','refreshing');releaseFrame(proxy.bases.shift()!);await expect.poll(()=>proxy.bases.length).toBe(1);await expect(feedback).toHaveAttribute('data-status','refreshing');
 releaseFrame(proxy.bases.shift()!);await expect(feedback).toHaveAttribute('data-status','success');releaseFrame(ack);expect(wire.sent.filter(m=>m.type==='MilitaryCommand'&&m.command.type==='Deploy')).toHaveLength(1);writeFileSync(info.outputPath('injected-dual-barrier.json'),JSON.stringify({scope:'injected ACK/base timing; native Deploy',wire},null,2));
});
test('native full army rejects third Ready job with no resource transfer',async({page,hosts},info)=>{
 const wire=observe(page);await player(page,hosts.restored);for(let i=0;i<2;i++){await page.getByRole('button',{name:'Start training',exact:true}).click();await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');}
 await ready(page,'7');const {army,province}=inputs(wire);for(const job of ['0','6']){await (await choose(page,job,army,province,true)).click();await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');}
 const before=latest(wire,'MilitaryResult').military!;await (await choose(page,'7',army,province,true)).click();await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','rejected');const after=latest(wire,'MilitaryResult').military!;
 expect(after.background).toEqual(before.background);expect(after.divisions).toEqual(before.divisions);expect(after.jobs.find(j=>j.id==='7')).toEqual(before.jobs.find(j=>j.id==='7'));expect(after.armies.find(a=>a.id===army)!.divisions).toHaveLength(2);writeFileSync(info.outputPath('native-full-army.json'),JSON.stringify({before,after,wire},null,2));
});
test('native ko/en keyboard mobile deployment and real disconnected filter preserve stale data without authority',async({page,hosts},info)=>{
 const gate=await upgradeGate(hosts.restored),wire=observe(page);let closed=false;
 try{await player(page,gate.url);await ready(page,'0');const {army,province}=inputs(wire);await page.setViewportSize({width:390,height:844});await page.getByLabel('Language').selectOption('ko');
 const card=page.locator('[data-military-job="0"]');await card.getByLabel('배치할 군',{exact:true}).selectOption(army);await card.getByLabel('배치 위치',{exact:true}).selectOption(String(province));await card.getByRole('button',{name:'배치',exact:true}).focus();await page.keyboard.press('Enter');await expect(card).toContainText('배치됨');await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.getByLabel('언어').selectOption('en');await gate.close();closed=true;const panel=page.getByTestId('military-panel');await expect(panel).toHaveAttribute('data-status','stale');await panel.getByLabel('Displayed nation',{exact:true}).selectOption('2');await expect(panel).toHaveAttribute('data-status','stale');
 const foreign=panel.locator('[data-military-job="1"]');await expect(foreign.getByRole('button',{name:'Deploy',exact:true})).toBeDisabled();await expect(foreign.getByLabel('Deployment army',{exact:true})).toBeDisabled();expect(wire.sent.filter(m=>m.type==='MilitaryCommand'&&m.command.type==='Deploy')).toHaveLength(1);writeFileSync(info.outputPath('native-ko-mobile-disconnected.json'),JSON.stringify(wire,null,2));
 }finally{if(!closed)await gate.close();}
});

test('native panel kept open through Join activates Deploy after Snapshot without changing filters',async({page,hosts},info)=>{
 const wire=observe(page);
 try{
  await page.goto(hosts.restored);await page.getByRole('button',{name:'Open military information',exact:true}).click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
  await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();await ready(page,'0');
  const card=page.locator('[data-military-job="0"]');await expect(card.getByLabel('Deployment army',{exact:true})).toBeEnabled();
  const {army,province}=inputs(wire),deploy=await choose(page,'0',army,province);await expect(deploy).toBeEnabled();
  const afterHello=wire.sent.slice(wire.sent.reduce((index,m,i)=>m.type==='Hello'?i:index,-1)),join=afterHello.findIndex(m=>m.type==='Join'),world=afterHello.findIndex(m=>m.type==='Query'&&m.request.startsWith('deploy-world:'));
  expect(join).toBeGreaterThanOrEqual(0);expect(world).toBeGreaterThan(join);expect(afterHello.some(m=>m.type==='Query'&&m.request==='deploy-world:0')).toBe(false);
  await deploy.click();await expect(card).toContainText('Deployed');await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
 }finally{writeFileSync(info.outputPath('native-open-panel-join-wire.json'),JSON.stringify({scope:'real Join with panel kept open; no filter manipulation or response injection',wire},null,2));}
});
test('native manual reconnect with panel kept open activates Deploy after Snapshot without changing filters',async({page,hosts},info)=>{
 const proxy=await bridge(page),wire=observe(page);
 try{
  await player(page,hosts.restored);await ready(page,'0');const {army,province}=inputs(wire);await choose(page,'0',army,province,true);
  proxy.disconnect();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','stale');await page.getByRole('button',{name:'Reconnect',exact:true}).click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');await ready(page,'0');
  const card=page.locator('[data-military-job="0"]');await expect(card.getByLabel('Deployment army',{exact:true})).toBeEnabled();await expect(card.getByLabel('Deployment army',{exact:true})).toHaveValue('');await expect(card.getByLabel('Allow understrength deployment',{exact:true})).not.toBeChecked();
  const current=inputs(wire),deploy=await choose(page,'0',current.army,current.province);await expect(deploy).toBeEnabled();
  const afterHello=wire.sent.slice(wire.sent.reduce((index,m,i)=>m.type==='Hello'?i:index,-1));expect(afterHello.findIndex(m=>m.type==='Query'&&m.request.startsWith('deploy-world:'))).toBeGreaterThan(afterHello.findIndex(m=>m.type==='Join'));
  await deploy.click();await expect(card).toContainText('Deployed');await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
 }finally{writeFileSync(info.outputPath('native-open-panel-reconnect-wire.json'),JSON.stringify({scope:'native session reset on manual reconnect; actual socket close, no filter manipulation',wire},null,2));}
});
