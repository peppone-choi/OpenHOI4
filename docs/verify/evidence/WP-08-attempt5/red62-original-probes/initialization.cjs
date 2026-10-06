const {chromium,firefox,webkit,expect}=require('E:/openhoi/.orchestrator/wt/WP-08-verify5/target/wp08-verify5/red62-source/client/node_modules/@playwright/test');const fs=require('node:fs'),assert=require('node:assert/strict');
const base='http://127.0.0.1:19443';
(async()=>{const results=[];for(const [name,engine,opt] of [['chrome',chromium,{channel:'chrome'}],['edge',chromium,{channel:'msedge'}],['chromium',chromium,{}],['firefox',firefox,{}],['webkit',webkit,{}]]){
 const browser=await engine.launch(opt);
 try{for(const kind of ['nullGL','nullAdapterGL','shaderFailure']){
  const page=await browser.newPage({viewport:{width:1280,height:720}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(kind=>{
   const p=window.__initProbe={kind,contexts:0,shaderCalls:0,losses:0,rafCalls:0,cancelCalls:0,activeRaf:[],pointerAdds:0,pointerRemoves:0,observerAdds:0,observerDisconnects:0};
   Object.defineProperty(navigator,'gpu',{value:kind==='nullAdapterGL'?{requestAdapter:async()=>null}:undefined});
   const context=HTMLCanvasElement.prototype.getContext;HTMLCanvasElement.prototype.getContext=function(type,...args){if(type==='webgl2'&&kind!=='shaderFailure')return null;const gl=Reflect.apply(context,this,[type,...args]);if(type==='webgl2'&&gl){p.contexts++;this.addEventListener('webglcontextlost',()=>p.losses++);}return gl;};
   if(kind==='shaderFailure'){const create=WebGL2RenderingContext.prototype.createShader;WebGL2RenderingContext.prototype.createShader=function(...args){p.shaderCalls++;throw new Error('independent compile stage injection');};}
   const add=EventTarget.prototype.addEventListener,remove=EventTarget.prototype.removeEventListener;EventTarget.prototype.addEventListener=function(type,...args){if(this instanceof HTMLDivElement&&this.classList.contains('province-map')&&['pointerdown','pointermove','pointerup','wheel'].includes(type))p.pointerAdds++;return Reflect.apply(add,this,[type,...args]);};EventTarget.prototype.removeEventListener=function(type,...args){if(this instanceof HTMLDivElement&&this.classList.contains('province-map')&&['pointerdown','pointermove','pointerup','wheel'].includes(type))p.pointerRemoves++;return Reflect.apply(remove,this,[type,...args]);};
   const raf=requestAnimationFrame,cancel=cancelAnimationFrame;window.requestAnimationFrame=cb=>{p.rafCalls++;let id;id=raf(t=>{p.activeRaf=p.activeRaf.filter(x=>x!==id);cb(t);});p.activeRaf.push(id);return id;};window.cancelAnimationFrame=id=>{p.cancelCalls++;p.activeRaf=p.activeRaf.filter(x=>x!==id);cancel(id);};
   const observe=ResizeObserver.prototype.observe,disconnect=ResizeObserver.prototype.disconnect;ResizeObserver.prototype.observe=function(...args){p.observerAdds++;return Reflect.apply(observe,this,args);};ResizeObserver.prototype.disconnect=function(...args){p.observerDisconnects++;return Reflect.apply(disconnect,this,args);};
  },kind);
  try{
   await page.goto(base);await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(page.getByTestId('province-map').locator('canvas')).toHaveCount(0);await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
   await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');const tick=await page.getByTestId('tick').textContent();await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toContainText('초기화할 수 없습니다');await expect(page.getByTestId('country-panel')).toContainText('북부 시험국');await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('1');await page.waitForTimeout(500);await expect(page.getByTestId('tick')).toHaveText(tick);assert.deepEqual(errors,[]);
   const probe=await page.evaluate(()=>window.__initProbe);assert.equal(probe.activeRaf.length,0);assert.equal(probe.pointerAdds-probe.pointerRemoves,0);assert.equal(probe.observerAdds-probe.observerDisconnects,0);
   if(kind==='shaderFailure'){assert.ok(probe.shaderCalls>0);assert.ok(probe.contexts>0);assert.equal(probe.losses,probe.contexts);}else{assert.equal(probe.contexts,0);assert.equal(probe.shaderCalls,0);}
   await page.screenshot({path:`${__dirname}/${name}-init-${kind}.png`});results.push({name,kind,pass:true,errors,tick,probe});console.log(name,kind,'PASS',JSON.stringify(probe));
  }catch(e){results.push({name,kind,pass:false,error:String(e),errors,probe:await page.evaluate(()=>window.__initProbe).catch(()=>null)});console.error(name,kind,e);process.exitCode=1;}
  finally{fs.writeFileSync(`${__dirname}/initialization-results.json`,JSON.stringify(results,null,2));await page.close();}
 }}finally{await browser.close();}
}})().catch(e=>{console.error(e);process.exitCode=1});
