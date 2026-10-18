import {test as base,expect,type Page} from '@playwright/test';
import {spawn} from 'node:child_process';
import {createServer} from 'node:net';
import {mkdtempSync,cpSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {resolve} from 'node:path';
import {decode,encode} from '@msgpack/msgpack';
import type {ServerMessage,ClientMessage,WorldView} from '../src/proto/protocol';
import {provinceAnchors} from '../src/map/divisionMarkers';
import {decodeIndex,pick} from '../src/map/model';
const test=base.extend<{host:string;wire:ReturnType<typeof observe>}>({wire:async({page},use,info)=>{const wire=observe(page);try{await use(wire);}finally{writeFileSync(info.outputPath('movement-wire.json'),JSON.stringify(wire,null,2));}},host:async({},use,info)=>{
 const root=mkdtempSync(resolve(tmpdir(),'openhoi-movement-'));cpSync(resolve('../data/packs/testland_m2_military'),resolve(root,'testland'),{recursive:true});
 const listener=createServer();await new Promise<void>(done=>listener.listen(0,'127.0.0.1',done));const port=(listener.address() as {port:number}).port;await new Promise<void>(done=>listener.close(()=>done()));
 const child=spawn(resolve('../target/debug',process.platform==='win32'?'oh_server.exe':'oh_server'),['--port',String(port),'--pack-root',root,'--scenario','m2_military'],{cwd:resolve('..'),stdio:'ignore'});
 const exited=new Promise<{code:number|null;signal:string|null}>(done=>child.once('exit',(code,signal)=>done({code,signal})));
 try{const url=`http://127.0.0.1:${port}`;await expect.poll(async()=>{if(child.exitCode!==null)throw Error('host exited');try{return(await fetch(url)).status;}catch{return 0;}}).toBe(200);await use(url);}
 finally{
  if(child.exitCode===null&&child.signalCode===null)child.kill('SIGINT');const timer=setTimeout(()=>child.kill('SIGKILL'),5000);
  let result;try{result=await exited;}finally{clearTimeout(timer);}
  let alive=false;try{if(child.pid){process.kill(child.pid,0);alive=true;}}catch{}
  writeFileSync(info.outputPath('movement-host-cleanup.json'),JSON.stringify({pid:child.pid,...result,alive}));rmSync(root,{recursive:true});expect(alive).toBe(false);expect(result.code).toBe(0);
 }
}});
test.use({trace:'retain-on-failure'});
function observe(page:Page){const sent:ClientMessage[]=[],received:ServerMessage[]=[],events:Array<{direction:'sent'|'received';message:ClientMessage|ServerMessage}>=[];page.on('websocket',socket=>{socket.on('framesent',e=>{if(e.payload instanceof Buffer){const message=decode(e.payload) as ClientMessage;sent.push(message);events.push({direction:'sent',message});}});socket.on('framereceived',e=>{if(e.payload instanceof Buffer){const message=decode(e.payload) as ServerMessage;received.push(message);events.push({direction:'received',message});}});});return {sent,received,events};}
async function startAsNation(page:Page,host:string,wire:ReturnType<typeof observe>){
 await page.goto(host+'/?forceWebGL=1');
 // The production button exists before the initial map/font/economy layout is ready.
 // Confirm actual readiness and the new Join, not merely a completed pointer action.
 const map=page.getByTestId('province-map');
 await expect(map).toHaveAttribute('data-backend','webgl2');
 await expect.poll(async()=>Number(await map.getAttribute('data-frames'))).toBeGreaterThan(0);
 await expect(page.getByTestId('economy-panel')).toHaveAttribute('data-status','ready');
 await page.evaluate(()=>document.fonts.ready);
 const start=page.getByRole('button',{name:'Start as selected nation',exact:true});await start.click();
 await expect.poll(()=>wire.sent.filter(m=>m.type==='Join'&&m.nation==='N01').length).toBe(1);
 await expect(start).toHaveCount(0);await expect(page.getByTestId('connection')).toHaveText('Connected');
}
async function trainAndDeploy(page:Page,wire:ReturnType<typeof observe>){
 await page.getByRole('button',{name:'Start training',exact:true}).click();await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');
 await page.getByTestId('speed-3').click();if(await page.getByTestId('pause').textContent()==='Resume')await page.getByTestId('pause').click();
 const job=page.locator('[data-military-job="0"]');await expect(job).toContainText('Ready',{timeout:15000});await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
 const military=wire.received.filter((m):m is Extract<ServerMessage,{type:'MilitaryResult'}>=>m.type==='MilitaryResult'&&!!m.military).at(-1)!.military!;
 const world=wire.received.filter((m):m is Extract<ServerMessage,{type:'WorldResult'}>=>m.type==='WorldResult'&&m.request.startsWith('deploy-world:')&&!!m.world).at(-1)!.world!;
 const army=military.armies.find(a=>a.nation===1)!.id,province=world.provinces.find(p=>p.controller===1&&p.state!==null)!.id;
 await job.getByLabel('Deployment army',{exact:true}).selectOption(army);await job.getByLabel('Deployment location',{exact:true}).selectOption(String(province));// Fresh jobs reserve the fixture's initially empty stock; opt in explicitly.
 await job.getByLabel('Allow understrength deployment',{exact:true}).check();await job.getByRole('button',{name:'Deploy',exact:true}).click();
 await expect(page.getByTestId('military-command-feedback')).toHaveAttribute('data-status','success');return {province,world};
}

const commands=(wire:ReturnType<typeof observe>)=>wire.sent.filter((m):m is Extract<ClientMessage,{type:'MovementCommand'}>=>m.type==='MovementCommand');
const latest=(wire:ReturnType<typeof observe>)=>wire.received.filter((m):m is Extract<ServerMessage,{type:'MovementResult'}>=>m.type==='MovementResult'&&m.supported&&!!m.movement).at(-1)!.movement!;
async function prepared(page:Page,host:string,wire:ReturnType<typeof observe>){
 await startAsNation(page,host,wire);await page.getByRole('button',{name:'Open military information',exact:true}).click();await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');
 const {world}=await trainAndDeploy(page,wire);await expect(page.locator('.division-marker')).toHaveCount(1);await expect(page.getByRole('button',{name:'Choose movement destination',exact:true})).toBeVisible();
 const meta=await(await fetch(`${host}/maps/${world.map_id}/metadata`)).json() as {pack_hash:string};
 const index=decodeIndex(await(await fetch(`${host}/maps/${world.map_id}/index.bin?pack=${encodeURIComponent(meta.pack_hash)}`)).arrayBuffer(),world.width,world.height,world.province_ids);
 return {world,index,anchors:provinceAnchors(world,index)};
}
async function provinceClick(page:Page,world:WorldView,anchors:ReturnType<typeof provinceAnchors>,id:number,button:'left'|'right'='right'){
 const map=page.getByTestId('province-map');await map.scrollIntoViewIfNeeded();
 const box=(await map.boundingBox())!,camera=JSON.parse((await map.getAttribute('data-camera'))!) as {x:number;y:number;width:number;height:number},p=anchors.get(id)!;
 expect(world.province_ids).toContain(id);await page.mouse.click(box.x+(.5+(p.x-camera.x)/camera.width)*box.width,box.y+(.5+(p.y-camera.y)/camera.height)*box.height,{button});
}
async function settled(page:Page){await expect(page.getByTestId('movement-controls')).toHaveAttribute('data-busy','false');}
async function selectOwn(page:Page){await page.locator('.division-marker[data-division-nation="1"]').click();await expect(page.locator('.division-marker[data-division-nation="1"]')).toHaveAttribute('aria-pressed','true');}
async function runUntilProgress(page:Page,wire:ReturnType<typeof observe>){
 // Speed 5 is unbounded: the complete route can finish between observations.
 await page.getByTestId('speed-3').click();await expect(page.getByTestId('speed')).toHaveText('3');await page.getByTestId('pause').click();
 await expect.poll(()=>latest(wire).units.some(u=>u.route.length>0&&BigInt(u.elapsed.bits)>0n)).toBe(true);
 await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
}
async function assertRoute(page:Page,wire:ReturnType<typeof observe>,world:WorldView,index:Uint16Array){
 await expect.poll(async()=>{
  const units=latest(wire).units.filter(u=>u.nation===1&&u.route.length),lines=page.locator('[data-movement-division]');if(await lines.count()!==units.length)return false;
  for(const u of units){const line=page.locator(`[data-movement-division="${u.division}"]`);const points=JSON.parse((await line.getAttribute('data-map-points'))!) as Array<{x:number;y:number}>;
   if(JSON.stringify(points.map(p=>pick(index,world.province_ids,world.width,world.height,p.x,p.y)))!==JSON.stringify([u.province,...u.route]))return false;
   const follows=await line.evaluate(el=>{const svg=(el as SVGPolylineElement).ownerSVGElement!,poly=el as SVGPolylineElement,host=document.querySelector('[data-testid="province-map"]') as HTMLElement,camera=JSON.parse(host.dataset.camera!),box=host.getBoundingClientRect(),matrix=svg.getScreenCTM()!;return Array.from({length:poly.points.numberOfItems},(_,i)=>{const p=poly.points.getItem(i),screen=new DOMPoint(p.x,p.y).matrixTransform(matrix);return {x:camera.x+((screen.x-box.x)/box.width-.5)*camera.width,y:camera.y+((screen.y-box.y)/box.height-.5)*camera.height};});});
   if(!points.every((p,i)=>Math.abs(p.x-follows[i].x)<.05&&Math.abs(p.y-follows[i].y)<.05))return false;
  }return true;
 }).toBe(true);
}
test('native selection/right click, authoritative route through camera changes, tick positions and arrival',async({page,host,wire},info)=>{
 test.setTimeout(60000);const {world,index,anchors}=await prepared(page,host,wire);
 await provinceClick(page,world,anchors,14);expect(commands(wire)).toHaveLength(0);
 await selectOwn(page);expect(commands(wire)).toHaveLength(0);const selected=await page.getByTestId('selected-province').textContent();
 await provinceClick(page,world,anchors,14);await settled(page);await expect(page.locator('[data-movement-result]')).toHaveAttribute('data-status','accepted');
 expect(commands(wire).map(m=>m.command)).toEqual([{type:'Move',division:'0',destination:14}]);await expect(page.getByTestId('selected-province')).toHaveText(selected!);
 const sequence=commands(wire)[0].sequence,ack=wire.received.find(m=>m.type==='CommandResult'&&m.sequence===sequence);expect(ack).toMatchObject({accepted:true});
 const ackIndex=wire.events.findIndex(e=>e.direction==='received'&&e.message.type==='CommandResult'&&e.message.sequence===sequence);expect(ackIndex).toBeGreaterThan(-1);expect(wire.events.slice(ackIndex+1).some(e=>e.direction==='sent'&&e.message.type==='Query'&&e.message.kind==='movement')).toBe(true);
 await assertRoute(page,wire,world,index);
 const map=page.getByTestId('province-map');await map.scrollIntoViewIfNeeded();const rect=(await map.boundingBox())!;
 await page.mouse.move(rect.x+20,rect.y+20);await page.mouse.down();await page.mouse.move(rect.x+30,rect.y+28,{steps:4});await page.mouse.up();await assertRoute(page,wire,world,index);
 await map.hover();await page.mouse.wheel(0,-100);await assertRoute(page,wire,world,index);await page.getByRole('button',{name:'Reset view',exact:true}).click();await assertRoute(page,wire,world,index);
 await page.setViewportSize({width:1000,height:900});await assertRoute(page,wire,world,index);
 await runUntilProgress(page,wire);
 await expect.poll(async()=>{const u=latest(wire).units.find(u=>u.division==='0')!;if(!u.route.length)return false;const ratio=Number(BigInt(u.elapsed.bits)*1000000000n/BigInt(u.current_leg_hours.bits))/1000000000;return Math.abs(Number(await page.locator('[data-moving-division="0"]').getAttribute('data-ratio'))-ratio)<1e-9;}).toBe(true);
 const pausedTick=(await page.getByTestId('tick').textContent())!;await expect.poll(()=>latest(wire).tick).toBe(pausedTick);const frozen=await page.locator('[data-moving-division="0"]').getAttribute('data-ratio'),frame=Number(await map.getAttribute('data-frames'));await expect.poll(async()=>Number(await map.getAttribute('data-frames'))).toBeGreaterThan(frame+3);await expect(page.locator('[data-moving-division="0"]')).toHaveAttribute('data-ratio',frozen!);await page.screenshot({path:info.outputPath('server-route-paused.png'),fullPage:true});
 const before=latest(wire).units.find(u=>u.division==='0')!;expect(before.route.at(-1)).toBe(14);
 await page.getByTestId('pause').click();await expect.poll(()=>latest(wire).units.find(u=>u.division==='0')?.route.length,{timeout:20000}).toBe(0);await page.getByTestId('pause').click();
 await expect(page.locator('[data-movement-division]')).toHaveCount(0);await expect(page.locator('[data-moving-division]')).toHaveCount(0);await expect(page.locator('[data-division-province="14"][data-division-nation="1"]')).toHaveText('1');
 await expect.poll(()=>wire.received.filter((m):m is Extract<ServerMessage,{type:'MilitaryResult'}>=>m.type==='MilitaryResult'&&!!m.military).at(-1)!.military!.divisions.find(d=>d.id==='0')?.province).toBe(14);
 writeFileSync(info.outputPath('native-route-arrival.json'),JSON.stringify({before,arrived:latest(wire),wire},null,2));
});
test('native Stop retains one current leg, server rejection is generic in ko/en, no invented arrow',async({page,host,wire})=>{
 test.setTimeout(60000);const {world,anchors}=await prepared(page,host,wire);await selectOwn(page);await provinceClick(page,world,anchors,14);await settled(page);await runUntilProgress(page,wire);
 await page.getByRole('button',{name:'Stop selected divisions',exact:true}).click();await settled(page);
 await expect.poll(()=>latest(wire).units.find(u=>u.division==='0')?.route.length).toBe(1);await expect(page.locator('[data-movement-division="0"]')).toHaveAttribute('points',/^\S+ \S+$/);
 await page.getByTestId('pause').click();await expect.poll(()=>latest(wire).units.find(u=>u.division==='0')?.route.length,{timeout:20000}).toBe(0);await page.getByTestId('pause').click();await expect(page.locator('[data-movement-division]')).toHaveCount(0);
 await provinceClick(page,world,anchors,40);await settled(page);await expect(page.locator('[data-movement-result="0"]')).toHaveAttribute('data-status','rejected');await expect(page.locator('[data-movement-result="0"]')).toContainText('movement rejected');
 const rejected=commands(wire).at(-1)!;expect(wire.received.find(m=>m.type==='CommandResult'&&m.sequence===rejected.sequence)).toMatchObject({accepted:false,reason_key:'invalid-message'});await expect(page.locator('[data-movement-division]')).toHaveCount(0);
 await page.getByRole('combobox',{name:'Language',exact:true}).selectOption('ko');await expect(page.locator('[data-movement-result="0"]')).toContainText('이동 거부됨');await expect(page.getByRole('button',{name:'선택 사단 정지',exact:true})).toBeEnabled();
});
test('keyboard one-shot destination, foreign stack cannot select, Esc/empty click/close clear selection',async({page,host,wire})=>{
 let foreign=false;
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();route.onMessage(frame=>server.send(frame));server.onMessage(frame=>{
  const m=decode(frame as Uint8Array) as ServerMessage;
  if(foreign&&m.type==='MovementResult'&&m.movement?.units.some(u=>u.route.length))m.movement.units.push({...m.movement.units[0],division:'999',nation:2,province:2});
  if(foreign&&m.type==='MilitaryResult'&&m.military&&m.military.divisions.length)m.military.divisions.push({...m.military.divisions[0],id:'999',nation:2,province:2});
  route.send(Buffer.from(encode(m)));
 });});
 const {world,anchors}=await prepared(page,host,wire);foreign=true;await page.getByTestId('speed-1').click();const other=page.locator('[data-division-nation="2"]');await expect(other).toHaveCount(1);await other.click();await expect(other).toHaveAttribute('aria-pressed','false');await expect(page.getByTestId('selected-province')).toHaveText('2');await expect(page.getByRole('button',{name:'Choose movement destination',exact:true})).toBeDisabled();
 await selectOwn(page);const target=page.getByRole('button',{name:'Choose movement destination',exact:true});await target.focus();await page.keyboard.press('Enter');await expect(target).toHaveAttribute('aria-pressed','true');await other.focus();await page.keyboard.press('Enter');await settled(page);await expect(target).toHaveAttribute('aria-pressed','false');expect(commands(wire).map(m=>m.command)).toEqual([{type:'Move',division:'0',destination:2}]);await expect(page.locator('[data-moving-division]')).toHaveCount(2);await expect(page.locator('[data-movement-division]')).toHaveCount(1);await expect(page.locator('[data-moving-division="999"]')).toHaveAttribute('aria-pressed','false');
 await page.keyboard.press('Escape');await expect(target).toBeDisabled();await provinceClick(page,world,anchors,14);expect(commands(wire)).toHaveLength(1);
 // Select the separate moving marker, then a real blank area outside the fitted map.
 await page.locator('[data-moving-division="0"]').click();await expect(target).toBeEnabled();const map=page.getByTestId('province-map');await map.scrollIntoViewIfNeeded();const box=(await map.boundingBox())!;await page.mouse.click(box.x+1,box.y+1);await expect(target).toBeDisabled();
 await page.locator('[data-moving-division="0"]').click();await page.getByTestId('military-panel').getByRole('button',{name:'Close military information',exact:true}).click();await expect(page.locator('[data-movement-division]')).toHaveCount(0);await expect(page.locator('[data-moving-division]')).toHaveCount(0);
 const queries=wire.sent.filter(m=>m.type==='Query'&&m.kind==='movement').length;await page.getByTestId('speed-2').click();await expect(page.getByTestId('speed')).toHaveText('2');expect(wire.sent.filter(m=>m.type==='Query'&&m.kind==='movement')).toHaveLength(queries);
 await page.getByRole('button',{name:'Open military information',exact:true}).click();await expect(target).toBeDisabled();
});
test('unsupported movement keeps static roster, unknown reference hides all overlays, disconnect/reconnect clear',async({page,host,wire})=>{
 let mode:'native'|'unsupported'|'unknown'='native',disconnect=()=>{};
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();disconnect=()=>server.close({code:1000,reason:'movement-test'});route.onMessage(frame=>server.send(frame));server.onMessage(frame=>{
  const m=decode(frame as Uint8Array) as ServerMessage;
  if(m.type==='MovementResult'){
   if(mode==='unsupported'){m.supported=false;m.movement=null;m.reason_key='unsupported-query';}
   if(mode==='unknown'&&m.movement)m.movement.units=m.movement.units.map(u=>({...u,route:[65535]}));
  }route.send(Buffer.from(encode(m)));
 });});
 const {world,anchors}=await prepared(page,host,wire);await selectOwn(page);await provinceClick(page,world,anchors,14);await settled(page);await expect(page.locator('[data-movement-division]')).toHaveCount(1);
 mode='unsupported';await page.getByTestId('speed-1').click();await expect(page.locator('[data-movement-division]')).toHaveCount(0);await expect(page.locator('[data-moving-division]')).toHaveCount(0);await expect(page.locator('.division-marker')).toHaveCount(1);await expect(page.getByRole('button',{name:'Choose movement destination',exact:true})).toHaveCount(0);
 mode='unknown';await page.getByTestId('speed-2').click();await expect(page.getByRole('alert').filter({hasText:'Map data'})).toBeVisible();await expect(page.locator('.division-marker')).toHaveCount(0);await expect(page.locator('[data-movement-division]')).toHaveCount(0);
 mode='native';await page.getByTestId('speed-3').click();await expect(page.locator('[data-movement-division]')).toHaveCount(1);disconnect();await expect(page.getByTestId('connection')).toHaveText('Disconnected');await expect(page.locator('[data-movement-division]')).toHaveCount(0);await expect(page.locator('[data-moving-division]')).toHaveCount(0);
 await page.getByRole('button',{name:'Reconnect',exact:true}).click();await expect(page.getByTestId('connection')).toHaveText('Connected');await expect(page.getByRole('button',{name:'Choose movement destination',exact:true})).toBeDisabled();
});

