const {chromium,firefox,webkit,expect}=require('../../client/node_modules/@playwright/test');
const {encode,decode}=require('../../client/node_modules/@msgpack/msgpack');
const fs=require('node:fs'),assert=require('node:assert/strict');
const base='http://127.0.0.1:19428',out=__dirname;
const products=[['chrome',chromium,{channel:'chrome'}],['edge',chromium,{channel:'msedge'}],['chromium',chromium,{}],['firefox',firefox,{}],['webkit',webkit,{}]];
(async()=>{
 const {png}=await import('../../client/e2e-m1/png.ts');const results=[];
 for(const [name,engine,options] of products){
  const browser=await engine.launch(options),context=await browser.newContext({viewport:{width:1280,height:720},locale:'en-US'}),page=await context.newPage();
  const errors=[],requests=[],messages=[],sent=[];page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>requests.push(r.url()));
  page.on('websocket',ws=>{ws.on('framereceived',f=>{if(Buffer.isBuffer(f.payload))messages.push(decode(f.payload));});ws.on('framesent',f=>{if(Buffer.isBuffer(f.payload))sent.push(decode(f.payload));});});
  const map=page.getByTestId('province-map');const read=async()=>({selected:await page.getByTestId('selected-province').textContent(),country:await page.getByTestId('country-panel').locator('h3').textContent(),state:await page.getByTestId('state-panel').locator('h3').textContent(),ledger:await page.getByTestId('state-panel').locator('.ledger-value strong').textContent(),date:await page.getByTestId('date').textContent(),hour:await page.getByTestId('hour').textContent(),tick:await page.getByTestId('tick').textContent(),worldTick:await page.getByTestId('national-panels').getAttribute('data-world-tick'),camera:await map.getAttribute('data-camera'),backend:await map.getAttribute('data-backend')});
  const click=async(x,y)=>{const b=await map.boundingBox(),c=JSON.parse(await map.getAttribute('data-camera'));await map.click({position:{x:(x-c.x)/c.width*b.width+b.width/2,y:(y-c.y)/c.height*b.height+b.height/2}});};
  try{
   await page.goto(base);await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
   const states=[];
   for(const lang of ['en','ko']){
    await page.getByRole('combobox').selectOption(lang);
    for(const id of [10,30]){
     await click(1,id===10?1:3);await expect(page.getByTestId('selected-province')).toHaveText(String(id));await expect(page.getByTestId('map-hover')).toHaveText(lang==='en'?`Province ${id} · Plains`:`프로빈스 ${id} · 평야`);
     await expect(page.getByTestId('country-panel').locator('h3')).toHaveText(lang==='en'?(id===10?'Northern Test Nation':'Southern Test Nation'):(id===10?'북부 시험국':'남부 시험국'));
     await expect(page.getByTestId('state-panel').locator('h3')).toHaveText(lang==='en'?(id===10?'Northern Test State':'Southern Test State'):(id===10?'북부 시험주':'남부 시험주'));
     await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText(id===10?'1':'0');
     await page.getByTestId('state-panel').getByText(lang==='en'?'Infrastructure':'기반시설',{exact:true}).click();await expect(page.getByTestId('state-panel').locator('.ledger-tooltip')).toBeVisible();
     await page.screenshot({path:`${out}/${name}-${lang}-province${id}.png`,fullPage:true});states.push({language:lang,province:id,ui:await read(),tooltip:await page.getByTestId('state-panel').locator('.ledger-tooltip').textContent()});
    }
   }
   await page.getByRole('combobox').selectOption('en');await page.mouse.move(0,0);
   const box=await map.boundingBox();await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.wheel(0,-200);await expect.poll(async()=>JSON.parse(await map.getAttribute('data-camera')).zoom).toBeGreaterThan(1);
   await page.mouse.down();await page.mouse.move(box.x+box.width/2+20,box.y+box.height/2+10,{steps:4});await page.mouse.up();await page.mouse.move(0,0);
   const before=await read();await page.getByRole('combobox').selectOption('ko');await expect(map).toHaveAttribute('data-camera',before.camera);await page.getByRole('combobox').selectOption('en');assert.deepEqual(await read(),before);
   for(const s of [1,2,3,4,5]){await page.getByTestId(`speed-${s}`).click();await expect(page.getByTestId('speed')).toHaveText(String(s));await expect(map).toHaveAttribute('data-camera',before.camera);await expect(page.getByTestId('selected-province')).toHaveText('30');}
   await page.getByTestId('pause').click();await expect(page.getByTestId('tick')).not.toHaveText(before.tick);await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');const afterTime=await read();assert.equal(afterTime.camera,before.camera);assert.equal(afterTime.selected,'30');assert.equal(afterTime.ledger,'0');
   await page.setViewportSize({width:1100,height:800});await expect.poll(async()=>JSON.parse(await map.getAttribute('data-camera')).width).not.toBe(JSON.parse(before.camera).width);const resized=await read();const c0=JSON.parse(before.camera),c1=JSON.parse(resized.camera);assert.equal(c1.zoom,c0.zoom);assert.equal(c1.x,c0.x);assert.equal(c1.y,c0.y);assert.equal(resized.selected,'30');
   await page.setViewportSize({width:1280,height:720});await expect(map).toHaveAttribute('data-camera',before.camera);await page.getByRole('button',{name:'Reset view'}).click();await page.mouse.move(0,0);
   const meta=await (await page.request.get(base+'/maps/testland/metadata')).json(),raw=await (await page.request.get(base+`/maps/testland/index.bin?pack=${meta.pack_hash}`)).body();assert.equal(raw.length,96);assert.equal(meta.schema_version,1);assert.deepEqual(meta.province_ids,[10,20,30,40,50,60]);assert.equal(meta.byte_length,'96');
   const initial=messages.find(m=>m.type==='WorldResult'&&m.request==='world'&&m.world).world;
   const oracle=[];for(const [mode,key] of [['Ownership','owner_color'],['Terrain','terrain_color'],['States','state_color'],['Control','controller_color']]){
    await page.getByRole('button',{name:mode,exact:true}).click();await page.getByRole('button',{name:'Clear selection'}).click().catch(()=>{});await page.mouse.move(0,0);await page.waitForTimeout(100);const b=await map.boundingBox(),cam=JSON.parse(await map.getAttribute('data-camera')),image=png(await map.screenshot({path:`${out}/${name}-oracle-${mode}.png`}));
    for(const [x,y] of [[1,1],[3,1],[1,3],[3,3],[1,5],[5,5]]){const dense=raw.readUInt16LE((y*meta.width+x)*2),id=meta.province_ids[dense],p=initial.provinces.find(p=>p.id===id),expected=p[key]??initial.neutral_color,actual=image.pixel((x-cam.x)/cam.width*b.width+b.width/2,(y-cam.y)/cam.height*b.height+b.height/2);for(let i=0;i<3;i++)assert.ok(Math.abs(actual[i]-expected[i])<=1,`${name} ${mode} ${id} ${actual} expected ${expected}`);oracle.push({mode,id,expected,actual});}
   }
   assert.equal(requests.filter(u=>new URL(u).origin!==base).length,0);assert.deepEqual(errors,[]);assert.ok(sent.some(m=>m.type==='Query'&&m.kind==='nation:2'));assert.ok(sent.some(m=>m.type==='Query'&&m.kind==='state:2'));
   const result={name,browser:browser.version(),head:'2c2aacc88bd22b0f3ba4393b22bef4b119616e69',scenario:'m1',seed:1,states,before,afterTime,resized,oracle,requests,errors,sent,diagnostics:await map.evaluate(el=>({...el.dataset})),scope:'actual GUI actions, no game state or GPU capability mutation; screenshots and server wire palette oracle'};fs.writeFileSync(`${out}/${name}-independent.json`,JSON.stringify(result,null,2));results.push({name,pass:true});console.log(name,'GUI continuity and server palette PASS');
  }finally{await browser.close();}
 }
 fs.writeFileSync(`${out}/ui-independent-results.json`,JSON.stringify(results,null,2));
})().catch(e=>{console.error(e);process.exitCode=1});
