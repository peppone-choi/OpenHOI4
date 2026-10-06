# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: map.spec.ts >> REQ-PLAT-03 adapterReject falls back to actual WebGL2 pixels
- Location: e2e-m1\map.spec.ts:93:93

# Error details

```
Error: expect(locator).toHaveAttribute(expected) failed

Locator: getByTestId('province-map')
Expected: "webgl2"
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toHaveAttribute" getByTestId('province-map') with timeout 5000ms
  - waiting for getByTestId('province-map')

```

```yaml
- main:
  - heading "OpenHOI4" [level=1]
  - paragraph: Local scenario
  - text: Language
  - combobox "Language":
    - option "English" [selected]
    - option "한국어"
  - status: Connected
  - region "Time controls":
    - term: Date
    - definition: Waiting for server
    - term: Hour
    - definition: —
    - term: Tick
    - definition: —
    - term: Speed
    - definition: —
    - button "Pause" [disabled]
    - button "Speed 1" [disabled]: "1"
    - button "Speed 2" [disabled]: "2"
    - button "Speed 3" [disabled]: "3"
    - button "Speed 4" [disabled]: "4"
    - button "Speed 5" [disabled]: "5"
  - link "Noto Sans KR · SIL Open Font License 1.1":
    - /url: /fonts/OFL.txt
```

# Test source

