// Isolated headless page only. Never attaches to existing tabs or global input.
import { chromium } from '../../client/node_modules/playwright-core/index.mjs';
import { mkdir,writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const evidence=resolve('target/evidence/WORLD-PREVIEW');await mkdir(evidence,{recursive:true});
const url=process.argv[2]??'http://127.0.0.1:4317/?world-preview=1';
const server=await chromium.launchServer({headless:true,channel:'chrome'});
const processInfo={pid:server.process().pid,argv:server.process().spawnargs,cwd:process.cwd(),headless:true,started:new Date().toISOString(),url};
await writeFile(resolve(evidence,'browser-process.json'),JSON.stringify(processInfo,null,2));
const browser=await chromium.connect(server.wsEndpoint());
const context=await browser.newContext({viewport:{width:1600,height:1000},deviceScaleFactor:1});
const page=await context.newPage();const consoleMessages=[],requests=[],errors=[];
page.on('console',m=>consoleMessages.push({type:m.type(),text:m.text()}));page.on('pageerror',e=>errors.push(e.message));
page.on('request',r=>requests.push({url:r.url(),type:r.resourceType()}));
const snapshots=[];
const ready=()=>page.waitForFunction(()=>{const h=document.querySelector('[data-testid="world-map"]');return Number(h?.getAttribute('data-frames'))>3&&!document.querySelector('[role="status"]');},{},{timeout:90000});
async function capture(name){await ready();await page.screenshot({path:resolve(evidence,`${name}.png`)});snapshots.push({name,host:await page.locator('[data-testid="world-map"]').evaluate(el=>({...el.dataset})),viewport:page.viewportSize()});}
let exit=0;
try{
 await page.goto(url);await capture('world');
 await page.getByRole('button',{name:'유럽',exact:true}).click();await capture('europe');
}catch(e){exit=1;errors.push(String(e));await page.screenshot({path:resolve(evidence,'failure.png')});}
await writeFile(resolve(evidence,'browser-capture.json'),JSON.stringify({processInfo,consoleMessages,requests,errors,snapshots,exit},null,2));
await context.close();await browser.close();await server.close();
console.log(JSON.stringify({exit,errors,snapshots}));process.exitCode=exit;
