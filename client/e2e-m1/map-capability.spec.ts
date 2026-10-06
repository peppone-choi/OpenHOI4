import {expect,test} from '@playwright/test';
import {mkdirSync,writeFileSync} from 'node:fs';
import {png} from './png';

const evidence=process.env.OH_MAP_CAPABILITY_EVIDENCE??'../target/wp08/capability';mkdirSync(evidence,{recursive:true});
test('REQ-PLAT-03 native backend capability renders pixels or preserves authority with localized notice',async({page,browser},info)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 // This local static page has the same secure origin, without modifying GPU
 // APIs, preferences, launch flags, server messages or game data.
 await page.goto('/fonts/OFL.txt');
 const capability=await page.evaluate(async()=>{
  const canvas=document.createElement('canvas');const creationErrors:string[]=[];
  canvas.addEventListener('webglcontextcreationerror',e=>creationErrors.push((e as WebGLContextEvent).statusMessage));
  const gl=canvas.getContext('webgl2',{antialias:false,alpha:false});
  let pixel:number[]|null=null,renderer:string|null=null,glError:number|null=null;
  if(gl){gl.clearColor(37/255,91/255,173/255,1);gl.clear(gl.COLOR_BUFFER_BIT);const bytes=new Uint8Array(4);gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,bytes);pixel=Array.from(bytes);glError=gl.getError();renderer=String(gl.getParameter(gl.RENDERER));gl.getExtension('WEBGL_lose_context')?.loseContext();}
  let adapterAvailable=false,adapterError:string|null=null;
  if(navigator.gpu){try{adapterAvailable=!!await navigator.gpu.requestAdapter();}catch(e){adapterError=String(e);}}
  return {secureContext:window.isSecureContext,webgl2:!!gl,creationErrors,pixel,glError,renderer,webgpu:!!navigator.gpu,adapterAvailable,adapterError};
 });
 if(capability.webgl2){expect(capability.pixel).toEqual([37,91,173,255]);expect(capability.glError).toBe(0);}
 await page.goto('/');
 const map=page.getByTestId('province-map');
 await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
 if(capability.webgl2||capability.adapterAvailable){
  await expect(map).toHaveAttribute('data-backend',/webgpu|webgl2/);await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);
  await page.mouse.move(0,0);
  const box=(await map.boundingBox())!,camera=JSON.parse((await map.getAttribute('data-camera'))!);
  const shot=png(await map.screenshot({path:`${evidence}/${info.project.name}-native.png`}));
  const pixel=shot.pixel((1-camera.x)/camera.width*box.width+box.width/2,(1-camera.y)/camera.height*box.height+box.height/2);
  for(let i=0;i<3;i++)expect(Math.abs(pixel[i]-[40,100,180][i])).toBeLessThanOrEqual(1);
 }else{
  await expect(page.getByRole('alert')).toContainText('could not initialize WebGPU or WebGL2');await expect(map.locator('canvas')).toHaveCount(0);
 }
 await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
 const tick=await page.getByTestId('tick').textContent();
 await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('1');
 await page.getByRole('combobox').selectOption('ko');await expect(page.getByTestId('country-panel')).toContainText('북부 시험국');
 if(!capability.webgl2&&!capability.adapterAvailable)await expect(page.getByRole('alert')).toContainText('초기화할 수 없습니다');
 await expect(page.getByTestId('tick')).toHaveText(tick!);expect(errors).toEqual([]);
 await page.screenshot({path:`${evidence}/${info.project.name}-native-authority-ko.png`,fullPage:true});
 writeFileSync(`${evidence}/${info.project.name}-native.json`,JSON.stringify({project:info.project.name,browser:browser.version(),capability,backend:await map.getAttribute('data-backend'),frames:await map.getAttribute('data-frames'),errors,tick,canvas:await map.locator('canvas').count()},null,2));
});
