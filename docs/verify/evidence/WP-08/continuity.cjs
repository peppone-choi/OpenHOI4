const {chromium,firefox,webkit,expect}=require('../../client/node_modules/@playwright/test'),fs=require('node:fs'),assert=require('node:assert/strict');
const out=__dirname+'/continuity';fs.mkdirSync(out,{recursive:true});
(async()=>{
 for(const [name,engine,options] of [['chrome',chromium,{channel:'chrome'}],['edge',chromium,{channel:'msedge'}],['chromium',chromium,{}],['firefox',firefox,{}],['webkit',webkit,{}]]){
  const browser=await engine.launch(options),page=await browser.newPage({viewport:{width:1280,height:720},locale:'en-US'}),requests=[],errors=[];
  page.on('request',r=>requests.push(r.url()));page.on('pageerror',e=>errors.push(e.message));
  try{
   await page.goto('http://127.0.0.1:19415/');const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);
   if(await page.getByTestId('pause').textContent()==='Pause'){await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume')}
   const camera=()=>map.getAttribute('data-camera').then(JSON.parse);
   const at=async(x,y)=>{const c=await camera(),b=await map.boundingBox();return {x:(x-c.x)/c.width*b.width+b.width/2,y:(y-c.y)/c.height*b.height+b.height/2}};
   await map.click({position:await at(1,3)});await expect(page.getByTestId('selected-province')).toHaveText('30');await expect(page.getByTestId('state-panel').locator('h3')).toHaveText('Southern Test State');
   const b=await map.boundingBox();await page.mouse.move(b.x+b.width/2,b.y+b.height/2);await page.mouse.wheel(0,-300);await expect.poll(async()=>(await camera()).zoom).toBeGreaterThan(1);await page.mouse.move(0,0);
   const initial=await camera(),frame=Number(await map.getAttribute('data-frames')),tick=await page.getByTestId('tick').textContent();
   for(const language of ['ko','en']){
    await page.getByRole('combobox').selectOption(language);await expect(page.getByTestId('state-panel').locator('h3')).toHaveText(language==='ko'?'남부 시험주':'Southern Test State');assert.deepEqual(await camera(),initial);await expect(page.getByTestId('selected-province')).toHaveText('30');
   }
   await page.getByTestId('pause').click();await expect.poll(()=>page.getByTestId('tick').textContent()).not.toBe(tick);await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');assert.deepEqual(await camera(),initial);await expect(page.getByTestId('selected-province')).toHaveText('30');
   const worldTick=await page.getByTestId('national-panels').getAttribute('data-world-tick');assert(BigInt(worldTick)>=BigInt(tick));
   await page.setViewportSize({width:900,height:720});await expect.poll(async()=> (await camera()).width).not.toBe(initial.width);const resized=await camera();for(const k of ['x','y','zoom'])assert.equal(resized[k],initial[k]);await expect(page.getByTestId('selected-province')).toHaveText('30');
   assert(Number(await map.getAttribute('data-frames'))>frame);assert.deepEqual(errors,[]);assert(requests.every(u=>new URL(u).origin==='http://127.0.0.1:19415'));
   await page.screenshot({path:out+'/'+name+'.png'});fs.writeFileSync(out+'/'+name+'.json',JSON.stringify({name,version:browser.version(),initial,resized,tick,worldTick,selected:'30',errors,requests,diagnostics:await map.evaluate(el=>({...el.dataset}))},null,2));console.log(name+' time/world/ko/en/resize/selection/frame PASS');
  }finally{await browser.close()}
 }
})().catch(e=>{console.error(e);process.exitCode=1});
