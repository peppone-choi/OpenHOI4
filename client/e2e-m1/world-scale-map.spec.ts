import { expect,test,type Locator,type APIRequestContext } from '@playwright/test';
import { existsSync,mkdirSync,readFileSync,writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
import { decode,encode } from '@msgpack/msgpack';
import type { MapMetadata,ServerMessage,WorldView } from '../src/proto/protocol';
import { PROTOCOL_VERSION } from '../src/proto/protocol';
import { decodeIndex,fnv,palettes,updateColors,validateWorldDisplay } from '../src/map/model';
import { provinceAnchors } from '../src/map/divisionMarkers';
import { fixtureFNV,loadWorldScale,scope,verifyPayload,worldScaleFixture } from './worldScaleFixture';
import { png } from './png';

const evidence=resolve(import.meta.dirname,'../../target/world-scale');
mkdirSync(evidence,{recursive:true});
function save(name:string,data:unknown){writeFileSync(resolve(evidence,name),JSON.stringify(data,null,2));}
function slope(points:{n:number;ms:number[]}[]){
 const xs=points.map(p=>Math.log(p.n)),ys=points.map(p=>Math.log(p.ms.reduce((a,b)=>a+b,0)/p.ms.length));
 const mean=(a:number[])=>a.reduce((x,y)=>x+y,0)/a.length,mx=mean(xs),my=mean(ys);
 return xs.reduce((s,x,i)=>s+(x-mx)*(ys[i]-my),0)/xs.reduce((s,x)=>s+(x-mx)**2,0);
}
async function baseWorld(request:APIRequestContext){
 return (await (await request.get('/maps/testland/metadata')).json()) as MapMetadata;
}
/** Browser-free real server handshake, matching network.ts Hello/Join/Query. */
async function serverWorld(baseURL:string):Promise<WorldView>{
 return new Promise((resolve,reject)=>{
  const ws=new WebSocket(baseURL.replace('http:','ws:')+'/ws');ws.binaryType='arraybuffer';let world:WorldView|undefined;
  const timer=setTimeout(()=>{ws.close();reject(new Error('Node server WorldResult timeout'));},60000);
  ws.onopen=()=>ws.send(encode({type:'Hello',protocol_version:PROTOCOL_VERSION}));
  ws.onmessage=event=>{try{
   const m=decode(new Uint8Array(event.data as ArrayBuffer)) as ServerMessage;
   if(m.type==='Welcome'){if(!m.accepted)throw new Error('Node server Hello rejected');ws.send(encode({type:'Join',session:'local',nation:null}));}
   if(m.type==='Snapshot')ws.send(encode({type:'Query',request:'world',kind:'world'}));
   if(m.type==='WorldResult'){if(!m.world)throw new Error('Node server world unavailable');world=m.world;ws.close();}
  }catch(error){clearTimeout(timer);ws.close();reject(error);}};
  ws.onerror=()=>{clearTimeout(timer);ws.close();reject(new Error('Node server WebSocket error'));};
  ws.onclose=()=>{clearTimeout(timer);if(world)resolve(world);else reject(new Error('Node server closed before WorldResult'));};
 });
}
test.afterEach(async({},info)=>{
 const name=info.title.includes('Node three runs')?'world-scale-node.json':info.title.includes('Chromium actual App')?'world-scale-chromium.json':null;
 if(!name||!existsSync(resolve(evidence,name)))return;
 const report=JSON.parse(readFileSync(resolve(evidence,name),'utf8'));
 report.testStatus=info.status;
 if(info.status!=='passed'){report.status='FAIL';report.failureClass??=info.status==='timedOut'?'execution-timeout':'assertion-or-runtime';report.error??=info.errors.map(e=>e.message).join('\n');}
 save(name,report);
});

test('W1 trust boundary rejects corrupted payloads and independent FNV vectors',async()=>{
 for(const name of ['index.bin','provinces.json'])expect(()=>verifyPayload(name,Buffer.from('corrupt'),'0'.repeat(64))).toThrow(`${name}: sha256 mismatch`);
 expect(fixtureFNV(Buffer.alloc(0))).toBe('cbf29ce484222325');
 expect(fixtureFNV(Buffer.from('hello'))).toBe('a430d84680aabd0b');
});

test('W1 Node three runs and four input sizes (measurements, not performance gates)',async({request,baseURL},info)=>{
 const report:Record<string,unknown>={...scope,status:'RUNNING',exactSHA:process.env.OH_EXACT_SHA,runner:process.env.RUNNER_OS,node:process.version,functions:{},series:{}};
 const flush=()=>save('world-scale-node.json',report);flush();
 const began=performance.now();
 try{
  const input=loadWorldScale();report.input=input.hashes;flush();
  const meta=await baseWorld(request),full=worldScaleFixture(input,await serverWorld(baseURL!),meta).world;
  report.inputTransport='Node native WebSocket; no browser fixture';validateWorldDisplay(full);
  expect(full.nations).toEqual([]);expect(full.states).toEqual([]);
  expect(full.provinces.every(p=>p.owner===null&&p.controller===null&&p.state===null&&p.owner_color===null&&p.controller_color===null&&p.state_color===null)).toBe(true);
  const buffer=Uint8Array.from(input.bytes).buffer;
  const index=decodeIndex(buffer,full.width,full.height,full.province_ids);
  const measurements=report.functions as Record<string,number[]>;
  const measure=(name:string,run:()=>unknown)=>{const times:number[]=[];measurements[name]=times;for(let repeat=0;repeat<3;repeat++){const start=performance.now();run();times.push(performance.now()-start);flush();}return times;};
  measure('validateWorldDisplay',()=>validateWorldDisplay(full));
  measure('decodeIndex',()=>decodeIndex(buffer,full.width,full.height,full.province_ids));
  measure('fnv',()=>expect(fnv(input.bytes)).toBe(input.indexHash));
  measure('palettes(owner)',()=>palettes(full,'map-mode-owner'));
  measure('updateColors(terrain)',()=>updateColors(new Uint8Array(256*256*4),full,'map-mode-terrain'));
  measure('provinceAnchors',()=>provinceAnchors(full,index));
  const series=report.series as Record<string,unknown>;
  for(const [name,run] of [['validateWorldDisplay',(w:WorldView)=>validateWorldDisplay(w)],['palettes(owner)',(w:WorldView)=>palettes(w,'map-mode-owner')],['updateColors(terrain)',(w:WorldView)=>updateColors(new Uint8Array(256*256*4),w,'map-mode-terrain')]] as const){
   const points:{n:number;ms:number[]}[]=[];series[name]={points};
   for(const n of [3111,6222,12444,24889]){const w={...full,province_ids:full.province_ids.slice(0,n),provinces:full.provinces.slice(0,n)};points.push({n,ms:measure(`${name}@${n}`,()=>run(w))});flush();}
   series[name]={points,logLogSlope:slope(points)};flush();
  }
  report.status='PASS';
 }catch(error){report.status='FAIL';report.error=String(error);throw error;}
 finally{report.elapsedMs=performance.now()-began;report.testStatus=info.status;flush();}
});

async function diagnostics(map:Locator){return map.evaluate(el=>({... (el as HTMLElement).dataset}));}
async function mapPoint(map:Locator,x:number,y:number){const box=(await map.boundingBox())!,camera=JSON.parse((await map.getAttribute('data-camera'))!);return {x:(x-camera.x)/camera.width*box.width+box.width/2,y:(y-camera.y)/camera.height*box.height+box.height/2};}

test('W1 Chromium actual App world-scale load, terrain, pick and source pixel oracle',async({page,request,browser},info)=>{
 const began=performance.now();
 const report:Record<string,unknown>={...scope,status:'RUNNING',phase:'fixture',exactSHA:process.env.OH_EXACT_SHA,browser:browser.version(),runner:process.env.RUNNER_OS,renderingScope:'GitHub-hosted software rendering; not product GPU performance',requests:[],errors:[],stages:{}};
 const stages=report.stages as Record<string,unknown>,requests=report.requests as string[],errors=report.errors as string[];
 const flush=()=>save('world-scale-chromium.json',report);flush();
 const map=page.getByTestId('province-map');let injectedAt=0,suppressedWorldResults=0;
 page.on('request',r=>{requests.push(r.url());flush();});page.on('websocket',ws=>{requests.push(ws.url());flush();});page.on('pageerror',e=>{errors.push(e.message);flush();});
 try{
  const input=loadWorldScale();report.input=input.hashes;flush();
  const originalMeta=await baseWorld(request);let fixture:ReturnType<typeof worldScaleFixture>|undefined;
  await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{
   const m=decode(data as Buffer) as ServerMessage;
   if(m.type==='WorldResult'&&m.world){
    // Freeze the injected display snapshot; do not mix server testland updates
    // or recurring 24,889-province palette rebuilds into one-shot measurements.
    if(fixture){suppressedWorldResults++;return;}
    fixture=worldScaleFixture(input,m.world,originalMeta);m.world=fixture.world;injectedAt=performance.now();report.injectedAtMs=injectedAt;report.phase='load';flush();route.send(Buffer.from(encode(m)));
   }else route.send(data);
  });});
  await page.route('**/maps/world_scale_harness/metadata',r=>r.fulfill({json:fixture!.meta}));
  await page.route('**/maps/world_scale_harness/index.bin?*',r=>r.fulfill({body:input.bytes,contentType:'application/octet-stream',headers:{'x-pack-hash':originalMeta.pack_hash}}));
  const cdp=await page.context().newCDPSession(page);await cdp.send('Performance.enable');report.metricsBefore=await cdp.send('Performance.getMetrics');flush();
  await page.goto('/?forceWebGL=1',{waitUntil:'commit'});
  await expect.poll(async()=>{
   const alert=await page.locator('.map-section [role=alert]').textContent({timeout:100}).catch(()=>null);
   if(alert)throw new Error(`MapView alert: ${alert}`);
   const frames=await map.getAttribute('data-frames',{timeout:1000}).catch(()=>null);
   return Number(frames)>=2&&await page.locator('.map-section [role=status]').count()===0;
  },{timeout:840000,intervals:[250,1000]}).toBe(true);
  stages.load={wallMs:performance.now()-injectedAt,diagnostics:await diagnostics(map)};report.metricsAfter=await cdp.send('Performance.getMetrics');flush();
  expect(requests.filter(u=>new URL(u).pathname.startsWith('/maps/')).some(u=>u.includes('/world_scale_harness/index.bin?'))).toBe(true);
  report.phase='terrain';const modeStart=performance.now();await page.getByRole('button',{name:'Terrain',exact:true}).click();await expect(map).toHaveAttribute('data-mode','map-mode-terrain');
  stages.terrain={wallMs:performance.now()-modeStart,diagnostics:await diagnostics(map)};flush();
  await expect(map).toHaveAttribute('data-upload-path','lookup');await expect(map).toHaveAttribute('data-last-changed-bytes','99556');
  const index=decodeIndex(Uint8Array.from(input.bytes).buffer,fixture!.world.width,fixture!.world.height,fixture!.world.province_ids);
  const largest=(kind:'land'|'sea')=>input.provinces.filter(p=>p.kind===kind).sort((a,b)=>b.pixels-a.pixels||a.id-b.id)[0];
  const land=largest('land'),sea=largest('sea');
  const anchors=provinceAnchors(fixture!.world,index),anchor=anchors.get(land.id)!;
  // Zoom at the exact independent raster anchor to make pointer quantization safe.
  const p=await mapPoint(map,anchor.x,anchor.y),box=(await map.boundingBox())!;
  await page.mouse.move(box.x+p.x,box.y+p.y);await page.mouse.wheel(0,-800);
  await expect.poll(async()=>JSON.parse((await map.getAttribute('data-camera'))!).zoom).toBeGreaterThan(1);
  report.phase='selection';const selectStart=performance.now();await map.click({position:await mapPoint(map,anchor.x,anchor.y)});await expect(map).toHaveAttribute('data-selected',String(land.id));
  stages.selection={id:land.id,anchor,wallMs:performance.now()-selectStart,diagnostics:await diagnostics(map)};flush();
  await expect(map).toHaveAttribute('data-selected','18014');await expect(map).toHaveAttribute('data-hover','18014');await expect(map).toHaveAttribute('data-updated-bytes','0');await expect(map).toHaveAttribute('data-upload-path','none');
  await page.getByRole('button',{name:'Clear selection'}).click();await page.getByRole('button',{name:'Reset view'}).click();await page.mouse.move(0,0);
  report.phase='pixels';
  // Choose screenshot pixel centres whose entire 5x5 screen neighbourhood maps
  // to the target in the INPUT raster. Never derive expected RGB from rendering.
  const pixels=[];
  for(const province of [land,sea]){
   await page.getByRole('button',{name:'Reset view'}).click();
   const target=anchors.get(province.id)!,position=await mapPoint(map,target.x,target.y),bounds=(await map.boundingBox())!;
   await page.mouse.move(bounds.x+position.x,bounds.y+position.y);
   const oldCamera=await map.getAttribute('data-camera');await page.mouse.wheel(0,-2400);
   await expect.poll(()=>map.getAttribute('data-camera')).not.toBe(oldCamera);
   await page.mouse.move(0,0);
   const frame=Number(await map.getAttribute('data-frames'));await expect.poll(async()=>Number(await map.getAttribute('data-frames'))).toBeGreaterThan(frame+1);
   const camera=JSON.parse((await map.getAttribute('data-camera'))!),rect=(await map.boundingBox())!;
   const image=png(await map.screenshot({path:resolve(evidence,`chromium-terrain-${province.kind}.png`),scale:'css'}));
   const dense=fixture!.world.province_ids.indexOf(province.id);let point:{x:number;y:number}|undefined;
   const at=(x:number,y:number)=>{const sx=Math.floor((x+.5-rect.width/2)/rect.width*camera.width+camera.x),sy=Math.floor((y+.5-rect.height/2)/rect.height*camera.height+camera.y);return sx>=0&&sx<8192&&sy>=0&&sy<4096?index[sy*8192+sx]:-1;};
   for(let y=3;y<image.height-3&&!point;y++)for(let x=3;x<image.width-3;x++)if([-2,-1,0,1,2].every(dy=>[-2,-1,0,1,2].every(dx=>at(x+dx,y+dy)===dense))){point={x,y};break;}
   expect(point,`input interior for ${province.kind} ${province.id}`).toBeDefined();
   const actual=image.pixel(point!.x,point!.y),expected=province.geography_color;pixels.push({id:province.id,kind:province.kind,point,camera,actual,expected,tolerance:1});stages.pixels=pixels;flush();
   expect(actual).toHaveLength(3);for(let channel=0;channel<3;channel++)expect(Math.abs(actual[channel]-expected[channel])).toBeLessThanOrEqual(1);
  }
  expect(errors).toEqual([]);
  expect(requests.every(u=>{const a=new URL(u),b=new URL(page.url());return a.host===b.host&&(a.protocol===b.protocol||a.protocol==='ws:');})).toBe(true);
  report.status='PASS';report.phase='complete';
 }catch(error){
  report.status='FAIL';const detail=String(error);report.error=detail;
  // Product MapView currently collapses texture-limit into backend-unavailable.
  // Do not claim to distinguish errors the actual UI does not expose.
  report.failureClass=detail.includes('texture-limit')?'map-texture-limit':detail.includes('initialize WebGPU or WebGL2')?'map-backend-unavailable':detail.includes('Map data could not')?'map-data-error':info.status==='timedOut'||/Timeout|timed out|exceeded/i.test(detail)?'execution-timeout':'assertion-or-runtime';
  throw error;
 }finally{
  report.elapsedMs=performance.now()-began;report.suppressedWorldResults=suppressedWorldResults;report.testStatus=report.status==='FAIL'?'failed':info.status;
  report.sameOrigin=requests.every(u=>{const a=new URL(u),b=new URL(page.url());return a.host===b.host&&(a.protocol===b.protocol||a.protocol==='ws:');});flush();
  if(!page.isClosed()){
   report.finalDiagnostics=await diagnostics(map).catch(()=>null);
   await page.screenshot({path:resolve(evidence,'chromium-final.png'),timeout:5000}).catch(error=>{report.screenshotError=String(error);});flush();
  }
 }
});
