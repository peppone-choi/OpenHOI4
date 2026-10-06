import {test as base} from '@playwright/test';
import {mkdirSync,writeFileSync} from 'node:fs';
const out=process.env.OH_RESIZE_PHASE_EVIDENCE!;
export const test=base.extend({page:async({page},use,info)=>{
 await page.addInitScript(()=>{
  const samples:unknown[]=[],ids=new WeakMap<HTMLCanvasElement,number>();let serial=0;
  Object.assign(window,{__resizeProbe:samples});
  setInterval(()=>{
   const host=document.querySelector<HTMLElement>('[data-testid="province-map"]');if(!host)return;
   const canvas=host.querySelector('canvas');if(canvas&&!ids.has(canvas))ids.set(canvas,++serial);
   const b=host.getBoundingClientRect(),c=canvas?.getBoundingClientRect();
   samples.push({ms:performance.now(),box:{x:b.x,y:b.y,width:b.width,height:b.height},canvas:canvas?{id:ids.get(canvas),width:canvas.width,height:canvas.height,cssWidth:c?.width,cssHeight:c?.height}:null,viewport:{width:innerWidth,height:innerHeight,dpr:devicePixelRatio,scrollX,scrollY,bodyHeight:document.body.scrollHeight},dataset:{...host.dataset},tick:document.querySelector('[data-testid="tick"]')?.textContent,pause:document.querySelector('[data-testid="pause"]')?.textContent,loading:Array.from(document.querySelectorAll('[role="status"]')).map(e=>e.textContent),selection:document.querySelector('[data-testid="selected-province"]')?.textContent});
  },100);
 });
 const started=Date.now();
 try{await use(page);}finally{
  mkdirSync(out,{recursive:true});
  const samples=await page.evaluate(()=> (window as unknown as {__resizeProbe:unknown[]}).__resizeProbe).catch(e=>({readError:String(e)}));
  writeFileSync(`${out}/${info.project.name}-geometry.json`,JSON.stringify({started,ended:Date.now(),timeout:info.timeout,status:info.status,errors:info.errors,samples},null,2));
 }
}});
