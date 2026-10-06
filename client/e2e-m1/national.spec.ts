import { expect,test } from '@playwright/test';
import { encode,decode } from '@msgpack/msgpack';
import { mkdirSync,writeFileSync } from 'node:fs';
import type { ServerMessage,ClientMessage } from '../src/proto/protocol';
const evidence=process.env.OH_E2E_EVIDENCE??'../target/wp09/e2e';mkdirSync(evidence,{recursive:true});
test('AC-M1-03 actual Rust M1 panels, separate owner/control, ledger, languages and time',async({page,request,browser},info)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('/');
 await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
 await expect(page.getByTestId('country-panel')).toContainText('Test Republic');
 await expect(page.getByTestId('state-panel')).toContainText('Northern Test State');
 await expect(page.getByTestId('province-20')).toHaveText('20Northern Test NationSouthern Test Nation');
 await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
 const tick=await page.getByTestId('tick').textContent();await page.waitForTimeout(400);await expect(page.getByTestId('tick')).toHaveText(tick!);
 const panel=page.getByTestId('state-panel');await panel.getByRole('button',{name:'Northern Test State'}).click();
 await panel.getByText('Infrastructure',{exact:true}).click();
 await expect(panel.locator('.ledger-value strong')).toHaveText('1');await expect(panel.locator('.ledger-tooltip tbody tr')).toHaveCount(1);
 await expect(panel.locator('.ledger-tooltip tbody tr td').nth(2)).toHaveText('1');await expect(panel.locator('.ledger-tooltip tbody tr td').nth(3)).toHaveText('1');
 const worldTick=await page.getByTestId('national-panels').getAttribute('data-world-tick');
 await expect(panel.locator('.ledger-tooltip')).toContainText(`Tick: ${worldTick}`);
 for(const speed of [1,2,3,4,5]){await page.getByTestId(`speed-${speed}`).click();await expect(page.getByTestId('speed')).toHaveText(String(speed));}
 await page.screenshot({path:`${evidence}/${info.project.name}-m1-en.png`,fullPage:true});
 await page.getByTestId('country-panel').getByRole('button',{name:'Southern Test Nation'}).click();await expect(page.getByTestId('country-panel')).toContainText('Test Council');
 await panel.getByRole('button',{name:'Southern Test State'}).click();await expect(panel.locator('.ledger-value strong')).toHaveText('0');
 await page.getByRole('combobox').selectOption('ko');await expect(page.getByTestId('country-panel')).toContainText('남부 시험국');await expect(panel).toContainText('남부 시험주');
 await panel.getByRole('button',{name:'북부 시험주'}).click();await expect(page.getByTestId('province-20')).toHaveText('20북부 시험국남부 시험국');
 await page.screenshot({path:`${evidence}/${info.project.name}-m1-ko.png`,fullPage:true});
 await page.getByTestId('pause').click();await expect(page.getByTestId('tick')).not.toHaveText(tick!);await page.getByTestId('pause').click();
 for(const lang of ['en','ko'])for(const file of ['map.ftl','national.ftl']){const f=await request.get(`/pack/localisation/${lang}/${file}`);expect(f.status()).toBe(200);expect(f.headers()['content-type']).toContain('text/plain');}
 expect(errors).toEqual([]);writeFileSync(`${evidence}/${info.project.name}-m1.json`,JSON.stringify({browser:browser.version(),project:info.project.name,scenario:'m1',seed:'1',errors,worldTick,viewport:{width:1280,height:720},render:'actual Rust M1 DOM panels; WP08 GPU map pending'},null,2));
});
test('REQ-NAT-01 MAP-08 actual query metadata, dense mapping, colors and refusal',async({page})=>{
 await page.goto('/');
 const messages:ClientMessage[]=[{type:'Hello',protocol_version:'m0-v1'},{type:'Query',request:'before',kind:'world'},{type:'Join',session:'local',nation:null},{type:'Command',sequence:'1',command:{type:'Pause',paused:true}},{type:'Query',request:'world',kind:'world'},{type:'Query',request:'north',kind:'nation:1'},{type:'Query',request:'state',kind:'state:1'},{type:'Query',request:'missing',kind:'state:65535'},{type:'Query',request:'bad',kind:'nation:abc'},{type:'Query',request:'scope',kind:'economy'}];
 const frames=await page.evaluate(bytes=>new Promise<number[][]>(resolve=>{const frames:number[][]=[];const ws=new WebSocket(`ws://${location.host}/ws`);ws.binaryType='arraybuffer';ws.onopen=()=>bytes.forEach(b=>ws.send(Uint8Array.from(b)));ws.onmessage=e=>frames.push(Array.from(new Uint8Array(e.data)));setTimeout(()=>{ws.close();resolve(frames);},500);}),messages.map(m=>Array.from(encode(m))));
 const values=frames.map(b=>decode(Uint8Array.from(b))) as ServerMessage[];
 const result=values.find(m=>m.type==='WorldResult'&&m.request==='world');expect(result?.type).toBe('WorldResult');if(result?.type!=='WorldResult'||!result.world)throw new Error('missing actual world');
 const world=result.world;expect(world.map_id).toBe('testland');expect(world.province_ids).toEqual([10,20,30,40,50,60]);expect(world.width).toBe(8);expect(world.height).toBe(6);
 expect(world.provinces.find(p=>p.id===20)).toMatchObject({owner:1,controller:2,owner_color:[40,100,180],controller_color:[180,70,40],state_color:[140,95,170],terrain_color:[155,135,95]});
 expect(world.provinces.find(p=>p.id===50)).toMatchObject({owner:null,controller:null});
 expect(world.states[0].infrastructure).toMatchObject({base:'1',final_value:'1',tick:world.tick,entries:[{value:'1',accumulated:'1'}]});
 for(const request of ['missing','bad'])expect(values).toContainEqual({type:'WorldResult',request,supported:false,reason_key:'unknown-entity',world:null});
 expect(values).toContainEqual({type:'WorldResult',request:'before',supported:false,reason_key:'not-joined',world:null});
 expect(values).toContainEqual({type:'QueryResult',request:'scope',supported:false,reason_key:'unsupported-query',state:null});
});
