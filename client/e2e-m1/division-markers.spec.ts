import {test as base,expect,type Page} from '@playwright/test';
import {spawn} from 'node:child_process';
import {createServer} from 'node:net';
import {mkdtempSync,cpSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {resolve} from 'node:path';
import {decode,encode} from '@msgpack/msgpack';
import type {ServerMessage,ClientMessage} from '../src/proto/protocol';
import {decodeIndex,pick} from '../src/map/model';
const test=base.extend<{host:string}>({host:async({},use,info)=>{
 const root=mkdtempSync(resolve(tmpdir(),'openhoi-markers-'));cpSync(resolve('../data/packs/testland_m2_military'),resolve(root,'testland'),{recursive:true});
 const listener=createServer();await new Promise<void>(done=>listener.listen(0,'127.0.0.1',done));const port=(listener.address() as {port:number}).port;await new Promise<void>(done=>listener.close(()=>done()));
 const child=spawn(resolve('../target/debug',process.platform==='win32'?'oh_server.exe':'oh_server'),['--port',String(port),'--pack-root',root,'--scenario','m2_military'],{cwd:resolve('..'),stdio:'ignore'});
 const exited=new Promise<{code:number|null;signal:string|null}>(done=>child.once('exit',(code,signal)=>done({code,signal})));
 try{const url=`http://127.0.0.1:${port}`;await expect.poll(async()=>{if(child.exitCode!==null)throw Error('host exited');try{return(await fetch(url)).status;}catch{return 0;}}).toBe(200);await use(url);}
 finally{
  if(child.exitCode===null&&child.signalCode===null)child.kill('SIGINT');const timer=setTimeout(()=>child.kill('SIGKILL'),5000);
  let result;try{result=await exited;}finally{clearTimeout(timer);}
  let alive=false;try{if(child.pid){process.kill(child.pid,0);alive=true;}}catch{}
  writeFileSync(info.outputPath('marker-host-cleanup.json'),JSON.stringify({pid:child.pid,...result,alive}));rmSync(root,{recursive:true});expect(alive).toBe(false);expect(result.code).toBe(0);
 }
}});
function observe(page:Page){const sent:ClientMessage[]=[],received:ServerMessage[]=[];page.on('websocket',socket=>{socket.on('framesent',e=>{if(e.payload instanceof Buffer)sent.push(decode(e.payload) as ClientMessage);});socket.on('framereceived',e=>{if(e.payload instanceof Buffer)received.push(decode(e.payload) as ServerMessage);});});return {sent,received};}
async function trainAndDeploy(page:Page,wire:ReturnType<typeof observe>){
 await page.getByRole('button',{name:'Start training',exact:true}).click();await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
 await page.getByTestId('speed-3').click();if(await page.getByTestId('pause').textContent()==='Resume')await page.getByTestId('pause').click();
 const job=page.locator('[data-military-job="0"]');await expect(job).toContainText('Ready',{timeout:15000});await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
 const military=wire.received.filter((m):m is Extract<ServerMessage,{type:'MilitaryResult'}>=>m.type==='MilitaryResult'&&!!m.military).at(-1)!.military!;
 const world=wire.received.filter((m):m is Extract<ServerMessage,{type:'WorldResult'}>=>m.type==='WorldResult'&&m.request.startsWith('deploy-world:')&&!!m.world).at(-1)!.world!;
 const army=military.armies.find(a=>a.nation===1)!.id,province=world.provinces.find(p=>p.controller===1&&p.state!==null)!.id;
 await job.getByLabel('Deployment army',{exact:true}).selectOption(army);await job.getByLabel('Deployment location',{exact:true}).selectOption(String(province));await job.getByRole('button',{name:'Deploy',exact:true}).click();
 await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');return {province,world};
}
test('native Deploy marker stays inside its province through pan zoom reset resize, keyboard ko/en and close',async({page,host},info)=>{
 const wire=observe(page);await page.goto(host+'/?forceWebGL=1');await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();
 await expect(page.locator('.division-marker')).toHaveCount(0);await page.getByRole('button',{name:'Open military information',exact:true}).click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');await expect(page.locator('.division-marker')).toHaveCount(0);
 const {province,world}=await trainAndDeploy(page,wire),marker=page.locator(`[data-division-province="${province}"][data-division-nation="1"]`),map=page.getByTestId('province-map');
 await expect(marker).toHaveCount(1);await expect(marker).toHaveText('1');await expect(marker).toHaveAccessibleName(`Province ${province}, nation 1: 1 divisions`);
 const meta=await(await fetch(`${host}/maps/${world.map_id}/metadata`)).json() as {pack_hash:string};
 const index=decodeIndex(await(await fetch(`${host}/maps/${world.map_id}/index.bin?pack=${encodeURIComponent(meta.pack_hash)}`)).arrayBuffer(),world.width,world.height,world.province_ids);
 const follows=async()=>{await expect.poll(async()=>{
  const geometry=await marker.evaluate(el=>{const host=document.querySelector('[data-testid="province-map"]') as HTMLElement,rect=host.getBoundingClientRect(),box=el.getBoundingClientRect(),camera=JSON.parse(host.dataset.camera!);return {x:camera.x+((box.x+box.width/2-rect.x)/rect.width-.5)*camera.width,y:camera.y+((box.y+box.height/2-rect.y)/rect.height-.5)*camera.height,anchorX:Number((el as HTMLElement).dataset.mapX),anchorY:Number((el as HTMLElement).dataset.mapY)};});
  return Math.abs(geometry.x-geometry.anchorX)<.05&&Math.abs(geometry.y-geometry.anchorY)<.05&&pick(index,world.province_ids,world.width,world.height,geometry.x,geometry.y)===province;
 }).toBe(true);};
 await follows();await marker.click();await expect(page.getByTestId('selected-province')).toHaveText(String(province));
 const before=await map.getAttribute('data-camera'),rect=(await map.boundingBox())!;await page.mouse.move(rect.x+20,rect.y+20);await page.mouse.down();await page.mouse.move(rect.x+35,rect.y+30,{steps:4});await page.mouse.up();await expect.poll(()=>map.getAttribute('data-camera')).not.toBe(before);await follows();
 await marker.hover();await page.mouse.wheel(0,-100);await follows();await page.getByRole('button',{name:'Reset view',exact:true}).click();await follows();
 await page.setViewportSize({width:1000,height:900});await follows();await page.getByLabel('Language',{exact:true}).selectOption('ko');await expect(marker).toHaveAccessibleName(`프로빈스 ${province}, 국가 1: 사단 1개`);
 await page.getByRole('button',{name:'선택 해제',exact:true}).click();await marker.focus();await page.keyboard.press('Enter');await expect(page.getByTestId('selected-province')).toHaveText(String(province));await page.getByLabel('언어',{exact:true}).selectOption('en');
 await page.getByRole('button',{name:'Close military information',exact:true}).click();await expect(page.locator('.division-marker')).toHaveCount(0);
 const queries=wire.sent.filter(m=>m.type==='Query'&&m.kind==='military').length;await page.getByTestId('speed-1').click();await expect(page.getByTestId('speed')).toHaveText('1');expect(wire.sent.filter(m=>m.type==='Query'&&m.kind==='military')).toHaveLength(queries);
 writeFileSync(info.outputPath('native-marker-wire.json'),JSON.stringify(wire,null,2));
});
test('injected unsupported/unknown province and real disconnect hide markers without silent bad data',async({page,host})=>{
 let mode:'native'|'unknown'|'unsupported'='native',disconnect=()=>{};
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();disconnect=()=>server.close({code:1000,reason:'marker-test'});route.onMessage(frame=>server.send(frame));server.onMessage(frame=>{
  const m=decode(frame as Uint8Array) as ServerMessage;
  if(m.type==='MilitaryResult'&&m.military){
   if(mode==='unknown')m.military.divisions=m.military.divisions.map(d=>({...d,province:65535}));
   if(mode==='unsupported'){m.supported=false;m.military=null;m.reason_key='unsupported-query';}
   route.send(Buffer.from(encode(m)));
  }else route.send(frame);
 });});
 const wire=observe(page);await page.goto(host+'/?forceWebGL=1');await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();await page.getByRole('button',{name:'Open military information',exact:true}).click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');await trainAndDeploy(page,wire);await expect(page.locator('.division-marker')).toHaveCount(1);
 mode='unknown';await page.getByTestId('speed-1').click();await expect(page.getByRole('alert').filter({hasText:'Map data'})).toBeVisible();await expect(page.locator('.division-marker')).toHaveCount(0);
 mode='unsupported';await page.getByTestId('speed-2').click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','unsupported');await expect(page.locator('.division-marker')).toHaveCount(0);
 mode='native';await page.getByTestId('speed-3').click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');await expect(page.locator('.division-marker')).toHaveCount(1);disconnect();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','stale');await expect(page.locator('.division-marker')).toHaveCount(0);
});
