import { expect,test,type Locator } from '@playwright/test';
import { mkdirSync,writeFileSync,readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { png } from './png';
import { encode,decode } from '@msgpack/msgpack';
import type { ServerMessage,MapMetadata } from '../src/proto/protocol';
import { displayFixture,wideDisplayFixture } from './displayFixture';
const evidence=process.env.OH_MAP_EVIDENCE??'../target/wp08/e2e';mkdirSync(evidence,{recursive:true});
async function mapPoint(map:Locator,x:number,y:number){const box=(await map.boundingBox())!;const c=JSON.parse((await map.getAttribute('data-camera'))!);return {x:(x-c.x)/c.width*box.width+box.width/2,y:(y-c.y)/c.height*box.height+box.height/2};}
function closeRGB(actual:number[],expected:readonly number[],label='pixel'){for(let i=0;i<3;i++)expect(Math.abs(actual[i]-expected[i]),`${label}: ${actual} expected ${expected}`).toBeLessThanOrEqual(1);}
test('REQ-MAP-04/05 PLAT-03 actual map pixels, modes, pick and viewport',async({page,browser},info)=>{
 const requests:string[]=[];page.on('request',r=>requests.push(r.url()));page.on('websocket',ws=>requests.push(ws.url()));const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/?forceWebGL=1');
 const map=page.getByTestId('province-map');
 await expect(map).toHaveAttribute('data-backend','webgl2');
 await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);
 await expect(page.getByRole('button',{name:'Terrain',exact:true})).toBeInViewport();
 await expect(page.getByTestId('pause')).toBeInViewport();
 await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
 const box=await map.boundingBox();expect(box).not.toBeNull();
 const camera=JSON.parse((await map.getAttribute('data-camera'))!);
 const at=(x:number,y:number)=>({x:(x-camera.x)/camera.width*box!.width+box!.width/2,y:(y-camera.y)/camera.height*box!.height+box!.height/2});
 for(const [name,color] of [['Ownership',[40,100,180]],['Terrain',[130,165,95]],['States',[140,95,170]]] as const){
  await page.getByRole('button',{name,exact:true}).click();await page.mouse.move(0,0);await page.waitForTimeout(80);
  const shot=await map.screenshot({path:`${evidence}/${info.project.name}-${name}.png`});const p=at(1,1);const actual=png(shot).pixel(p.x,p.y);for(let i=0;i<3;i++)expect(Math.abs(actual[i]-color[i]),`${name} pixel ${actual}`).toBeLessThanOrEqual(1);
 }
 await map.click({position:at(1,1)});await expect(page.getByTestId('selected-province')).toHaveText('10');
 await expect(page.getByTestId('map-hover')).toHaveText('Province 10 · Plains');
 await map.click({position:at(1,3)});await expect(page.getByTestId('selected-province')).toHaveText('30');await expect(page.getByTestId('map-hover')).toHaveText('Province 30 · Plains');
 await expect(page.getByTestId('country-panel').locator('h3')).toHaveText('Southern Test Nation');await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('Southern Test State');
 await page.getByTestId('state-panel').getByText('Infrastructure',{exact:true}).click();await expect(page.getByTestId('state-panel').locator('.ledger-tooltip')).toContainText('Value ledger');await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');
 await expect(page.getByTestId('state-panel').locator('.ledger-tooltip')).toBeInViewport();await expect(map).toBeInViewport();
 await page.getByRole('combobox').selectOption('ko');await expect(page.getByTestId('map-hover')).toHaveText('프로빈스 30 · 평야');await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('남부 시험주');
 await map.click({position:at(1,1)});await expect(page.getByTestId('selected-province')).toHaveText('10');await expect(page.getByTestId('map-hover')).toHaveText('프로빈스 10 · 평야');await expect(page.getByTestId('country-panel').locator('h3')).toHaveText('북부 시험국');
 await page.screenshot({path:`${evidence}/${info.project.name}-first-map.png`});
 expect(errors).toEqual([]);expect(requests.every(u=>new URL(u).host===new URL(page.url()).host)).toBe(true);
 writeFileSync(`${evidence}/${info.project.name}-map.json`,JSON.stringify({browser:browser.version(),requests,errors,diagnostics:await map.evaluate(el=>({... (el as HTMLElement).dataset})),viewport:{width:1280,height:720},scenario:'m1',seed:'1'},null,2));
});

