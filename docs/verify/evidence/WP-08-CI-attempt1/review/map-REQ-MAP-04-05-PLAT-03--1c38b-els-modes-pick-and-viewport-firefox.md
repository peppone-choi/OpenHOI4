# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: map.spec.ts >> REQ-MAP-04/05 PLAT-03 actual map pixels, modes, pick and viewport
- Location: e2e-m1/map.spec.ts:11:1

# Error details

```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  getByTestId('province-map')
Expected: "webgl2"
Received: ""
Timeout:  5000ms

Call log:
  - Expect "toHaveAttribute" getByTestId('province-map') with timeout 5000ms
  - waiting for getByTestId('province-map')
    13 × locator resolved to <div class="province-map" aria-label="Province map" data-testid="province-map"></div>
       - unexpected value "null"

```

# Page snapshot

```yaml
- main [ref=e3]:
  - generic [ref=e4]:
    - generic [ref=e5]:
      - heading "OpenHOI4" [level=1] [ref=e6]
      - paragraph [ref=e7]: Local scenario
    - generic [ref=e8]:
      - text: Language
      - combobox "Language" [ref=e9]:
        - option "English" [selected]
        - option "한국어"
  - status [ref=e10]: Connected
  - region "Time controls" [ref=e11]:
    - generic [ref=e12]:
      - generic [ref=e13]:
        - term [ref=e14]: Date
        - definition [ref=e15]: 2000-01-01
      - generic [ref=e16]:
        - term [ref=e17]: Hour
        - definition [ref=e18]: "9"
      - generic [ref=e19]:
        - term [ref=e20]: Tick
        - definition [ref=e21]: "9"
      - generic [ref=e22]:
        - term [ref=e23]: Speed
        - definition [ref=e24]: "1"
    - generic [ref=e25]:
      - button "Pause" [ref=e26] [cursor=pointer]
      - button "Speed 1" [pressed] [ref=e27] [cursor=pointer]: "1"
      - button "Speed 2" [ref=e28] [cursor=pointer]: "2"
      - button "Speed 3" [ref=e29] [cursor=pointer]: "3"
      - button "Speed 4" [ref=e30] [cursor=pointer]: "4"
      - button "Speed 5" [ref=e31] [cursor=pointer]: "5"
  - generic [ref=e32]:
    - region "Province map" [ref=e33]:
      - navigation "Map modes" [ref=e34]:
        - button "Ownership" [pressed] [ref=e35] [cursor=pointer]
        - button "Control" [ref=e36] [cursor=pointer]
        - button "Terrain" [ref=e37] [cursor=pointer]
        - button "States" [ref=e38] [cursor=pointer]
        - button "Reset view" [ref=e39] [cursor=pointer]
      - alert [ref=e41]: This browser could not initialize WebGPU or WebGL2. Try a browser that supports WebGL2.
      - generic [ref=e42]:
        - generic [ref=e43]:
          - text: "Province:"
          - strong [ref=e44]: —
        - generic [ref=e45]: Click to select · Drag to pan · Scroll to zoom
        - button "Clear selection" [disabled] [ref=e46]
    - complementary [ref=e47]:
      - generic [ref=e48]:
        - region "Country" [ref=e49]:
          - heading "Country" [level=2] [ref=e50]
          - navigation [ref=e51]:
            - button "Northern Test Nation" [pressed] [ref=e52] [cursor=pointer]
            - button "Southern Test Nation" [ref=e53] [cursor=pointer]
          - heading "Northern Test Nation" [level=3] [ref=e54]
          - generic [ref=e55]:
            - generic [ref=e56]:
              - term [ref=e57]: Tag
              - definition [ref=e58]: NTH
            - generic [ref=e59]:
              - term [ref=e60]: Capital province
              - definition [ref=e61]: "10"
            - generic [ref=e62]:
              - term [ref=e63]: Government
              - definition [ref=e64]: Test Republic
          - heading "Ideology support (ratio)" [level=4] [ref=e65]
          - generic [ref=e66]:
            - generic [ref=e67]:
              - term [ref=e68]: Civic
              - definition [ref=e69]: "0.75"
            - generic [ref=e70]:
              - term [ref=e71]: Local
              - definition [ref=e72]: "0.25"
        - region "State" [ref=e73]:
          - heading "State" [level=2] [ref=e74]
          - navigation [ref=e75]:
            - button "Northern Test State" [pressed] [ref=e76] [cursor=pointer]
            - button "Southern Test State" [ref=e77] [cursor=pointer]
          - heading "Northern Test State" [level=3] [ref=e78]
          - generic [ref=e79]:
            - generic [ref=e80]:
              - term [ref=e81]: Owner
              - definition [ref=e82]: Northern Test Nation
            - generic [ref=e83]:
              - term [ref=e84]: Population
              - definition [ref=e85]: "1000"
            - generic [ref=e86]:
              - term [ref=e87]: Steel
              - definition [ref=e88]: "2"
            - generic [ref=e89]:
              - term [ref=e90]: Industry
              - definition [ref=e91]: "1"
          - group [ref=e92]:
            - generic "Show ledger for Infrastructure" [ref=e93] [cursor=pointer]:
              - text: Infrastructure
              - strong [ref=e94]: "1"
          - table [ref=e95]:
            - rowgroup [ref=e96]:
              - row [ref=e97]:
                - columnheader "Province" [ref=e98]
                - columnheader "Owner" [ref=e99]
                - columnheader "Controller" [ref=e100]
            - rowgroup [ref=e101]:
              - row [ref=e102]:
                - cell "10" [ref=e103]
                - cell "Northern Test Nation" [ref=e104]
                - cell "Northern Test Nation" [ref=e105]
              - row [ref=e106]:
                - cell "20" [ref=e107]
                - cell "Northern Test Nation" [ref=e108]
                - cell "Southern Test Nation" [ref=e109]
  - link "Noto Sans KR · SIL Open Font License 1.1" [ref=e111] [cursor=pointer]:
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
> 15  |  await expect(map).toHaveAttribute('data-backend','webgl2');
      |                    ^ Error: expect(locator).toHaveAttribute(expected) failed
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
  101 |  await page.goto('/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-backend','webgl2');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.mouse.move(0,0);
  102 |  const p=await mapPoint(map,1,1);closeRGB(png(await map.screenshot({path:`${evidence}/${info.project.name}-${failure}.png`})).pixel(p.x,p.y),[40,100,180]);
  103 | });
  104 | test('REQ-PLAT-03 no backend shows localized notices',async({page})=>{
  105 |  await page.addInitScript(()=>{Object.defineProperty(navigator,'gpu',{value:undefined});const original=HTMLCanvasElement.prototype.getContext;HTMLCanvasElement.prototype.getContext=function(this:HTMLCanvasElement,type:string,...args:unknown[]){if(type==='webgl2')return null;return Reflect.apply(original,this,[type,...args]);} as typeof original;});
  106 |  await page.goto('/');await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toContainText('초기화할 수 없습니다');
  107 | });
  108 | test('REQ-PLAT-03 actual WebGL2 texture limit is checked before texture creation',async({page})=>{
  109 |  await page.addInitScript(()=>{Object.defineProperty(navigator,'gpu',{value:undefined});const original=WebGL2RenderingContext.prototype.getParameter;WebGL2RenderingContext.prototype.getParameter=function(parameter:number){return parameter===this.MAX_TEXTURE_SIZE?128:original.call(this,parameter);};});
  110 |  await page.goto('/');await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);
  111 | });
  112 | test('REQ-MAP-04 missing bundled index fails visibly without external download',async({page,request})=>{
  113 |  const requests:string[]=[];page.on('request',r=>requests.push(r.url()));await page.route('**/maps/testland/index.bin?*',r=>r.fulfill({status:404}));await page.goto('/');await expect(page.getByRole('alert')).toContainText('Map data could not be loaded');expect(requests.every(u=>new URL(u).host===new URL(page.url()).host)).toBe(true);
  114 |  expect((await request.get('/maps/missing/metadata')).status()).toBe(404);expect((await request.get('/maps/testland/index.bin?pack=wrong')).status()).toBe(409);expect((await request.get('/maps/testland/index.bin')).status()).toBe(409);
  115 | });
```