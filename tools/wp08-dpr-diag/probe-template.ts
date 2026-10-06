import {expect} from '../../../client/node_modules/@playwright/test';
import {test,begin,phase,profilePng,finish} from '../profile';
import {decode,encode} from '../../../client/node_modules/@msgpack/msgpack';
import {mkdirSync,writeFileSync} from 'node:fs';
import type {ServerMessage,WorldView} from '../../../client/src/proto/protocol';
import {png} from '../../../client/e2e-m1/png';
const evidence=process.env.OH_MAP_DPR_EVIDENCE??'../target/wp08/dpr';mkdirSync(evidence,{recursive:true});
for(const force of [false,true])test(`REQ-PLAT-03 DPR surface preserves display and authority ${force?'forced GL':'preferred'}`,async({browser,browserName},info)=>{
 // Chromium supports an actual live DPR change through tab-scoped CDP.
 // Firefox/WebKit exercise native DPR2 creation; no dynamic-change claim.
 begin(info,force);
 const context=await browser.newContext({viewport:{width:1280,height:720},deviceScaleFactor:browserName==='chromium'?1:2,locale:'en-US',baseURL:info.project.use.baseURL});
 const page=await context.newPage(),errors:string[]=[],steps:unknown[]=[];let world:WorldView|undefined,worlds=0,reload=false,changed=false,metadata=0,index=0;
 const map=page.getByTestId('province-map'),cam=async()=>JSON.parse((await map.getAttribute('data-camera'))!),life=()=>page.evaluate(()=>(window as unknown as {__dprLife:()=>unknown}).__dprLife());
 try{
 await page.addInitScript(()=>{
  const media=new Set<EventListenerOrEventListenerObject>(),listeners=new Set<EventListenerOrEventListenerObject>(),observers=new Set<ResizeObserver>(),rafs=new Set<number>();
  const add=EventTarget.prototype.addEventListener,remove=EventTarget.prototype.removeEventListener;
  EventTarget.prototype.addEventListener=function(type,listener,options){if(listener){if(this instanceof MediaQueryList&&type==='change')media.add(listener);if(this instanceof HTMLElement&&this.classList.contains('province-map'))listeners.add(listener);}return add.call(this,type,listener,options);};
  EventTarget.prototype.removeEventListener=function(type,listener,options){if(listener){if(this instanceof MediaQueryList&&type==='change')media.delete(listener);if(this instanceof HTMLElement&&this.classList.contains('province-map'))listeners.delete(listener);}return remove.call(this,type,listener,options);};
  const observe=ResizeObserver.prototype.observe,disconnect=ResizeObserver.prototype.disconnect;
  ResizeObserver.prototype.observe=function(target,options){if(target instanceof HTMLElement&&target.classList.contains('province-map'))observers.add(this);return observe.call(this,target,options);};
  ResizeObserver.prototype.disconnect=function(){observers.delete(this);return disconnect.call(this);};
  const raf=window.requestAnimationFrame,cancel=window.cancelAnimationFrame;
  window.requestAnimationFrame=fn=>{let id:number;id=raf(t=>{rafs.delete(id);fn(t);});rafs.add(id);return id;};window.cancelAnimationFrame=id=>{rafs.delete(id);cancel(id);};
  Object.assign(window,{__dprLife:()=>({media:media.size,listeners:listeners.size,observers:observers.size,rafs:rafs.size})});
 });
 page.on('pageerror',e=>errors.push(e.message));
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data as Buffer) as ServerMessage;if(m.type==='WorldResult'&&m.world){if(++worlds>=2)m.world.provinces.find(p=>p.id===10)!.terrain_color=[77,111,155];world=m.world;}if(reload&&!changed){changed=true;route.send(Buffer.from(encode({type:'Welcome',engine_version:'fixture',protocol_version:'m0-v1',accepted:true,reason_key:null,packs:[{id:'testland',version:'0.1.0',hash:'0000000000000001'}],sessions:['local']})));}route.send(Buffer.from(encode(m)));});});
 await page.route('**/maps/testland/metadata',r=>{metadata++;return reload?r.fulfill({status:404}):r.continue();});
 await page.route('**/maps/testland/index.bin?*',r=>{index++;return r.continue();});
 phase('navigate');
 await page.goto(force?'/?forceWebGL=1':'/');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
 phase('paused-ready');
 const box=(await map.boundingBox())!,camera=await cam();
 await map.click({position:{x:(1-camera.x)/camera.width*box.width+box.width/2,y:(3-camera.y)/camera.height*box.height+box.height/2}});
 await expect(page.getByTestId('selected-province')).toHaveText('30');await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');
 await page.getByTestId('speed-2').click();await expect.poll(()=>worlds).toBeGreaterThanOrEqual(2);await page.getByRole('button',{name:'Terrain',exact:true}).click();
 await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.wheel(0,-200);await expect.poll(async()=>(await cam()).zoom).toBeGreaterThan(1);await page.mouse.down();await page.mouse.move(box.x+box.width/2+20,box.y+box.height/2+10,{steps:4});await page.mouse.up();await page.mouse.move(0,0);
 phase('presentation-setup');
 const saved=await cam(),authority={tick:await page.getByTestId('tick').textContent(),date:await page.getByTestId('date').textContent(),hour:await page.getByTestId('hour').textContent()},backend=await map.getAttribute('data-backend'),cdp=browserName==='chromium'?await context.newCDPSession(page):null;
 const buffer=()=>map.locator('canvas').evaluate((c:HTMLCanvasElement)=>({width:c.width,height:c.height,expectedWidth:Math.floor(c.clientWidth*devicePixelRatio),expectedHeight:Math.floor(c.clientHeight*devicePixelRatio)}));
 for(const dpr of cdp?[2,1.5,1,2,1.5,1]:[2]){
  phase(`dpr-${steps.length}-${dpr}-request`);
  const old=await map.locator('canvas').elementHandle();if(cdp)await cdp.send('Emulation.setDeviceMetricsOverride',{width:1280,height:720,deviceScaleFactor:dpr,mobile:false});
  await expect.poll(()=>page.evaluate(()=>devicePixelRatio)).toBe(dpr);
  // No Reset or CSS geometry change may be used to trigger this update.
  await expect.poll(async()=>{const b=await buffer();return {width:b.width-b.expectedWidth,height:b.height-b.expectedHeight};}).toEqual({width:0,height:0});
  await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);expect(await map.boundingBox()).toEqual(box);expect(await cam()).toEqual(saved);await expect(map).toHaveAttribute('data-mode','map-mode-terrain');
  if(cdp){expect(await old!.evaluate(c=>c.isConnected)).toBe(backend==='webgpu');if(backend==='webgl2')expect(await old!.evaluate((c:HTMLCanvasElement)=>c.getContext('webgl2')!.isContextLost())).toBe(true);}
  // Element screenshots on Chromium restore the configured context DPR1.
  // Capture the full native viewport without a clip for this live DPR test.
  // Existing resize/mode tests retain their element screenshot stability gate.
  phase(`dpr-${steps.length}-${dpr}-ready`);
  const path=`${evidence}/${info.project.name}-${force}-dpr${dpr}-${steps.length}.png`,b=(await map.boundingBox())!,c=await cam(),viewport=await page.evaluate(()=>({width:innerWidth,height:innerHeight,scrollX,scrollY}));
  const bytes=cdp?Buffer.from((await cdp.send('Page.captureScreenshot',{format:'png',captureBeyondViewport:false})).data,'base64'):await map.screenshot({path});if(cdp)writeFileSync(path,bytes);
  phase(`dpr-${steps.length}-${dpr}-capture-done`);
  const image=profilePng(()=>png(bytes)),x=(1-c.x)/c.width*b.width+b.width/2,y=(1-c.y)/c.height*b.height+b.height/2;
  const actual=cdp?image.pixel((b.x+x)*image.width/viewport.width,(b.y+y)*image.height/viewport.height):image.pixel(x*image.width/b.width,y*image.height/b.height),expected=world!.provinces.find(p=>p.id===10)!.terrain_color!;for(let i=0;i<3;i++)expect(Math.abs(actual[i]-expected[i])).toBeLessThanOrEqual(1);
  phase(`dpr-${steps.length}-${dpr}-decode-done`);
  const gpu=await map.evaluate(async(host,{x,y})=>{if(host.dataset.backend!=='webgl2')return null;const canvas=host.querySelector('canvas')!,gl=canvas.getContext('webgl2')!;return new Promise<{pixel:number[];error:number;lost:boolean}>(resolve=>requestAnimationFrame(()=>{const pixel=new Uint8Array(4);gl.readPixels(Math.floor(x*canvas.width/host.clientWidth),canvas.height-1-Math.floor(y*canvas.height/host.clientHeight),1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);resolve({pixel:Array.from(pixel),error:gl.getError(),lost:gl.isContextLost()});}));},{x,y});
  phase(`dpr-${steps.length}-${dpr}-raw-done`);
  if(gpu){expect(gpu.error).toBe(0);expect(gpu.lost).toBe(false);for(let i=0;i<3;i++)expect(Math.abs(gpu.pixel[i]-expected[i])).toBeLessThanOrEqual(1);}
  await expect.poll(()=>page.evaluate(()=>devicePixelRatio)).toBe(dpr);await expect.poll(async()=>{const b=await buffer();return {width:b.width-b.expectedWidth,height:b.height-b.expectedHeight};}).toEqual({width:0,height:0});
  await expect(page.getByTestId('selected-province')).toHaveText('30');await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');for(const [key,value] of Object.entries(authority))await expect(page.getByTestId(key)).toHaveText(value!);
  const resources=await life();expect(resources).toMatchObject({listeners:6,observers:1});steps.push({dpr,backend,buffer:await buffer(),actual,expected,gpu,image:{width:image.width,height:image.height,capture:cdp?'native full viewport without clip':'element screenshot'},camera:c,box:b,authority,resources});
  phase(`dpr-${steps.length-1}-${dpr}-assertions-done`);
 }
 phase('cleanup');
 await cdp?.detach();expect(metadata).toBe(1);expect(index).toBe(1);
 reload=true;await page.getByTestId('speed-3').click();await expect(page.getByRole('alert').filter({hasText:'Map data'})).toContainText('Map data');await expect(map.locator('canvas')).toHaveCount(0);await expect.poll(life).toEqual({media:0,listeners:0,observers:0,rafs:0});
 await expect(page.getByTestId('selected-province')).toHaveText('30');await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('0');for(const [key,value] of Object.entries(authority))await expect(page.getByTestId(key)).toHaveText(value!);expect(errors).toEqual([]);
 }finally{phase('finally');writeFileSync(`${evidence}/${info.project.name}-${force}.json`,JSON.stringify({browser:browser.version(),scope:browserName==='chromium'?'live CDP DPR1↔2, fixed CSS box':'native DPR2 creation only; live CDP unavailable',steps,errors,resources:await page.evaluate(()=>(window as unknown as {__dprLife?:()=>unknown}).__dprLife?.()).catch(()=>null),surface:await map.evaluate(host=>{const canvas=host.querySelector('canvas');return {dpr:devicePixelRatio,resolution1:matchMedia('(resolution: 1dppx)').matches,resolution2:matchMedia('(resolution: 2dppx)').matches,canvas:canvas?{width:canvas.width,height:canvas.height,cssWidth:canvas.clientWidth,cssHeight:canvas.clientHeight}:null,dataset:{...host.dataset}};}).catch(()=>null)},null,2));await context.close();finish(info);}
});
