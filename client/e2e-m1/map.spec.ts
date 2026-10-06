import { expect,test } from '@playwright/test';
import { mkdirSync,writeFileSync } from 'node:fs';
import { png } from './png';
const evidence=process.env.OH_MAP_EVIDENCE??'../target/wp08/e2e';mkdirSync(evidence,{recursive:true});
test('REQ-MAP-04/05 PLAT-03 actual map pixels, modes, pick and viewport',async({page,browser},info)=>{
 const requests:string[]=[];page.on('request',r=>requests.push(r.url()));const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/?forceWebGL=1');
 const map=page.getByTestId('province-map');
 await expect(map).toHaveAttribute('data-backend','webgl2');
 await expect(map).toHaveAttribute('data-frames',/^[1-9]\d*$/);
 await expect(page.getByRole('button',{name:'Terrain',exact:true})).toBeInViewport();
 await expect(page.getByTestId('pause')).toBeInViewport();
 const box=await map.boundingBox();expect(box).not.toBeNull();
 const camera=JSON.parse((await map.getAttribute('data-camera'))!);
 const at=(x:number,y:number)=>({x:(x-camera.x)/camera.width*box!.width+box!.width/2,y:(y-camera.y)/camera.height*box!.height+box!.height/2});
 for(const [name,color] of [['Ownership',[40,100,180]],['Terrain',[130,165,95]],['States',[140,95,170]]] as const){
  await page.getByRole('button',{name,exact:true}).click();await page.mouse.move(0,0);await page.waitForTimeout(80);
  const shot=await map.screenshot({path:`${evidence}/${info.project.name}-${name}.png`});const p=at(1,1);const actual=png(shot).pixel(p.x,p.y);for(let i=0;i<3;i++)expect(Math.abs(actual[i]-color[i]),`${name} pixel ${actual}`).toBeLessThanOrEqual(1);
 }
 await map.click({position:at(1,1)});await expect(page.getByTestId('selected-province')).toHaveText('10');
 await page.screenshot({path:`${evidence}/${info.project.name}-first-map.png`});
 expect(errors).toEqual([]);expect(requests.every(u=>new URL(u).host===new URL(page.url()).host)).toBe(true);
 writeFileSync(`${evidence}/${info.project.name}-map.json`,JSON.stringify({browser:browser.version(),requests,errors,diagnostics:await map.evaluate(el=>({... (el as HTMLElement).dataset})),viewport:{width:1280,height:720},scenario:'m1',seed:'1'},null,2));
});
