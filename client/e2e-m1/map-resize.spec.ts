import {expect} from '@playwright/test';
import {test} from './resize-probe-fixture';
import {decode,encode} from '@msgpack/msgpack';
import {mkdirSync,writeFileSync} from 'node:fs';
import type {ServerMessage,WorldView} from '../src/proto/protocol';
import {png} from './png';
const evidence=process.env.OH_MAP_RESIZE_EVIDENCE??'../target/wp08/resize';mkdirSync(evidence,{recursive:true});
test('REQ-MAP-05 resize and reset preserve presentation pixels and camera authority',async({page,request,browser},info)=>{
 const errors:string[]=[],captures:unknown[]=[];let world:WorldView|undefined;
 page.on('pageerror',e=>errors.push(e.message));let worlds=0;
 // Display-only wire fixture changes the latest palette; authority values,
 // geometry, IDs, ticks and game rules are forwarded unchanged.
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world){if(++worlds>=2)m.world.provinces.find(p=>p.id===10)!.terrain_color=[77,111,155];world=m.world;route.send(Buffer.from(encode(m)));}else route.send(data);});});
 const map=page.getByTestId('province-map'),camera=async()=>JSON.parse((await map.getAttribute('data-camera'))!);
 const click=async(x:number,y:number)=>{const b=(await map.boundingBox())!,c=await camera();await map.click({position:{x:(x-c.x)/c.width*b.width+b.width/2,y:(y-c.y)/c.height*b.height+b.height/2}});};
 const capture=async(label:string)=>{
  await page.mouse.move(0,0);const b=(await map.boundingBox())!,c=await camera(),x=(1-c.x)/c.width*b.width+b.width/2,y=(1-c.y)/c.height*b.height+b.height/2;
  const gpu=await map.evaluate(async(host,{x,y})=>{const canvas=host.querySelector('canvas')!;if((host as HTMLElement).dataset.backend!=='webgl2')return null;const gl=canvas.getContext('webgl2')!;return await new Promise<{pixel:number[];error:number;lost:boolean}>(resolve=>requestAnimationFrame(()=>{const v=new Uint8Array(4);gl.readPixels(Math.floor(x*canvas.width/host.clientWidth),canvas.height-1-Math.floor(y*canvas.height/host.clientHeight),1,1,gl.RGBA,gl.UNSIGNED_BYTE,v);resolve({pixel:Array.from(v),error:gl.getError(),lost:gl.isContextLost()});}));},{x,y});
  const key=await map.getAttribute('data-mode')==='map-mode-terrain'?'terrain_color':'owner_color';
  const actual=png(await map.screenshot({path:`${evidence}/${info.project.name}-${label}.png`})).pixel(x,y),expected=world!.provinces.find(p=>p.id===10)![key]!;
  captures.push({label,camera:c,gpu,actual,expected});writeFileSync(`${evidence}/${info.project.name}-resize.json`,JSON.stringify({browser:browser.version(),captures,errors},null,2));
  if(gpu){expect(gpu.error).toBe(0);expect(gpu.lost).toBe(false);for(let i=0;i<3;i++)expect(Math.abs(gpu.pixel[i]-expected[i])).toBeLessThanOrEqual(1);}
  for(let i=0;i<3;i++)expect(Math.abs(actual[i]-expected[i]),`${label}: visible ${actual}, expected ${expected}`).toBeLessThanOrEqual(1);
 };
 await page.goto('/');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');await capture('initial');const initial=await camera();
 for(const lang of ['en','ko']){await page.getByRole('combobox').selectOption(lang);for(const id of [10,30]){await click(1,id===10?1:3);await expect(page.getByTestId('selected-province')).toHaveText(String(id));await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText(id===10?'1':'0');const ledger=page.getByTestId('state-panel').locator('.ledger-value');if(await ledger.getAttribute('open')===null)await ledger.locator('summary').click();await expect(page.getByTestId('state-panel').locator('.ledger-tooltip')).toBeVisible();}}
 await page.getByRole('combobox').selectOption('en');const b=(await map.boundingBox())!;await page.mouse.move(b.x+b.width/2,b.y+b.height/2);await page.mouse.wheel(0,-200);await expect.poll(async()=>(await camera()).zoom).toBeGreaterThan(1);await page.mouse.down();await page.mouse.move(b.x+b.width/2+20,b.y+b.height/2+10,{steps:4});await page.mouse.up();await page.mouse.move(0,0);const before=await camera();
 for(const s of [1,2,3,4,5]){await page.getByTestId(`speed-${s}`).click();await expect(page.getByTestId('speed')).toHaveText(String(s));}
 const tick=await page.getByTestId('tick').textContent();await page.getByTestId('pause').click();await expect(page.getByTestId('tick')).not.toHaveText(tick!);await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');const paused=await page.getByTestId('tick').textContent();await expect.poll(()=>worlds).toBeGreaterThanOrEqual(2);await page.getByRole('button',{name:'Terrain',exact:true}).click();await expect(map).toHaveAttribute('data-mode','map-mode-terrain');
 await page.setViewportSize({width:1100,height:800});await expect.poll(async()=>(await camera()).width).not.toBe(before.width);const resized=await camera();expect([resized.x,resized.y,resized.zoom]).toEqual([before.x,before.y,before.zoom]);await expect(page.getByTestId('selected-province')).toHaveText('30');await capture('resized');
 await expect(map).toHaveAttribute('data-mode','map-mode-terrain');await page.setViewportSize({width:1280,height:720});await expect.poll(camera).toEqual(before);await capture('restored');await page.getByRole('button',{name:'Reset view',exact:true}).click();await expect.poll(camera).toEqual(initial);await capture('reset');await expect(map).toHaveAttribute('data-mode','map-mode-terrain');await expect(page.getByTestId('tick')).toHaveText(paused!);await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');
 await page.getByRole('button',{name:'Clear selection',exact:true}).click();
 const meta=await (await request.get('/maps/testland/metadata')).json(),raw=await (await request.get(`/maps/testland/index.bin?pack=${meta.pack_hash}`)).body();
 const oracle:unknown[]=[];for(const [mode,key] of [['Ownership','owner_color'],['Terrain','terrain_color'],['States','state_color'],['Control','controller_color']] as const){
  await page.getByRole('button',{name:mode,exact:true}).click();await page.mouse.move(0,0);await page.waitForTimeout(100);const box=(await map.boundingBox())!,cam=await camera(),image=png(await map.screenshot({path:`${evidence}/${info.project.name}-after-resize-${mode}.png`}));
  for(const [x,y] of [[1,1],[3,1],[1,3],[3,3],[1,5],[5,5]]){const dense=raw.readUInt16LE((y*meta.width+x)*2),id=meta.province_ids[dense],expected=world!.provinces.find(p=>p.id===id)![key]??world!.neutral_color,actual=image.pixel((x-cam.x)/cam.width*box.width+box.width/2,(y-cam.y)/cam.height*box.height+box.height/2);for(let i=0;i<3;i++)expect(Math.abs(actual[i]-expected[i]),`${mode} ${id}: ${actual}, expected ${expected}`).toBeLessThanOrEqual(1);oracle.push({mode,id,actual,expected});}
 }
 expect(errors).toEqual([]);expect(await page.getByRole('alert').count()).toBe(0);writeFileSync(`${evidence}/${info.project.name}-resize.json`,JSON.stringify({browser:browser.version(),captures,oracle,errors,worlds,scope:'display-only latest terrain palette fixture; authority values and geometry unchanged'},null,2));
});