test('REQ-PLAT-03 preferred actual backend produces the same map RGB and source shader',async({page,browser},info)=>{
 await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-backend',/webgpu|webgl2/);await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);
 await page.mouse.move(0,0);const shot=await map.screenshot({path:`${evidence}/${info.project.name}-preferred.png`});const p=await mapPoint(map,1,1);closeRGB(png(shot).pixel(p.x,p.y),[40,100,180]);
 const diagnostics=await map.evaluate(el=>({... (el as HTMLElement).dataset}));expect(diagnostics.shaderLanguage).toBe(diagnostics.backend==='webgpu'?'wgsl':'glsl');expect(Number(diagnostics.shaderLength)).toBeGreaterThan(0);
 for(const [name,x,color] of [['Control',3,[180,70,40]],['Terrain',1,[130,165,95]],['States',1,[140,95,170]],['Ownership',1,[40,100,180]]] as const){await page.getByRole('button',{name,exact:true}).click();await page.mouse.move(0,0);await page.waitForTimeout(60);const point=await mapPoint(map,x,1);closeRGB(png(await map.screenshot({path:`${evidence}/${info.project.name}-preferred-${name}.png`})).pixel(point.x,point.y),color,name);}
 await expect.poll(async()=>Number(await map.getAttribute('data-frames'))).toBeGreaterThan(60);
 writeFileSync(`${evidence}/${info.project.name}-preferred.json`,JSON.stringify({browser:browser.version(),diagnostics:await map.evaluate(el=>({... (el as HTMLElement).dataset})),performanceScope:'tiny synthetic map; renderer frame interval, not global hardware benchmark'},null,2));
});
test('REQ-MAP-04 changed server RGB updates only affected lookup texel on actual backend',async({page},info)=>{
 let inject=false;await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(inject&&m.type==='WorldResult'&&m.world&&m.request==='world'){m.world.provinces[0].owner_color=[18,44,110];route.send(Buffer.from(encode(m)));}else route.send(data);});});
 await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);inject=true;await expect(map).toHaveAttribute('data-last-changed-bytes','4');await page.mouse.move(0,0);const point=await mapPoint(map,1,1);closeRGB(png(await map.screenshot()).pixel(point.x,point.y),[18,44,110]);const untouched=await mapPoint(map,3,1);closeRGB(png(await map.screenshot()).pixel(untouched.x,untouched.y),[40,100,180]);writeFileSync(`${evidence}/${info.project.name}-partial-update.json`,JSON.stringify(await map.evaluate(el=>({... (el as HTMLElement).dataset})),null,2));
});
test('REQ-MAP-04/05 RG8 high dense byte and LUT second row pick real ID65535',async({page,request},info)=>{
 const meta=await (await request.get('/maps/testland/metadata')).json() as MapMetadata;let fixture:ReturnType<typeof wideDisplayFixture>|undefined;
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world&&m.request==='world'){fixture=wideDisplayFixture(m.world,meta);m.world=fixture.world;route.send(Buffer.from(encode(m)));}else route.send(data);});});
 await page.route('**/maps/display_fixture/metadata',r=>r.fulfill({json:fixture!.meta}));await page.route('**/maps/display_fixture/index.bin?*',r=>r.fulfill({body:fixture!.bytes,headers:{'x-pack-hash':meta.pack_hash}}));
 await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.mouse.move(0,0);const p=await mapPoint(map,256.5,3);closeRGB(png(await map.screenshot({path:`${evidence}/${info.project.name}-dense256.png`})).pixel(p.x,p.y),[17,123,231]);await map.click({position:p});await expect(page.getByTestId('selected-province')).toHaveText('65535');writeFileSync(`${evidence}/${info.project.name}-dense256.json`,JSON.stringify(await map.evaluate(el=>({... (el as HTMLElement).dataset})),null,2));
});

