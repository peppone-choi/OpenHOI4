import {chromium,firefox,webkit} from '../../client/node_modules/playwright/index.mjs';
import {encode,decode} from '../../client/node_modules/@msgpack/msgpack/dist.esm/index.mjs';
import {spawn} from 'node:child_process';import {writeFileSync,readFileSync,mkdirSync,openSync} from 'node:fs';import assert from 'node:assert/strict';
const root='E:/openhoi/.orchestrator/wt/WP-09-verify',out=root+'/target/wp09-verify/adversarial';mkdirSync(out,{recursive:true});
const server=spawn(root+'/target/debug/oh_server.exe',['--port','19422','--pack-root',root+'/data/packs'],{cwd:root,windowsHide:true,stdio:['ignore',openSync(out+'/server.log','w'),'pipe']});
const url='http://127.0.0.1:19422';let browser;
const cases=[
 ['bad-id-shape',m=>m.world.nations[0].id='1','reject'],['bad-state-shape',m=>m.world.states[0].owner={bad:true},'reject'],['bad-province-shape',m=>m.world.provinces[1].state='1','reject'],['bad-scalar-shape',m=>m.world.states[0].population=1,'reject'],['bad-support-shape',m=>m.world.nations[0].support[0].value=0.75,'reject'],['bad-ledger-row',m=>m.world.states[0].infrastructure.entries[0].accumulated=1,'reject'],['bad-palette',m=>m.world.provinces[0].terrain_color=[256,1,2],'reject'],['bad-world-metadata',m=>m.world.width=-1,'reject'],
 ['unknown-state-owner',m=>m.world.states[0].owner=65535,'safe'],['unknown-province-owner',m=>m.world.provinces[1].owner=65535,'safe'],['unknown-controller',m=>m.world.provinces[1].controller=65535,'safe'],['unknown-state',m=>m.world.provinces[1].state=65535,'safe'],['unknown-capital',m=>m.world.nations[0].capital=65535,'safe'],['empty-nations',m=>m.world.nations=[],'safe'],['empty-states',m=>m.world.states=[],'safe'],['empty-provinces',m=>m.world.provinces=[],'safe'],['empty-ledger',m=>m.world.states[0].infrastructure.entries=[],'safe'],['empty-support',m=>m.world.nations[0].support=[],'safe'],['null-world',m=>m.world=null,'preserve'],['refused-world',m=>{m.supported=false;m.reason_key='unknown-entity';m.world=null},'preserve'],['precision-wire',m=>Object.assign(m,decode(readFileSync(root+'/target/wp09-verify/precise-world.msgpack'))),'precision'],
];const products=[['chrome',chromium,{channel:'chrome'}],['edge',chromium,{channel:'msedge'}],['chromium',chromium,{}],['firefox',firefox,{}],['webkit',webkit,{}]];const rows=[];
try{
 for(let i=0;i<100;i++){try{await fetch(url);break}catch{await new Promise(r=>setTimeout(r,50));}}
 for(const [product,type,options]of products){browser=await type.launch({headless:true,...options});
  for(const [name,change,kind]of cases){const context=await browser.newContext({viewport:{width:1280,height:720}});const page=await context.newPage();const errors=[],requests=[];let worlds=0;
   page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>requests.push(r.url()));
   await page.addInitScript(()=>{window.__ws={close:0,send:0};const c=WebSocket.prototype.close,s=WebSocket.prototype.send;WebSocket.prototype.close=function(...a){window.__ws.close++;return c.apply(this,a)};WebSocket.prototype.send=function(...a){window.__ws.send++;return s.apply(this,a)}});
   await page.routeWebSocket('**/ws',route=>{const backend=route.connectToServer();backend.onMessage(data=>{const message=decode(data);if(message.type==='WorldResult'&&message.world&&++worlds>=3){change(message);route.send(Buffer.from(encode(message)));}else route.send(data)});});
   await page.goto(url);await page.getByTestId('country-panel').waitFor({timeout:10000});await page.waitForFunction(()=>document.querySelector('[data-testid=connection]')?.textContent==='Disconnected',{},{timeout:2500}).catch(()=>{});
   const row={product,name,kind,errors,worlds,connection:await page.getByTestId('connection').textContent(),alert:await page.getByRole('alert').allTextContents(),nation:await page.getByTestId('country-panel').textContent(),state:await page.getByTestId('state-panel').textContent(),ws:await page.evaluate(()=>window.__ws),externalRequests:requests.filter(u=>!u.startsWith(url)&&!u.startsWith('ws://127.0.0.1:19422'))};
   assert.deepEqual(errors,[],`${product}/${name}: render errors`);assert.deepEqual(row.externalRequests,[]);assert.ok(await page.getByTestId('pause').count());
   if(kind==='reject'){assert.equal(row.connection,'Disconnected');assert.equal(row.ws.close,1);assert.equal(await page.getByTestId('pause').isDisabled(),true);assert.deepEqual(row.alert,['Invalid server message; connection closed']);assert.ok(row.nation.includes('Northern Test Nation'));assert.ok(row.state.includes('Northern Test State'));const count=row.ws.send;await page.waitForTimeout(150);assert.equal((await page.evaluate(()=>window.__ws)).send,count);await page.getByRole('combobox').selectOption('ko');assert.equal(await page.getByRole('alert').textContent(),'잘못된 서버 메시지로 연결을 종료했습니다');}
   if(kind==='preserve'){assert.ok(row.nation.includes('Northern Test Nation'));assert.ok(row.state.includes('Northern Test State'));}
   if(kind==='precision'){const panel=page.getByTestId('state-panel');await panel.locator('.ledger-value summary').click();const ledger=decode(readFileSync(root+'/target/wp09-verify/precise-world.msgpack')).world.states[0].infrastructure;assert.equal(await panel.locator('.ledger-value strong').textContent(),ledger.final_value);const cells=await panel.locator('.ledger-tooltip tbody tr td:nth-child(3)').allTextContents();assert.deepEqual(cells,ledger.entries.map(e=>e.value));row.preciseLedger=ledger;}
   if(product==='chromium'&&['unknown-state-owner','empty-nations','bad-ledger-row','precision-wire'].includes(name))await page.screenshot({path:out+'/'+name+'.png',fullPage:true});
   rows.push(row);writeFileSync(out+'/results.json',JSON.stringify({rows},null,2));console.log(product,name,row.connection,'close',row.ws.close,'errors',errors.length);await context.close();
  }await browser.close();browser=null;
 }
}finally{if(browser)await browser.close();server.kill();}
console.log('Independent adversarial cases passed',rows.length);