test('spectator never selects or commands; Join epoch and missing roster ids reset selection',async({page,host,wire})=>{
 let remove=false;
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();route.onMessage(frame=>server.send(frame));server.onMessage(frame=>{
  const m=decode(frame as Uint8Array) as ServerMessage;
  if(m.type==='MilitaryResult'&&m.military&&!remove){const template=m.military.templates[0];m.military.divisions.push({id:'999',nation:1,province:1,army:'0',template:template.template,normal:template.normal,manpower:'0',equipment:[]});}
  route.send(Buffer.from(encode(m)));
 });});
 await page.goto(host+'/?forceWebGL=1');await expect(page.getByTestId('province-map')).toHaveAttribute('data-backend','webgl2');await expect(page.getByTestId('economy-panel')).toHaveAttribute('data-status','ready');await page.evaluate(()=>document.fonts.ready);
 await page.getByRole('button',{name:'Open military information',exact:true}).click();const marker=page.locator('[data-division-nation="1"]');await expect(marker).toHaveCount(1);await marker.focus();await page.keyboard.press('Enter');await expect(marker).toHaveAttribute('aria-pressed','false');expect(commands(wire)).toHaveLength(0);await expect(page.locator('[data-movement-division]')).toHaveCount(0);
 await page.getByRole('button',{name:'Start as selected nation',exact:true}).click();await expect.poll(()=>wire.sent.some(m=>m.type==='Join'&&m.nation==='N01')).toBe(true);await expect(page.getByTestId('military-panel')).toHaveAttribute('data-status','ready');await expect(marker).toHaveAttribute('aria-pressed','false');
 const target=page.getByRole('button',{name:'Choose movement destination',exact:true});await expect(target).toBeDisabled();await marker.focus();await page.keyboard.press('Enter');await expect(marker).toHaveAttribute('aria-pressed','true');await expect(target).toBeEnabled();remove=true;await page.getByTestId('speed-1').click();await expect(marker).toHaveCount(0);await expect(target).toBeDisabled();expect(commands(wire)).toHaveLength(0);
});
