# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: map-resize.spec.ts >> REQ-MAP-05 resize and reset preserve presentation pixels and camera authority
- Location: e2e-m1/map-resize.spec.ts:8:1

# Error details

```
Test timeout of 30000ms exceeded.
```

```
Error: locator.screenshot: Element is not attached to the DOM
Call log:
  - taking element screenshot
  - waiting for fonts to load...
  - fonts loaded
  - attempting scroll into view action
    - waiting for element to be stable

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
        - definition [ref=e15]: 2011-02-03
      - generic [ref=e16]:
        - term [ref=e17]: Hour
        - definition [ref=e18]: "15"
      - generic [ref=e19]:
        - term [ref=e20]: Tick
        - definition [ref=e21]: "97239"
      - generic [ref=e22]:
        - term [ref=e23]: Speed
        - definition [ref=e24]: "5"
    - generic [ref=e25]:
      - button "Resume" [ref=e26] [cursor=pointer]
      - button "Speed 1" [ref=e27] [cursor=pointer]: "1"
      - button "Speed 2" [ref=e28] [cursor=pointer]: "2"
      - button "Speed 3" [ref=e29] [cursor=pointer]: "3"
      - button "Speed 4" [ref=e30] [cursor=pointer]: "4"
      - button "Speed 5" [pressed] [ref=e31] [cursor=pointer]: "5"
  - generic [ref=e32]:
    - region "Province map" [ref=e33]:
      - navigation "Map modes" [ref=e34]:
        - button "Ownership" [ref=e35] [cursor=pointer]
        - button "Control" [ref=e36] [cursor=pointer]
        - button "Terrain" [active] [pressed] [ref=e37] [cursor=pointer]
        - button "States" [ref=e38] [cursor=pointer]
        - button "Reset view" [ref=e39] [cursor=pointer]
      - generic [ref=e42]:
        - generic [ref=e43]:
          - text: "Province:"
          - strong [ref=e44]: "30"
        - generic [ref=e45]: Province 30 · Plains
        - button "Clear selection" [ref=e46] [cursor=pointer]
    - complementary [ref=e47]:
      - generic [ref=e48]:
        - region "Country" [ref=e49]:
          - heading "Country" [level=2] [ref=e50]
          - navigation [ref=e51]:
            - button "Northern Test Nation" [ref=e52] [cursor=pointer]
            - button "Southern Test Nation" [pressed] [ref=e53] [cursor=pointer]
          - heading "Southern Test Nation" [level=3] [ref=e54]
          - generic [ref=e55]:
            - generic [ref=e56]:
              - term [ref=e57]: Tag
              - definition [ref=e58]: STH
            - generic [ref=e59]:
              - term [ref=e60]: Capital province
              - definition [ref=e61]: "30"
            - generic [ref=e62]:
              - term [ref=e63]: Government
              - definition [ref=e64]: Test Council
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
            - button "Northern Test State" [ref=e76] [cursor=pointer]
            - button "Southern Test State" [pressed] [ref=e77] [cursor=pointer]
          - heading "Southern Test State" [level=3] [ref=e78]
          - generic [ref=e79]:
            - generic [ref=e80]:
              - term [ref=e81]: Owner
              - definition [ref=e82]: Southern Test Nation
            - generic [ref=e83]:
              - term [ref=e84]: Population
              - definition [ref=e85]: "2000"
            - generic [ref=e86]:
              - term [ref=e87]: Steel
              - definition [ref=e88]: "0"
            - generic [ref=e89]:
              - term [ref=e90]: Industry
              - definition [ref=e91]: "0"
          - group [ref=e92]:
            - generic "Show ledger for Infrastructure" [ref=e93] [cursor=pointer]:
              - text: Infrastructure
              - strong [ref=e94]: "0"
            - generic [ref=e95]:
              - heading "Value ledger" [level=2] [ref=e96]
              - paragraph [ref=e97]: "Tick: 97239"
              - generic [ref=e98]:
                - generic [ref=e99]:
                  - term [ref=e100]: Base value
                  - definition [ref=e101]: "0"
                - generic [ref=e102]:
                  - term [ref=e103]: Final value
                  - definition [ref=e104]: "0"
              - table [ref=e105]:
                - rowgroup [ref=e106]:
                  - row [ref=e107]:
                    - columnheader "Contribution" [ref=e108]
                    - columnheader "Operation" [ref=e109]
                    - columnheader "Value" [ref=e110]
                    - columnheader "Accumulated value" [ref=e111]
                - rowgroup [ref=e112]:
                  - row [ref=e113]:
                    - 'cell "Infrastructure Source: Base value" [ref=e114]':
                      - text: Infrastructure
                      - generic [ref=e115]: "Source: Base value"
                    - cell "Base value" [ref=e116]
                    - cell "0" [ref=e117]
                    - cell "0" [ref=e118]
          - table [ref=e119]:
            - rowgroup [ref=e120]:
              - row [ref=e121]:
                - columnheader "Province" [ref=e122]
                - columnheader "Owner" [ref=e123]
                - columnheader "Controller" [ref=e124]
            - rowgroup [ref=e125]:
              - row [ref=e126]:
                - cell "30" [ref=e127]
                - cell "Southern Test Nation" [ref=e128]
                - cell "Southern Test Nation" [ref=e129]
              - row [ref=e130]:
                - cell "40" [ref=e131]
                - cell "Southern Test Nation" [ref=e132]
                - cell "Southern Test Nation" [ref=e133]
  - link "Noto Sans KR · SIL Open Font License 1.1" [ref=e135] [cursor=pointer]:
    - /url: /fonts/OFL.txt
```

