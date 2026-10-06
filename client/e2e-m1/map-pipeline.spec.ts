import {expect,test} from '@playwright/test';
import {mkdirSync,writeFileSync} from 'node:fs';
import {png} from './png';
const evidence=process.env.OH_MAP_PIPELINE_EVIDENCE??'../target/wp08/pipeline';mkdirSync(evidence,{recursive:true});
for(const glFailure of ['none','unavailable','shader'] as const)test(`REQ-PLAT-03 actual device pipeline rejection ${glFailure==='none'?'falls back to GL pixels':`with GL ${glFailure} shows notice`}`,async({page,browser},info)=>{
 const noGL=glFailure!=='none';
 const errors:string[]=[],consoleErrors:string[]=[];page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')consoleErrors.push(m.text());});
 await page.addInitScript(failure=>{
  const probe={devices:0,destroyed:0,triggers:0,glShaderCalls:0};Object.defineProperty(window,'__gpuFailure',{value:probe});
  if(failure==='unavailable'){const original=HTMLCanvasElement.prototype.getContext;HTMLCanvasElement.prototype.getContext=function(this:HTMLCanvasElement,type:string,...args:unknown[]){return type==='webgl2'?null:Reflect.apply(original,this,[type,...args]);} as typeof original;}
  if(failure==='shader'){const source=WebGL2RenderingContext.prototype.shaderSource;WebGL2RenderingContext.prototype.shaderSource=function(shader,code){probe.glShaderCalls++;source.call(this,shader,code+'\nWP08 invalid shader source');};}
  if(!navigator.gpu)return;const requestAdapter=navigator.gpu.requestAdapter.bind(navigator.gpu);
  navigator.gpu.requestAdapter=async(...args)=>{const adapter=await requestAdapter(...args);if(!adapter)return adapter;const requestDevice=adapter.requestDevice.bind(adapter);adapter.requestDevice=async(...args)=>{const device=await requestDevice(...args);probe.devices++;const destroy=device.destroy.bind(device);device.destroy=()=>{probe.destroyed++;destroy();};device.createRenderPipelineAsync=async()=>{probe.triggers++;throw new Error('WP08 actual pipeline rejection');};return device;};return adapter;};
 },glFailure);
 await page.goto('/');const map=page.getByTestId('province-map');
 if(noGL){await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(map.locator('canvas')).toHaveCount(0);await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toContainText('초기화할 수 없습니다');await page.getByRole('combobox').selectOption('en');}
 else{await expect(map).toHaveAttribute('data-backend','webgl2');await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);await expect(map.locator('canvas')).toHaveCount(1);await expect(page.getByRole('alert')).toHaveCount(0);await page.mouse.move(0,0);const b=(await map.boundingBox())!,c=JSON.parse((await map.getAttribute('data-camera'))!),actual=png(await map.screenshot({path:`${evidence}/${info.project.name}-pipeline-${noGL}.png`})).pixel((1-c.x)/c.width*b.width+b.width/2,(1-c.y)/c.height*b.height+b.height/2);for(let i=0;i<3;i++)expect(Math.abs(actual[i]-[40,100,180][i])).toBeLessThanOrEqual(1);}
 const probe=await page.evaluate(()=>(window as unknown as {__gpuFailure:{devices:number;destroyed:number;triggers:number;glShaderCalls:number}}).__gpuFailure);
 if(info.project.name==='chrome'||info.project.name==='edge')expect(probe.devices).toBeGreaterThan(0);
 if(probe.devices){expect(probe.triggers).toBeGreaterThan(0);expect(probe.destroyed).toBe(probe.devices);expect(consoleErrors.some(e=>e.includes('WP08 actual pipeline rejection'))).toBe(true);}
 if(glFailure==='shader')expect(probe.glShaderCalls).toBeGreaterThan(0);
 await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');const tick=await page.getByTestId('tick').textContent();await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('1');await page.getByRole('combobox').selectOption('ko');await expect(page.getByTestId('tick')).toHaveText(tick!);await expect(page.getByTestId('country-panel')).toContainText('북부 시험국');expect(errors).toEqual([]);
 writeFileSync(`${evidence}/${info.project.name}-pipeline-${glFailure}.json`,JSON.stringify({browser:browser.version(),glFailure,probe,errors,consoleErrors,tick,backend:await map.getAttribute('data-backend'),scope:probe.devices?'actual native GPU device pipeline rejection':'no native adapter; actual GL availability/notice control'},null,2));
});