```ts
  1   | import { expect,test,type Locator } from '@playwright/test';
  2   | import { mkdirSync,writeFileSync,readFileSync } from 'node:fs';
  3   | import { resolve } from 'node:path';
  4   | import { png } from './png';
  5   | import { encode,decode } from '@msgpack/msgpack';
  6   | import type { ServerMessage,MapMetadata } from '../src/proto/protocol';
  7   | import { displayFixture,wideDisplayFixture } from './displayFixture';
  8   | const evidence=process.env.OH_MAP_EVIDENCE??'../target/wp08/e2e';mkdirSync(evidence,{recursive:true});
  9   | async function mapPoint(map:Locator,x:number,y:number){const box=(await map.boundingBox())!;const c=JSON.parse((await map.getAttribute('data-camera'))!);return {x:(x-c.x)/c.width*box.width+box.width/2,y:(y-c.y)/c.height*box.height+box.height/2};}
  10  | function closeRGB(actual:number[],expected:readonly number[],label='pixel'){for(let i=0;i<3;i++)expect(Math.abs(actual[i]-expected[i]),`${label}: ${actual} expected ${expected}`).toBeLessThanOrEqual(1);}
  11  | test('REQ-MAP-04/05 PLAT-03 actual map pixels, modes, pick and viewport',async({page,browser},info)=>{
  12  |  const requests:string[]=[];page.on('request',r=>requests.push(r.url()));page.on('websocket',ws=>requests.push(ws.url()));const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  13  |  await page.goto('/?forceWebGL=1');
  14  |  const map=page.getByTestId('province-map');
  15  |  await expect(map).toHaveAttribute('data-backend','webgl2');
  16  |  await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);
  17  |  await expect(page.getByRole('button',{name:'Terrain',exact:true})).toBeInViewport();
  18  |  await expect(page.getByTestId('pause')).toBeInViewport();
  19  |  await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
  20  |  const box=await map.boundingBox();expect(box).not.toBeNull();
  21  |  const camera=JSON.parse((await map.getAttribute('data-camera'))!);
  22  |  const at=(x:number,y:number)=>({x:(x-camera.x)/camera.width*box!.width+box!.width/2,y:(y-camera.y)/camera.height*box!.height+box!.height/2});
  23  |  for(const [name,color] of [['Ownership',[40,100,180]],['Terrain',[130,165,95]],['States',[140,95,170]]] as const){
  24  |   await page.getByRole('button',{name,exact:true}).click();await page.mouse.move(0,0);await page.waitForTimeout(80);
  25  |   const shot=await map.screenshot({path:`${evidence}/${info.project.name}-${name}.png`});const p=at(1,1);const actual=png(shot).pixel(p.x,p.y);for(let i=0;i<3;i++)expect(Math.abs(actual[i]-color[i]),`${name} pixel ${actual}`).toBeLessThanOrEqual(1);
  26  |  }
  27  |  await map.click({position:at(1,1)});await expect(page.getByTestId('selected-province')).toHaveText('10');
  28  |  await expect(page.getByTestId('map-hover')).toHaveText('Province 10 · Plains');
  29  |  await map.click({position:at(1,3)});await expect(page.getByTestId('selected-province')).toHaveText('30');await expect(page.getByTestId('map-hover')).toHaveText('Province 30 · Plains');
  30  |  await expect(page.getByTestId('country-panel').locator('h3')).toHaveText('Southern Test Nation');await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('Southern Test State');
  31  |  await page.getByTestId('state-panel').getByText('Infrastructure',{exact:true}).click();await expect(page.getByTestId('state-panel').locator('.ledger-tooltip')).toContainText('Value ledger');await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');
  32  |  await expect(page.getByTestId('state-panel').locator('.ledger-tooltip')).toBeInViewport();await expect(map).toBeInViewport();
  33  |  await page.getByRole('combobox').selectOption('ko');await expect(page.getByTestId('map-hover')).toHaveText('프로빈스 30 · 평야');await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('남부 시험주');
  34  |  await map.click({position:at(1,1)});await expect(page.getByTestId('selected-province')).toHaveText('10');await expect(page.getByTestId('map-hover')).toHaveText('프로빈스 10 · 평야');await expect(page.getByTestId('country-panel').locator('h3')).toHaveText('북부 시험국');
  35  |  await page.screenshot({path:`${evidence}/${info.project.name}-first-map.png`});
  36  |  expect(errors).toEqual([]);expect(requests.every(u=>new URL(u).host===new URL(page.url()).host)).toBe(true);
  37  |  writeFileSync(`${evidence}/${info.project.name}-map.json`,JSON.stringify({browser:browser.version(),requests,errors,diagnostics:await map.evaluate(el=>({... (el as HTMLElement).dataset})),viewport:{width:1280,height:720},scenario:'m1',seed:'1'},null,2));
  38  | });
  39  | 
  40  | test('REQ-PLAT-03 preferred actual backend produces the same map RGB and source shader',async({page,browser},info)=>{
  41  |  await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-backend',/webgpu|webgl2/);await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);
  42  |  await page.mouse.move(0,0);const shot=await map.screenshot({path:`${evidence}/${info.project.name}-preferred.png`});const p=await mapPoint(map,1,1);closeRGB(png(shot).pixel(p.x,p.y),[40,100,180]);
  43  |  const diagnostics=await map.evaluate(el=>({... (el as HTMLElement).dataset}));expect(diagnostics.shaderLanguage).toBe(diagnostics.backend==='webgpu'?'wgsl':'glsl');expect(Number(diagnostics.shaderLength)).toBeGreaterThan(0);
  44  |  for(const [name,x,color] of [['Control',3,[180,70,40]],['Terrain',1,[130,165,95]],['States',1,[140,95,170]],['Ownership',1,[40,100,180]]] as const){await page.getByRole('button',{name,exact:true}).click();await page.mouse.move(0,0);await page.waitForTimeout(60);const point=await mapPoint(map,x,1);closeRGB(png(await map.screenshot({path:`${evidence}/${info.project.name}-preferred-${name}.png`})).pixel(point.x,point.y),color,name);}
  45  |  await expect.poll(async()=>Number(await map.getAttribute('data-frames'))).toBeGreaterThan(60);
  46  |  writeFileSync(`${evidence}/${info.project.name}-preferred.json`,JSON.stringify({browser:browser.version(),diagnostics:await map.evaluate(el=>({... (el as HTMLElement).dataset})),performanceScope:'tiny synthetic map; renderer frame interval, not global hardware benchmark'},null,2));
  47  | });
  48  | test('REQ-MAP-04 changed server RGB updates only affected lookup texel on actual backend',async({page},info)=>{
  49  |  let inject=false;await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(inject&&m.type==='WorldResult'&&m.world&&m.request==='world'){m.world.provinces[0].owner_color=[18,44,110];route.send(Buffer.from(encode(m)));}else route.send(data);});});
  50  |  await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);inject=true;await expect(map).toHaveAttribute('data-last-changed-bytes','4');await page.mouse.move(0,0);const point=await mapPoint(map,1,1);closeRGB(png(await map.screenshot()).pixel(point.x,point.y),[18,44,110]);const untouched=await mapPoint(map,3,1);closeRGB(png(await map.screenshot()).pixel(untouched.x,untouched.y),[40,100,180]);writeFileSync(`${evidence}/${info.project.name}-partial-update.json`,JSON.stringify(await map.evaluate(el=>({... (el as HTMLElement).dataset})),null,2));
  51  | });
  52  | test('REQ-MAP-04/05 RG8 high dense byte and LUT second row pick real ID65535',async({page,request},info)=>{
  53  |  const meta=await (await request.get('/maps/testland/metadata')).json() as MapMetadata;let fixture:ReturnType<typeof wideDisplayFixture>|undefined;
  54  |  await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world&&m.request==='world'){fixture=wideDisplayFixture(m.world,meta);m.world=fixture.world;route.send(Buffer.from(encode(m)));}else route.send(data);});});
  55  |  await page.route('**/maps/display_fixture/metadata',r=>r.fulfill({json:fixture!.meta}));await page.route('**/maps/display_fixture/index.bin?*',r=>r.fulfill({body:fixture!.bytes,headers:{'x-pack-hash':meta.pack_hash}}));
  56  |  await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.mouse.move(0,0);const p=await mapPoint(map,256.5,3);closeRGB(png(await map.screenshot({path:`${evidence}/${info.project.name}-dense256.png`})).pixel(p.x,p.y),[17,123,231]);await map.click({position:p});await expect(page.getByTestId('selected-province')).toHaveText('65535');writeFileSync(`${evidence}/${info.project.name}-dense256.json`,JSON.stringify(await map.evaluate(el=>({... (el as HTMLElement).dataset})),null,2));
  57  | });
  58  | 
  59  | for(const force of [false,true])test(`REQ-MAP-05 independent three border pixel oracle, u16 sentinel and camera ${force?'WebGL2':'preferred'}`,async({page,request},info)=>{
  60  |  const meta=await (await request.get('/maps/testland/metadata')).json() as MapMetadata;
  61  |  let fixture:ReturnType<typeof displayFixture>|undefined;
  62  |  await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world&&m.request==='world'){fixture=displayFixture(m.world,meta);m.world=fixture.world;route.send(Buffer.from(encode(m)));}else route.send(data);});});
  63  |  await page.route('**/maps/display_fixture/metadata',route=>route.fulfill({json:fixture!.meta}));
  64  |  await page.route('**/maps/display_fixture/index.bin?*',route=>route.fulfill({body:fixture!.bytes,contentType:'application/octet-stream',headers:{'x-pack-hash':meta.pack_hash}}));
  65  |  await page.goto(force?'/?forceWebGL=1':'/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);if(force)await expect(map).toHaveAttribute('data-backend','webgl2');
  66  |  await page.mouse.move(0,0);const shot=await map.screenshot({path:`${evidence}/${info.project.name}-borders-${force}.png`}),image=png(shot);
  67  |  const baseline=png(readFileSync(resolve(import.meta.dirname,'baselines/display-fixture.png')));
  68  |  expect([image.width,image.height]).toEqual([baseline.width,baseline.height]);
  69  |  let mismatches=0;for(let y=0;y<image.height;y++)for(let x=0;x<image.width;x++)if(image.pixel(x,y).some((v,i)=>Math.abs(v-baseline.pixel(x,y)[i])>1))mismatches++;
  70  |  // Minor raster boundary differences may affect at most 0.5% of the canvas.
  71  |  expect(mismatches/(image.width*image.height)).toBeLessThanOrEqual(.005);
  72  |  const boundaryPixels:number[][]=[];
  73  |  // Direct fixture source defines province-only x2, state-only x4, nation x6.
  74  |  for(const [x,color] of [[2,meta.style.province_border],[4,meta.style.state_border],[6,meta.style.nation_border]] as const){const p=await mapPoint(map,x,1);const c=image.pixel(p.x,p.y);boundaryPixels.push(c);closeRGB(c,color,`border x${x}`);}
  75  |  const widths:number[]=[];for(const [x,color] of [[2,meta.style.province_border],[4,meta.style.state_border],[6,meta.style.nation_border]] as const){const p=await mapPoint(map,x,1);let count=0;for(let dx=-6;dx<=6;dx++)if(image.pixel(p.x+dx,p.y).every((v,i)=>Math.abs(v-color[i])<=1))count++;widths.push(count);}
  76  |  expect(widths[0]).toBeGreaterThan(0);expect(widths[1]).toBeGreaterThan(widths[0]);expect(widths[2]).toBeGreaterThan(widths[1]);
  77  |  // Real ID0 selection is distinct from null, shader highlight and hover change pixels.
  78  |  const p0=await mapPoint(map,1,1);const box=(await map.boundingBox())!;await page.mouse.move(box.x+p0.x,box.y+p0.y);await expect(map).toHaveAttribute('data-hover','0');const hoverImage=png(await map.screenshot());closeRGB(hoverImage.pixel(p0.x,p0.y),[122,151,201],'hover');
  79  |  await map.click({position:p0});await expect(page.getByTestId('selected-province')).toHaveText('0');await page.mouse.move(0,0);const selectedImage=png(await map.screenshot());closeRGB(selectedImage.pixel(p0.x,p0.y),[122,142,162],'selection');
  80  |  await page.getByRole('button',{name:'Clear selection'}).click();await expect(page.getByTestId('selected-province')).toHaveText('—');
  81  |  await map.click({position:await mapPoint(map,11,1)});await expect(page.getByTestId('selected-province')).toHaveText('65535');await expect(page.getByText('This province has no state or country.')).toBeVisible();
  82  |  await map.click({position:await mapPoint(map,1,5)});await expect(page.getByTestId('selected-province')).toHaveText('65534');await expect(page.getByTestId('map-hover')).toHaveText('Province 65534 · Ocean');
  83  |  await page.getByRole('button',{name:'Clear selection'}).click();
  84  |  const center=await mapPoint(map,1,1);await page.mouse.move(box.x+center.x,box.y+center.y);await page.mouse.wheel(0,-300);await expect.poll(async()=>JSON.parse((await map.getAttribute('data-camera'))!).zoom).toBeGreaterThan(1);
  85  |  await map.click({position:await mapPoint(map,1,1)});await expect(page.getByTestId('selected-province')).toHaveText('0');
  86  |  const before=JSON.parse((await map.getAttribute('data-camera'))!);await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.down();await page.mouse.move(box.x+box.width-5,box.y+box.height/2,{steps:8});await page.mouse.up();const after=JSON.parse((await map.getAttribute('data-camera'))!);expect(after.x).toBeLessThan(before.x);
  87  |  await map.click({position:{x:2,y:2}});await expect(page.getByTestId('selected-province')).toHaveText('—');
  88  |  await page.getByRole('button',{name:'Reset view'}).click();const reset=JSON.parse((await map.getAttribute('data-camera'))!);expect(reset).toMatchObject({x:6,y:3,zoom:1});
  89  |  await page.mouse.move(0,0);const resetImage=png(await map.screenshot());for(const [x,color] of [[2,meta.style.province_border],[4,meta.style.state_border],[6,meta.style.nation_border]] as const){const p=await mapPoint(map,x,1);closeRGB(resetImage.pixel(p.x,p.y),color);}
  90  |  writeFileSync(`${evidence}/${info.project.name}-fixture-${force}.json`,JSON.stringify({boundaryPixels,widths,mismatches,baselinePixels:image.width*image.height,before,after,reset,diagnostics:await map.evaluate(el=>({... (el as HTMLElement).dataset})),oracle:'independent 12x6 displayFixture.ts plus reviewed new screenshot baseline; no shipped golden changed'},null,2));
  91  | });
  92  | 
  93  | for(const failure of ['missingGPU','nullAdapter','adapterReject','deviceReject','insecure'])test(`REQ-PLAT-03 ${failure} falls back to actual WebGL2 pixels`,async({page},info)=>{
  94  |  await page.addInitScript(kind=>{
  95  |   if(kind==='insecure')Object.defineProperty(window,'isSecureContext',{value:false});
  96  |   if(kind==='missingGPU')Object.defineProperty(navigator,'gpu',{value:undefined});
  97  |   if(kind==='nullAdapter')Object.defineProperty(navigator,'gpu',{value:{requestAdapter:async()=>null}});
  98  |   if(kind==='adapterReject')Object.defineProperty(navigator,'gpu',{value:{requestAdapter:async()=>{throw new Error('test adapter rejection');}}});
  99  |   if(kind==='deviceReject')Object.defineProperty(navigator,'gpu',{value:{requestAdapter:async()=>({features:new Set(),requestDevice:async()=>{throw new Error('test device rejection');}})}});
  100 |  },failure);
> 101 |  await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-backend','webgl2');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.mouse.move(0,0);
      |                                                                                    ^ Error: expect(locator).toHaveAttribute(expected) failed
  102 |  const p=await mapPoint(map,1,1);closeRGB(png(await map.screenshot({path:`${evidence}/${info.project.name}-${failure}.png`})).pixel(p.x,p.y),[40,100,180]);
  103 | });
  104 | for(const preferred of [false,true])test(`REQ-PLAT-03 no backend shows localized notices${preferred?' after preferred rejection':''}`,async({page},info)=>{
  105 |  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  106 |  await page.addInitScript(prefer=>{Object.defineProperty(navigator,'gpu',{value:prefer?{requestAdapter:async()=>null}:undefined});const original=HTMLCanvasElement.prototype.getContext;HTMLCanvasElement.prototype.getContext=function(this:HTMLCanvasElement,type:string,...args:unknown[]){if(type==='webgl2')return null;return Reflect.apply(original,this,[type,...args]);} as typeof original;},preferred);
  107 |  await page.goto('/');await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toContainText('초기화할 수 없습니다');
  108 |  await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('재개');
  109 |  const tick=await page.getByTestId('tick').textContent();await expect(page.getByTestId('country-panel')).toContainText('북부 시험국');await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('1');
  110 |  await page.getByRole('combobox').selectOption('en');await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(page.getByTestId('tick')).toHaveText(tick!);await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
  111 |  expect(errors).toEqual([]);
  112 |  writeFileSync(`${evidence}/${info.project.name}-no-backend-${preferred?'preferred':'direct'}.json`,JSON.stringify({preferred,errors,tick,canvas:0,authorityPreserved:true},null,2));
  113 | });
  114 | test('REQ-PLAT-03 actual WebGL2 texture limit is checked before texture creation',async({page})=>{
  115 |  await page.addInitScript(()=>{Object.defineProperty(navigator,'gpu',{value:undefined});const original=WebGL2RenderingContext.prototype.getParameter;WebGL2RenderingContext.prototype.getParameter=function(parameter:number){return parameter===this.MAX_TEXTURE_SIZE?128:original.call(this,parameter);};});
  116 |  await page.goto('/');await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);
  117 | });
  118 | test('REQ-MAP-04 missing bundled index fails visibly without external download',async({page,request})=>{
  119 |  const requests:string[]=[];page.on('request',r=>requests.push(r.url()));await page.route('**/maps/testland/index.bin?*',r=>r.fulfill({status:404}));await page.goto('/');await expect(page.getByRole('alert')).toContainText('Map data could not be loaded');expect(requests.every(u=>new URL(u).host===new URL(page.url()).host)).toBe(true);
  120 |  expect((await request.get('/maps/missing/metadata')).status()).toBe(404);expect((await request.get('/maps/testland/index.bin?pack=wrong')).status()).toBe(409);expect((await request.get('/maps/testland/index.bin')).status()).toBe(409);
  121 | });
  122 | for(const invalid of ['oldVersion','futureVersion','wrongPack','duplicateIds','badStyle','shortBytes','wrongHash'])test(`REQ-MAP-04 schema boundary ${invalid} rejects atomically`,async({page,request})=>{
  123 |  const meta=await (await request.get('/maps/testland/metadata')).json();
  124 |  const bytes=await (await request.get(`/maps/testland/index.bin?pack=${meta.pack_hash}`)).body();
  125 |  if(invalid==='oldVersion')delete meta.schema_version;if(invalid==='futureVersion')meta.schema_version=65535;
  126 |  if(invalid==='wrongPack')meta.pack_hash='0000000000000000';if(invalid==='duplicateIds')meta.province_ids[1]=meta.province_ids[0];if(invalid==='badStyle')meta.style.fit_milli=0;
  127 |  if(invalid==='wrongHash')meta.index_hash='0000000000000000';
  128 |  await page.route('**/maps/testland/metadata',r=>r.fulfill({json:meta}));
  129 |  if(invalid==='shortBytes')await page.route('**/maps/testland/index.bin?*',r=>r.fulfill({body:bytes.subarray(0,bytes.length-1),headers:{'x-pack-hash':meta.pack_hash}}));
  130 |  await page.goto('/');await expect(page.getByRole('alert')).toContainText('Map data could not be loaded');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
  131 | });
  132 | for(const invalid of ['dangling','changedDimensions','changedDense'])test(`REQ-MAP-05 ${invalid} preserves last valid map and closes wire`,async({page})=>{
  133 |  let count=0;await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world&&++count>=3){if(invalid==='dangling')m.world.provinces[0].owner=65535;if(invalid==='changedDimensions')m.world.width++;if(invalid==='changedDense')m.world.province_ids.reverse();route.send(Buffer.from(encode(m)));}else route.send(data);});});
  134 |  await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await expect(page.getByTestId('connection')).toHaveText('Disconnected');await expect(page.getByRole('alert')).toHaveText('Invalid server message; connection closed');await expect(page.getByTestId('pause')).toBeDisabled();
  135 |  await page.mouse.move(0,0);const point=await mapPoint(map,1,1);closeRGB(png(await map.screenshot()).pixel(point.x,point.y),[40,100,180]);await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
  136 | });
  137 | 
```