# Test source

```ts
  1  | import {expect} from '@playwright/test';
  2  | import {test} from './resize-probe-fixture';
  3  | import {decode,encode} from '@msgpack/msgpack';
  4  | import {mkdirSync,writeFileSync} from 'node:fs';
  5  | import type {ServerMessage,WorldView} from '../src/proto/protocol';
  6  | import {png} from './png';
  7  | const evidence=process.env.OH_MAP_RESIZE_EVIDENCE??'../target/wp08/resize';mkdirSync(evidence,{recursive:true});
  8  | test('REQ-MAP-05 resize and reset preserve presentation pixels and camera authority',async({page,request,browser},info)=>{
  9  |  const errors:string[]=[],captures:unknown[]=[];let world:WorldView|undefined;
  10 |  page.on('pageerror',e=>errors.push(e.message));let worlds=0;
  11 |  // Display-only wire fixture changes the latest palette; authority values,
  12 |  // geometry, IDs, ticks and game rules are forwarded unchanged.
  13 |  await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world){if(++worlds>=2)m.world.provinces.find(p=>p.id===10)!.terrain_color=[77,111,155];world=m.world;route.send(Buffer.from(encode(m)));}else route.send(data);});});
  14 |  const map=page.getByTestId('province-map'),camera=async()=>JSON.parse((await map.getAttribute('data-camera'))!);
  15 |  const click=async(x:number,y:number)=>{const b=(await map.boundingBox())!,c=await camera();await map.click({position:{x:(x-c.x)/c.width*b.width+b.width/2,y:(y-c.y)/c.height*b.height+b.height/2}});};
  16 |  const capture=async(label:string)=>{
  17 |   await page.mouse.move(0,0);const b=(await map.boundingBox())!,c=await camera(),x=(1-c.x)/c.width*b.width+b.width/2,y=(1-c.y)/c.height*b.height+b.height/2;
  18 |   const gpu=await map.evaluate(async(host,{x,y})=>{const canvas=host.querySelector('canvas')!;if((host as HTMLElement).dataset.backend!=='webgl2')return null;const gl=canvas.getContext('webgl2')!;return await new Promise<{pixel:number[];error:number;lost:boolean}>(resolve=>requestAnimationFrame(()=>{const v=new Uint8Array(4);gl.readPixels(Math.floor(x*canvas.width/host.clientWidth),canvas.height-1-Math.floor(y*canvas.height/host.clientHeight),1,1,gl.RGBA,gl.UNSIGNED_BYTE,v);resolve({pixel:Array.from(v),error:gl.getError(),lost:gl.isContextLost()});}));},{x,y});
  19 |   const key=await map.getAttribute('data-mode')==='map-mode-terrain'?'terrain_color':'owner_color';
> 20 |   const actual=png(await map.screenshot({path:`${evidence}/${info.project.name}-${label}.png`})).pixel(x,y),expected=world!.provinces.find(p=>p.id===10)![key]!;
     |                              ^ Error: locator.screenshot: Element is not attached to the DOM
  21 |   captures.push({label,camera:c,gpu,actual,expected});writeFileSync(`${evidence}/${info.project.name}-resize.json`,JSON.stringify({browser:browser.version(),captures,errors},null,2));
  22 |   if(gpu){expect(gpu.error).toBe(0);expect(gpu.lost).toBe(false);for(let i=0;i<3;i++)expect(Math.abs(gpu.pixel[i]-expected[i])).toBeLessThanOrEqual(1);}
  23 |   for(let i=0;i<3;i++)expect(Math.abs(actual[i]-expected[i]),`${label}: visible ${actual}, expected ${expected}`).toBeLessThanOrEqual(1);
  24 |  };
  25 |  await page.goto('/');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');await capture('initial');const initial=await camera();
  26 |  for(const lang of ['en','ko']){await page.getByRole('combobox').selectOption(lang);for(const id of [10,30]){await click(1,id===10?1:3);await expect(page.getByTestId('selected-province')).toHaveText(String(id));await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText(id===10?'1':'0');const ledger=page.getByTestId('state-panel').locator('.ledger-value');if(await ledger.getAttribute('open')===null)await ledger.locator('summary').click();await expect(page.getByTestId('state-panel').locator('.ledger-tooltip')).toBeVisible();}}
  27 |  await page.getByRole('combobox').selectOption('en');const b=(await map.boundingBox())!;await page.mouse.move(b.x+b.width/2,b.y+b.height/2);await page.mouse.wheel(0,-200);await expect.poll(async()=>(await camera()).zoom).toBeGreaterThan(1);await page.mouse.down();await page.mouse.move(b.x+b.width/2+20,b.y+b.height/2+10,{steps:4});await page.mouse.up();await page.mouse.move(0,0);const before=await camera();
  28 |  for(const s of [1,2,3,4,5]){await page.getByTestId(`speed-${s}`).click();await expect(page.getByTestId('speed')).toHaveText(String(s));}
  29 |  const tick=await page.getByTestId('tick').textContent();await page.getByTestId('pause').click();await expect(page.getByTestId('tick')).not.toHaveText(tick!);await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');const paused=await page.getByTestId('tick').textContent();await expect.poll(()=>worlds).toBeGreaterThanOrEqual(2);await page.getByRole('button',{name:'Terrain',exact:true}).click();await expect(map).toHaveAttribute('data-mode','map-mode-terrain');
  30 |  await page.setViewportSize({width:1100,height:800});await expect.poll(async()=>(await camera()).width).not.toBe(before.width);const resized=await camera();expect([resized.x,resized.y,resized.zoom]).toEqual([before.x,before.y,before.zoom]);await expect(page.getByTestId('selected-province')).toHaveText('30');await capture('resized');
  31 |  await expect(map).toHaveAttribute('data-mode','map-mode-terrain');await page.setViewportSize({width:1280,height:720});await expect.poll(camera).toEqual(before);await capture('restored');await page.getByRole('button',{name:'Reset view',exact:true}).click();await expect.poll(camera).toEqual(initial);await capture('reset');await expect(map).toHaveAttribute('data-mode','map-mode-terrain');await expect(page.getByTestId('tick')).toHaveText(paused!);await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');
  32 |  await page.getByRole('button',{name:'Clear selection',exact:true}).click();
  33 |  const meta=await (await request.get('/maps/testland/metadata')).json(),raw=await (await request.get(`/maps/testland/index.bin?pack=${meta.pack_hash}`)).body();
  34 |  const oracle:unknown[]=[];for(const [mode,key] of [['Ownership','owner_color'],['Terrain','terrain_color'],['States','state_color'],['Control','controller_color']] as const){
  35 |   await page.getByRole('button',{name:mode,exact:true}).click();await page.mouse.move(0,0);await page.waitForTimeout(100);const box=(await map.boundingBox())!,cam=await camera(),image=png(await map.screenshot({path:`${evidence}/${info.project.name}-after-resize-${mode}.png`}));
  36 |   for(const [x,y] of [[1,1],[3,1],[1,3],[3,3],[1,5],[5,5]]){const dense=raw.readUInt16LE((y*meta.width+x)*2),id=meta.province_ids[dense],expected=world!.provinces.find(p=>p.id===id)![key]??world!.neutral_color,actual=image.pixel((x-cam.x)/cam.width*box.width+box.width/2,(y-cam.y)/cam.height*box.height+box.height/2);for(let i=0;i<3;i++)expect(Math.abs(actual[i]-expected[i]),`${mode} ${id}: ${actual}, expected ${expected}`).toBeLessThanOrEqual(1);oracle.push({mode,id,actual,expected});}
  37 |  }
  38 |  expect(errors).toEqual([]);expect(await page.getByRole('alert').count()).toBe(0);writeFileSync(`${evidence}/${info.project.name}-resize.json`,JSON.stringify({browser:browser.version(),captures,oracle,errors,worlds,scope:'display-only latest terrain palette fixture; authority values and geometry unchanged'},null,2));
  39 | });
  40 |
```
