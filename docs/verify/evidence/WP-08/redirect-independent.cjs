const {chromium,firefox,webkit,expect}=require('../../client/node_modules/@playwright/test');
const http=require('node:http'),net=require('node:net'),fs=require('node:fs'),assert=require('node:assert/strict');
const base='http://127.0.0.1:19415',out=__dirname+'/independent-redirect';fs.mkdirSync(out,{recursive:true});
async function listen(s){await new Promise(r=>s.listen(0,'127.0.0.1',r));return 'http://127.0.0.1:'+s.address().port}
async function close(s){await new Promise(r=>{s.close(r);s.closeAllConnections()})}
(async()=>{
 const meta=await(await fetch(base+'/maps/testland/metadata')).json(),bytes=Buffer.from(await(await fetch(base+'/maps/testland/index.bin?pack='+meta.pack_hash)).arrayBuffer());
 let foreignHits=[],sourceHits=[],state;const sockets=new Set();
 const foreign=http.createServer((req,res)=>{foreignHits.push(req.url);res.writeHead(200,{'access-control-allow-origin':'*','access-control-expose-headers':'x-pack-hash','x-pack-hash':meta.pack_hash,'content-type':state.resource==='metadata'?'application/json':'application/octet-stream'});res.end(state.resource==='metadata'?JSON.stringify(meta):bytes)});
 const foreignOrigin=await listen(foreign);
 const proxy=http.createServer((req,res)=>{
  const p=new URL(req.url,'http://source.invalid');
  if(p.pathname==='/control'||(p.pathname==='/maps/testland/'+(state.resource==='metadata'?'metadata':'index.bin')&&!p.searchParams.has('alias'))){
   sourceHits.push({url:req.url,status:state.status,location:state.location});res.writeHead(state.status,{location:state.location});res.end();return;
  }
  const request=http.request(base+req.url,{method:req.method,headers:{...req.headers,host:new URL(base).host}},response=>{res.writeHead(response.statusCode,response.headers);response.pipe(res)});request.on('error',e=>{res.writeHead(502);res.end(String(e))});req.pipe(request);
 });
 proxy.on('connection',s=>{sockets.add(s);s.on('close',()=>sockets.delete(s))});
 proxy.on('upgrade',(req,socket,head)=>{const upstream=net.connect(19415,'127.0.0.1',()=>{upstream.write(`${req.method} ${req.url} HTTP/${req.httpVersion}\r\n${req.rawHeaders.reduce((a,v,i,all)=>{if(i%2===0)a.push(`${v}: ${all[i+1]}`);return a},[]).join('\r\n')}\r\n\r\n`);if(head.length)upstream.write(head);socket.pipe(upstream);upstream.pipe(socket)});sockets.add(upstream);upstream.on('close',()=>sockets.delete(upstream));upstream.on('error',()=>socket.destroy());socket.on('error',()=>upstream.destroy());socket.on('close',()=>upstream.destroy())});
 const origin=await listen(proxy);const results=[];
 try{
  for(const [name,engine,options] of [['chrome',chromium,{channel:'chrome'}],['edge',chromium,{channel:'msedge'}],['chromium',chromium,{}],['firefox',firefox,{}],['webkit',webkit,{}]]){
   const browser=await engine.launch(options);
   try{
    for(const resource of ['metadata','index'])for(const sameOrigin of [false,true])for(const status of [301,302,303,307,308]){
     const endpoint=resource==='metadata'?'/maps/testland/metadata':'/maps/testland/index.bin?pack='+meta.pack_hash;
     state={resource,status,location:sameOrigin?endpoint+(resource==='metadata'?'?':'&')+'alias=1':foreignOrigin+'/payload'};
     const context=await browser.newContext({viewport:{width:1280,height:720},locale:'en-US'}),page=await context.newPage(),requests=[],failures=[],errors=[];
     try{
      await page.goto(origin+'/fonts/OFL.txt');
      const control=await page.evaluate(async()=>{const r=await fetch('/control',{mode:'cors',redirect:'follow'});return {status:r.status,url:r.url,redirected:r.redirected,pack:r.headers.get('x-pack-hash'),body:Array.from(new Uint8Array(await r.arrayBuffer()))}});
      assert.equal(control.status,200);assert.equal(control.redirected,true);
      assert.deepEqual(Buffer.from(control.body),resource==='metadata'?Buffer.from(JSON.stringify(meta)):bytes);
      if(!sameOrigin){assert.equal(foreignHits.length,1);assert.equal(control.pack,meta.pack_hash)}
      foreignHits=[];sourceHits=[];
      page.on('request',r=>requests.push(r.url()));page.on('requestfailed',r=>failures.push({url:r.url(),error:r.failure()}));page.on('pageerror',e=>errors.push(e.message));
      await page.goto(origin+'/?forceWebGL=1');
      await expect(page.getByRole('alert')).toHaveText('Map data could not be loaded. Check the bundled scenario files.');
      await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);
      await expect(page.getByTestId('connection')).toHaveText('Connected');
      await expect(page.getByTestId('country-panel').locator('h3')).toHaveText('Northern Test Nation');
      await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('Northern Test State');
      assert.equal(foreignHits.length,0);assert.equal(sourceHits.length,1);assert.equal(sourceHits[0].status,status);
      assert(requests.every(u=>new URL(u).origin===origin));assert(!requests.some(u=>new URL(u).searchParams.has('alias')));
      await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toHaveText('지도 데이터를 불러올 수 없습니다. 동봉된 시나리오 파일을 확인하세요.');
      const expectedDiagnostic=origin.replace(/^http:\//,'')+endpoint+' due to access control checks.';
      assert.deepEqual(errors.filter(e=>!(name==='webkit'&&e===expectedDiagnostic)),[]);
      const record={name,version:browser.version(),resource,sameOrigin,status,control:{...control,body:undefined,bodyLength:control.body.length},foreignHits,sourceHits,requests,failures,errors,alerts:await page.getByRole('alert').allTextContents(),canvas:0};results.push(record);
      fs.writeFileSync(out+'/'+name+'-'+resource+'-'+sameOrigin+'-'+status+'.json',JSON.stringify(record,null,2));
      if(status===308&&!sameOrigin)await page.screenshot({path:out+'/'+name+'-'+resource+'.png'});
     }finally{await context.close()}
    }
   }finally{await browser.close()}
   console.log(name+' 20 redirect cases PASS');
  }
  fs.writeFileSync(out+'/summary.json',JSON.stringify({commit:'87c1a99a30de25732376d628958c68d37ce64cda',cases:results.length,allPassed:true,statuses:[301,302,303,307,308],basis:'native HTTP Location + browser follow positive control with exact payload; source count1, follow count0, localized errors, canvas0, authority panels'},null,2));
 }finally{for(const s of sockets)s.destroy();await close(proxy);await close(foreign)}
})().catch(e=>{console.error(e);process.exitCode=1});
