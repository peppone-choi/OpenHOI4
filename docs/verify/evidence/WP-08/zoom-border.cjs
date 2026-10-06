const {chromium,expect}=require('../../client/node_modules/@playwright/test');const {decode,encode}=require('../../client/node_modules/@msgpack/msgpack'),fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const {displayFixture}=await import('../../client/e2e-m1/displayFixture.ts'),{png}=await import('../../client/e2e-m1/png.ts');
 const base='http://127.0.0.1:19415',meta=await(await fetch(base+'/maps/testland/metadata')).json();const out=__dirname+'/zoom-border';fs.mkdirSync(out,{recursive:true});
 const browser=await chromium.launch({channel:'chrome'});try{
  for(const force of [false,true]){
   const page=await browser.newPage({viewport:{width:1280,height:720},locale:'en-US'});let fixture;
   await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{const m=decode(data);if(m.type==='WorldResult'&&m.world&&m.request==='world'){fixture=displayFixture(m.world,meta);m.world=fixture.world;route.send(Buffer.from(encode(m)))}else route.send(data)})});
   await page.route('**/maps/display_fixture/metadata',r=>r.fulfill({json:fixture.meta}));await page.route('**/maps/display_fixture/index.bin?*',r=>r.fulfill({body:fixture.bytes,headers:{'x-pack-hash':meta.pack_hash}}));
   await page.goto(base+(force?'/?forceWebGL=1':'/'));const map=page.getByTestId('province-map');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);const records=[];
   for(let step=0;step<2;step++){
    if(step){const b=await map.boundingBox();await page.mouse.move(b.x+b.width/2,b.y+b.height/2);await page.mouse.wheel(0,-300);await expect.poll(async()=>JSON.parse(await map.getAttribute('data-camera')).zoom).toBeGreaterThan(1)}
    await page.mouse.move(0,0);const c=JSON.parse(await map.getAttribute('data-camera')),b=await map.boundingBox(),image=png(await map.screenshot({path:out+'/'+force+'-'+step+'.png'})),samples=[];
    for(const [x,color,milli] of [[2,meta.style.province_border,meta.style.province_width_milli],[4,meta.style.state_border,meta.style.state_width_milli],[6,meta.style.nation_border,meta.style.nation_width_milli]]){
     const px=(x-c.x)/c.width*b.width+b.width/2,py=(1-c.y)/c.height*b.height+b.height/2;let width=0;for(let dx=-8;dx<=8;dx++)if(image.pixel(px+dx,py).every((v,i)=>Math.abs(v-color[i])<=1))width++;
     const nominal=2*milli/1000;assert(width>0);assert(Math.abs(width-nominal)<=1,JSON.stringify({x,width,nominal,zoom:c.zoom}));samples.push({x,color,width,nominal,rasterError:Math.abs(width-nominal)});
    }records.push({camera:c,samples});
   }
   fs.writeFileSync(out+'/'+force+'.json',JSON.stringify({force,diagnostics:await map.evaluate(el=>({...el.dataset})),records},null,2));console.log(force?'actual WebGL2':'actual preferred',JSON.stringify(records));await page.close();
  }
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
