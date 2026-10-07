// Isolated headless page only. Never attaches to existing tabs or global input.
import { chromium } from '../../client/node_modules/playwright-core/index.mjs';
import { mkdir,mkdtemp,readFile,writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
const evidence=resolve(process.argv.find(a=>a.startsWith('--out='))?.slice(6)??'target/evidence/WORLD-PREVIEW-quality-P06-2/capture');await mkdir(evidence,{recursive:true});
const url=process.argv[2]??'http://127.0.0.1:4317/?world-preview=1';
const profile=await mkdtemp(resolve(evidence,'chrome-profile-'));
// Preserve browser security/GPU defaults: no Playwright default feature switches
// or --no-sandbox. Only isolated headless/profile/pipe/no-window launch arguments.
const server=await chromium.launchServer({host:'127.0.0.1',channel:'chrome',chromiumSandbox:true,ignoreDefaultArgs:true,args:['--headless',`--user-data-dir=${profile}`,'--remote-debugging-pipe','--no-startup-window']});
const processInfo={pid:server.process().pid,argv:server.process().spawnargs,cwd:process.cwd(),headless:true,started:new Date().toISOString(),url,rendererHead:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),rendererStatus:execFileSync('git',['status','--porcelain'],{encoding:'utf8'})};
await writeFile(resolve(evidence,'browser-process.json'),JSON.stringify(processInfo,null,2));
const browser=await chromium.connect(server.wsEndpoint());
const context=await browser.newContext({viewport:{width:1600,height:1000},deviceScaleFactor:1});
const page=await context.newPage();const consoleMessages=[],requests=[],errors=[],badResponses=[],failedRequests=[];
const resourceLifecycle=[];await page.exposeFunction('__previewResourceTrace',detail=>resourceLifecycle.push(detail));
await page.addInitScript(()=>document.addEventListener('openhoi-preview-resource',event=>window.__previewResourceTrace(event.detail)));
const inputDir=process.argv.find(a=>a.startsWith('--input-dir='))?.slice(12);
if(inputDir){
 for(const file of ['metadata.json','index.bin','provinces.json']){
  const bytes=await readFile(resolve(inputDir,file));
  await page.route(`**/preview/world/${file}`,route=>route.fulfill({status:200,contentType:file.endsWith('.json')?'application/json':'application/octet-stream',body:bytes}));
 }
}
const baseline=process.argv.find(a=>a.startsWith('--baseline='))?.slice(11);
const historicalInputs=[];
if(baseline){
 if(!/^[a-f0-9]{40}$/.test(baseline))throw new Error('exact baseline commit SHA required');
 for(const file of ['metadata.json','index.bin','provinces.json']){
  const bytes=execFileSync('git',['show',`${baseline}:client/public/preview/world/${file}`],{maxBuffer:32*1024*1024});
  historicalInputs.push({file,commit:baseline,sha256:createHash('sha256').update(bytes).digest('hex')});
  await page.route(`**/preview/world/${file}`,route=>route.fulfill({status:200,contentType:file.endsWith('.json')?'application/json':'application/octet-stream',body:bytes}));
 }
}
page.on('console',m=>consoleMessages.push({type:m.type(),text:m.text(),location:m.location()}));page.on('pageerror',e=>errors.push(e.message));
page.on('request',r=>requests.push({url:r.url(),type:r.resourceType()}));
page.on('response',r=>{if(r.status()>=400)badResponses.push({url:r.url(),status:r.status()});});
page.on('requestfailed',r=>failedRequests.push({url:r.url(),failure:r.failure()}));
const snapshots=[];
const actions=[];let browserPeakWorkingSet=null;
function observeBrowserMemory(){if(process.platform==='win32'){const value=Number(execFileSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',`(Get-Process -Id ${server.process().pid}).PeakWorkingSet64`],{encoding:'utf8',windowsHide:true}).trim());browserPeakWorkingSet=Math.max(browserPeakWorkingSet??0,value);}}
const ready=()=>page.waitForFunction(()=>{const h=document.querySelector('[data-testid="world-map"]');return Number(h?.getAttribute('data-frames'))>3&&!document.querySelector('[role="status"]');},{},{timeout:90000});
async function capture(name){await ready();await page.screenshot({path:resolve(evidence,`${name}.png`)});observeBrowserMemory();snapshots.push({name,host:await page.locator('[data-testid="world-map"]').evaluate(el=>({...el.dataset})),viewport:page.viewportSize()});}
let exit=0;
try{
 await page.goto(url);await capture('world');
 await page.getByRole('button',{name:'유럽',exact:true}).click();await capture('europe');
 if(process.argv.includes('--compare')){
  await page.getByRole('button',{name:'한반도',exact:true}).click();await capture('korea');
  await page.getByRole('button',{name:'히말라야',exact:true}).click();await capture('himalaya-geography');
 }
 if(process.argv.includes('--full')){
  const host=page.locator('[data-testid="world-map"]');
  await page.mouse.move(650,480);await page.waitForFunction(()=>document.querySelector('[data-testid="world-map"]')?.getAttribute('data-hover'));
  const hover=await host.getAttribute('data-hover');await page.mouse.click(650,480);
  await page.waitForFunction(()=>document.querySelector('[data-testid="world-map"]')?.getAttribute('data-selected'));
  actions.push({action:'hover/select',hover,selected:await host.getAttribute('data-selected')});await capture('europe-selected');
  const before=JSON.parse(await host.getAttribute('data-camera'));
  await page.mouse.move(600,550);await page.mouse.wheel(0,-330);
  await page.waitForFunction(zoom=>JSON.parse(document.querySelector('[data-testid="world-map"]').getAttribute('data-camera')).zoom>zoom,before.zoom);
  const zoomed=JSON.parse(await host.getAttribute('data-camera'));
  await page.mouse.move(600,550);await page.mouse.down();await page.mouse.move(730,600,{steps:10});await page.mouse.up();
  const panned=JSON.parse(await host.getAttribute('data-camera'));if(panned.x===zoomed.x||panned.y===zoomed.y)throw new Error('pan did not move camera');
  actions.push({action:'wheel/pan',before,zoomed,panned});await capture('europe-zoom-pan');
  await page.getByRole('button',{name:'프로빈스 경계',exact:true}).click();await ready();
  const after=JSON.parse(await host.getAttribute('data-camera'));if(Math.abs(after.x-panned.x)>.01||Math.abs(after.y-panned.y)>.01||Math.abs(after.zoom-panned.zoom)>.01)throw new Error('border recreation lost camera');
  actions.push({action:'border-off',camera:after});await capture('borders-off');
  await page.getByRole('button',{name:'프로빈스 경계',exact:true}).click();await ready();
  await page.getByRole('button',{name:'프로빈스',exact:true}).click();await capture('province-colors');
  await page.getByRole('button',{name:'지리',exact:true}).click();
  await page.getByRole('button',{name:'한반도',exact:true}).click();await capture('korea');
  await page.getByRole('button',{name:'히말라야',exact:true}).click();await ready();
  await page.getByRole('button',{name:'지형',exact:true}).click();await page.mouse.move(800,500);await page.mouse.click(800,500);await capture('himalaya-terrain');
  await page.getByRole('button',{name:'미주',exact:true}).click();await capture('americas');
  await page.getByRole('button',{name:'전세계',exact:true}).click();await ready();
  await page.getByRole('button',{name:'3D 샘플',exact:true}).click();await ready();
  if(await host.getAttribute('data-unit-samples')!=='0')throw new Error('hidden samples remain in scene');await capture('samples-off');
  await page.getByRole('button',{name:'3D 샘플',exact:true}).click();await ready();
  await page.setViewportSize({width:850,height:650});await capture('resize');
  const rail=await page.locator('.preview-rail').evaluate(el=>{el.scrollTop=el.scrollHeight;const box=el.getBoundingClientRect(),last=el.querySelector('button:last-of-type').getBoundingClientRect();return {top:box.top,bottom:box.bottom,height:box.height,scrollHeight:el.scrollHeight,scrollTop:el.scrollTop,lastTop:last.top,lastBottom:last.bottom,viewportHeight:innerHeight,documentWidth:document.documentElement.scrollWidth,viewportWidth:innerWidth};});
  if(rail.bottom>612||rail.lastTop<rail.top||rail.lastBottom>rail.bottom||rail.documentWidth>rail.viewportWidth)throw new Error('small viewport rail not fully reachable: '+JSON.stringify(rail));
  actions.push({action:'small-viewport-rail-reachability',...rail});await capture('resize-rail-scrolled');
  await page.setViewportSize({width:1280,height:720});await capture('resize-1280x720');
  await page.setViewportSize({width:1600,height:1000});await ready();
  await page.getByRole('button',{name:'언어',exact:true}).click();await capture('english');
  await page.goto(url+'&forceWebGL=1');await capture('world-webgl2');
  if(await host.getAttribute('data-backend')!=='webgl2')throw new Error('explicit WebGL2 not presented');
  await page.getByRole('button',{name:'한반도',exact:true}).click();await capture('korea-webgl2');
  await page.setViewportSize({width:1280,height:800});await capture('resize-webgl2');
  const bad=badResponses.filter(r=>r.status>=400),consoleErrors=consoleMessages.filter(m=>m.type==='error');
  if(bad.length||consoleErrors.length||errors.length)throw new Error('browser errors present: '+JSON.stringify({bad,consoleErrors,errors}));
  const foreign=requests.filter(r=>!r.url.startsWith(new URL(url).origin));if(foreign.length)throw new Error('external runtime requests');
 }
}catch(e){exit=1;errors.push(String(e));await page.screenshot({path:resolve(evidence,'failure.png')});}
await context.close();await browser.close();await server.close();
const browserNativeExit=server.process().exitCode;
if(browserNativeExit!==0){exit=1;errors.push(`browser native exit ${browserNativeExit}`);}
const expectedCancellations=failedRequests.filter(r=>r.failure?.errorText==='net::ERR_ABORTED'&&resourceLifecycle.some(e=>e.phase==='cancel'&&e.signalAborted&&new URL(e.path,url).href===r.url));
const unexplainedFailures=failedRequests.filter(r=>!expectedCancellations.includes(r));
if(unexplainedFailures.length){exit=1;errors.push('unexplained request failures: '+JSON.stringify(unexplainedFailures));}
await writeFile(resolve(evidence,'browser-capture.json'),JSON.stringify({processInfo,browserNativeExit,browserPeakWorkingSet,memoryNote:'Browser parent PID peak working set only; excludes renderer/GPU subprocesses. Not total browser/GPU/JS memory.',consoleMessages,requests,badResponses,failedRequests,resourceLifecycle,expectedCancellations,unexplainedFailures,errors,snapshots,actions,historicalInputs,inputDir,comparisonNote:baseline?'Historical baked map inputs only; current renderer/HUD. Not a capture of the complete old source tree.':undefined,exit},null,2));
console.log(JSON.stringify({exit,errors,snapshots}));process.exitCode=exit;
