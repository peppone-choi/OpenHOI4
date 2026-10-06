const path = require('node:path');
const fs = require('node:fs');
const assert = require('node:assert/strict');
const {spawn} = require('node:child_process');
const net = require('node:net');
const root = process.cwd();
const dest = path.join(root,'target/evidence/WP-05-verify');
const {chromium,firefox,webkit} = require(path.join(root,'client/node_modules/playwright'));
const {decode} = require(path.join(root,'client/node_modules/@msgpack/msgpack'));
(async()=>{
 const reserve=net.createServer(); await new Promise(r=>reserve.listen(0,'127.0.0.1',r)); const port=reserve.address().port; await new Promise(r=>reserve.close(r));
 const server=spawn(path.join(root,'target/debug/oh_server.exe'),['--port',String(port)],{cwd:root,windowsHide:true});
 const results=[];
 try {
  for(let i=0;i<100;i++){try {if((await fetch(`http://127.0.0.1:${port}/`)).ok)break;}catch{} await new Promise(r=>setTimeout(r,50));}
  for(const [name,engine,channel] of [['chromium',chromium],['firefox',firefox],['webkit',webkit],['chrome',chromium,'chrome'],['edge',chromium,'msedge']]){
   const browser=await engine.launch({headless:true,...(channel?{channel}:{})});
   try {
    const page=await browser.newPage(); let state;
    page.on('websocket',ws=>ws.on('framereceived',({payload})=>{if(Buffer.isBuffer(payload)){const m=decode(payload); if(m.type==='Snapshot'||m.type==='Delta')state=m.state;}}));
    await page.goto(`http://127.0.0.1:${port}/`); await page.getByTestId('pause').click();
    await page.getByTestId('pause').getByText('Resume',{exact:true}).waitFor();
    await page.waitForTimeout(350); assert.equal(state.paused,true);
    const shown={}; for(const field of ['date','hour','tick','speed']){shown[field]=await page.getByTestId(field).textContent(); assert.equal(shown[field],String(state[field]));}
    results.push({name,version:browser.version(),wireState:state,display:shown,result:'PASS'}); console.log(name+' exact server date/hour/tick/speed: PASS');
   }finally{await browser.close();}
  }
  fs.writeFileSync(path.join(dest,'exact-display.json'),JSON.stringify(results,null,2));
 }finally{server.kill();}
})().catch(e=>{console.error(e);process.exitCode=1;});