for(const force of [false,true])test(`REQ-MAP-05 independent three border pixel oracle, u16 sentinel and camera ${force?'WebGL2':'preferred'}`,async({page,request},info)=>{
 const meta=await (await request.get('/maps/testland/metadata')).json() as MapMetadata;
 let fixture:ReturnType<typeof displayFixture>|undefined;
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world&&m.request==='world'){fixture=displayFixture(m.world,meta);m.world=fixture.world;route.send(Buffer.from(encode(m)));}else route.send(data);});});
 await page.route('**/maps/display_fixture/metadata',route=>route.fulfill({json:fixture!.meta}));
 await page.route('**/maps/display_fixture/index.bin?*',route=>route.fulfill({body:fixture!.bytes,contentType:'application/octet-stream',headers:{'x-pack-hash':meta.pack_hash}}));
 await page.goto(force?'/?forceWebGL=1':'/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);if(force)await expect(map).toHaveAttribute('data-backend','webgl2');
 await page.mouse.move(0,0);const shot=await map.screenshot({path:`${evidence}/${info.project.name}-borders-${force}.png`}),image=png(shot);
 const baseline=png(readFileSync(resolve(import.meta.dirname,'baselines/display-fixture.png')));
 expect([image.width,image.height]).toEqual([baseline.width,baseline.height]);
 let mismatches=0;for(let y=0;y<image.height;y++)for(let x=0;x<image.width;x++)if(image.pixel(x,y).some((v,i)=>Math.abs(v-baseline.pixel(x,y)[i])>1))mismatches++;
 // Minor raster boundary differences may affect at most 0.5% of the canvas.
 expect(mismatches/(image.width*image.height)).toBeLessThanOrEqual(.005);
 const boundaryPixels:number[][]=[];
 // Direct fixture source defines province-only x2, state-only x4, nation x6.
 for(const [x,color] of [[2,meta.style.province_border],[4,meta.style.state_border],[6,meta.style.nation_border]] as const){const p=await mapPoint(map,x,1);const c=image.pixel(p.x,p.y);boundaryPixels.push(c);closeRGB(c,color,`border x${x}`);}
 const widths:number[]=[];for(const [x,color] of [[2,meta.style.province_border],[4,meta.style.state_border],[6,meta.style.nation_border]] as const){const p=await mapPoint(map,x,1);let count=0;for(let dx=-6;dx<=6;dx++)if(image.pixel(p.x+dx,p.y).every((v,i)=>Math.abs(v-color[i])<=1))count++;widths.push(count);}
 expect(widths[0]).toBeGreaterThan(0);expect(widths[1]).toBeGreaterThan(widths[0]);expect(widths[2]).toBeGreaterThan(widths[1]);
 // Real ID0 selection is distinct from null, shader highlight and hover change pixels.
 const p0=await mapPoint(map,1,1);const box=(await map.boundingBox())!;await page.mouse.move(box.x+p0.x,box.y+p0.y);await expect(map).toHaveAttribute('data-hover','0');const hoverImage=png(await map.screenshot());closeRGB(hoverImage.pixel(p0.x,p0.y),[122,151,201],'hover');
 await map.click({position:p0});await expect(page.getByTestId('selected-province')).toHaveText('0');await page.mouse.move(0,0);const selectedImage=png(await map.screenshot());closeRGB(selectedImage.pixel(p0.x,p0.y),[122,142,162],'selection');
 await page.getByRole('button',{name:'Clear selection'}).click();await expect(page.getByTestId('selected-province')).toHaveText('—');
 await map.click({position:await mapPoint(map,11,1)});await expect(page.getByTestId('selected-province')).toHaveText('65535');await expect(page.getByText('This province has no state or country.')).toBeVisible();
 await map.click({position:await mapPoint(map,1,5)});await expect(page.getByTestId('selected-province')).toHaveText('65534');await expect(page.getByTestId('map-hover')).toHaveText('Province 65534 · Ocean');
 await page.getByRole('button',{name:'Clear selection'}).click();
 const center=await mapPoint(map,1,1);await page.mouse.move(box.x+center.x,box.y+center.y);await page.mouse.wheel(0,-300);await expect.poll(async()=>JSON.parse((await map.getAttribute('data-camera'))!).zoom).toBeGreaterThan(1);
 await map.click({position:await mapPoint(map,1,1)});await expect(page.getByTestId('selected-province')).toHaveText('0');
 const before=JSON.parse((await map.getAttribute('data-camera'))!);await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.down();await page.mouse.move(box.x+box.width-5,box.y+box.height/2,{steps:8});await page.mouse.up();const after=JSON.parse((await map.getAttribute('data-camera'))!);expect(after.x).toBeLessThan(before.x);
 await map.click({position:{x:2,y:2}});await expect(page.getByTestId('selected-province')).toHaveText('—');
 await page.getByRole('button',{name:'Reset view'}).click();const reset=JSON.parse((await map.getAttribute('data-camera'))!);expect(reset).toMatchObject({x:6,y:3,zoom:1});
 await page.mouse.move(0,0);const resetImage=png(await map.screenshot());for(const [x,color] of [[2,meta.style.province_border],[4,meta.style.state_border],[6,meta.style.nation_border]] as const){const p=await mapPoint(map,x,1);closeRGB(resetImage.pixel(p.x,p.y),color);}
 writeFileSync(`${evidence}/${info.project.name}-fixture-${force}.json`,JSON.stringify({boundaryPixels,widths,mismatches,baselinePixels:image.width*image.height,before,after,reset,diagnostics:await map.evaluate(el=>({... (el as HTMLElement).dataset})),oracle:'independent 12x6 displayFixture.ts plus reviewed new screenshot baseline; no shipped golden changed'},null,2));
});

for(const failure of ['missingGPU','nullAdapter','adapterReject','deviceReject','insecure'])test(`REQ-PLAT-03 ${failure} falls back to actual WebGL2 pixels`,async({page},info)=>{
 await page.addInitScript(kind=>{
  if(kind==='insecure')Object.defineProperty(window,'isSecureContext',{value:false});
  if(kind==='missingGPU')Object.defineProperty(navigator,'gpu',{value:undefined});
  if(kind==='nullAdapter')Object.defineProperty(navigator,'gpu',{value:{requestAdapter:async()=>null}});
  if(kind==='adapterReject')Object.defineProperty(navigator,'gpu',{value:{requestAdapter:async()=>{throw new Error('test adapter rejection');}}});
  if(kind==='deviceReject')Object.defineProperty(navigator,'gpu',{value:{requestAdapter:async()=>({features:new Set(),requestDevice:async()=>{throw new Error('test device rejection');}})}});
 },failure);
 await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-backend','webgl2');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.mouse.move(0,0);
 const p=await mapPoint(map,1,1);closeRGB(png(await map.screenshot({path:`${evidence}/${info.project.name}-${failure}.png`})).pixel(p.x,p.y),[40,100,180]);
});
test('REQ-PLAT-03 no backend shows localized notices',async({page})=>{
 await page.addInitScript(()=>{Object.defineProperty(navigator,'gpu',{value:undefined});const original=HTMLCanvasElement.prototype.getContext;HTMLCanvasElement.prototype.getContext=function(this:HTMLCanvasElement,type:string,...args:unknown[]){if(type==='webgl2')return null;return Reflect.apply(original,this,[type,...args]);} as typeof original;});
 await page.goto('/');await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toContainText('초기화할 수 없습니다');
});
test('REQ-PLAT-03 actual WebGL2 texture limit is checked before texture creation',async({page})=>{
 await page.addInitScript(()=>{Object.defineProperty(navigator,'gpu',{value:undefined});const original=WebGL2RenderingContext.prototype.getParameter;WebGL2RenderingContext.prototype.getParameter=function(parameter:number){return parameter===this.MAX_TEXTURE_SIZE?128:original.call(this,parameter);};});
 await page.goto('/');await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);
});
test('REQ-MAP-04 missing bundled index fails visibly without external download',async({page,request})=>{
 const requests:string[]=[];page.on('request',r=>requests.push(r.url()));await page.route('**/maps/testland/index.bin?*',r=>r.fulfill({status:404}));await page.goto('/');await expect(page.getByRole('alert')).toContainText('Map data could not be loaded');expect(requests.every(u=>new URL(u).host===new URL(page.url()).host)).toBe(true);
 expect((await request.get('/maps/missing/metadata')).status()).toBe(404);expect((await request.get('/maps/testland/index.bin?pack=wrong')).status()).toBe(409);expect((await request.get('/maps/testland/index.bin')).status()).toBe(409);
});
for(const invalid of ['oldVersion','futureVersion','wrongPack','duplicateIds','badStyle','shortBytes','wrongHash'])test(`REQ-MAP-04 schema boundary ${invalid} rejects atomically`,async({page,request})=>{
 const meta=await (await request.get('/maps/testland/metadata')).json();
 const bytes=await (await request.get(`/maps/testland/index.bin?pack=${meta.pack_hash}`)).body();
 if(invalid==='oldVersion')delete meta.schema_version;if(invalid==='futureVersion')meta.schema_version=65535;
 if(invalid==='wrongPack')meta.pack_hash='0000000000000000';if(invalid==='duplicateIds')meta.province_ids[1]=meta.province_ids[0];if(invalid==='badStyle')meta.style.fit_milli=0;
 if(invalid==='wrongHash')meta.index_hash='0000000000000000';
 await page.route('**/maps/testland/metadata',r=>r.fulfill({json:meta}));
 if(invalid==='shortBytes')await page.route('**/maps/testland/index.bin?*',r=>r.fulfill({body:bytes.subarray(0,bytes.length-1),headers:{'x-pack-hash':meta.pack_hash}}));
 await page.goto('/');await expect(page.getByRole('alert')).toContainText('Map data could not be loaded');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
});
for(const invalid of ['dangling','changedDimensions','changedDense'])test(`REQ-MAP-05 ${invalid} preserves last valid map and closes wire`,async({page})=>{
 let count=0;await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world&&++count>=3){if(invalid==='dangling')m.world.provinces[0].owner=65535;if(invalid==='changedDimensions')m.world.width++;if(invalid==='changedDense')m.world.province_ids.reverse();route.send(Buffer.from(encode(m)));}else route.send(data);});});
 await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await expect(page.getByTestId('connection')).toHaveText('Disconnected');await expect(page.getByRole('alert')).toHaveText('Invalid server message; connection closed');await expect(page.getByTestId('pause')).toBeDisabled();
 await page.mouse.move(0,0);const point=await mapPoint(map,1,1);closeRGB(png(await map.screenshot()).pixel(point.x,point.y),[40,100,180]);await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
});
