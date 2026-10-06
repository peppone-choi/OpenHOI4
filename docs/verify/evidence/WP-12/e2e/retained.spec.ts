import {expect,test} from 'E:/openhoi/.orchestrator/wt/WP-12-verify2/client/node_modules/@playwright/test/index.mjs';
import {encode,decode} from 'E:/openhoi/.orchestrator/wt/WP-12-verify2/client/node_modules/@msgpack/msgpack/dist.esm/index.mjs';
import {writeFileSync} from 'node:fs';
const out='E:/openhoi/.orchestrator/wt/WP-12-verify2/target/wp12-verify2';
test('independent invalid Delta retains last normal state, closes once and blocks later traffic',async({page},info)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{
  const Original=window.WebSocket;const audit={closeCalls:0,sendCalls:0,events:0};(window as any).__wireAudit=audit;
  class ObservedSocket extends Original {
   constructor(url:string|URL,protocols?:string|string[]){super(url,protocols);this.addEventListener('close',()=>audit.events++);}
   close(code?:number,reason?:string){audit.closeCalls++;super.close(code,reason);}
   send(data:string|ArrayBufferLike|Blob|ArrayBufferView){audit.sendCalls++;super.send(data);}
  }
  window.WebSocket=ObservedSocket;
 });
 let inject:()=>void;let normal:any;let upstream=0;
 await page.routeWebSocket('**/ws',route=>{
  const server=route.connectToServer();
  route.onMessage(data=>{upstream++;server.send(data);});
  server.onMessage(data=>{const m=decode(data as Buffer) as any;if(m.type==='Snapshot'){normal={...m.state};route.send(Buffer.from(encode({...m,state:{...m.state,paused:true}})));}else if(m.type!=='Delta')route.send(data);});
  inject=()=>route.send(Buffer.from(encode({type:'Delta',sequence:'18446744073709551615',state:{...normal,tick:{invalid:true}}})));
 });
 await page.goto('/');await expect(page.getByTestId('connection')).toHaveText('Connected');
 await expect(page.getByTestId('date')).toContainText('2000-');
 const previous=await page.locator('[data-testid=date],[data-testid=hour],[data-testid=tick],[data-testid=speed]').allTextContents();
 const beforeUpstream=upstream;
 inject!();await expect(page.getByTestId('connection')).toHaveText('Disconnected');
 await expect(page.getByRole('alert')).toHaveText('Invalid server message; connection closed');
 for(const id of ['pause','speed-1','speed-2','speed-3','speed-4','speed-5'])await expect(page.getByTestId(id)).toBeDisabled();
 await page.waitForTimeout(300);
 const retained=await page.locator('[data-testid=date],[data-testid=hour],[data-testid=tick],[data-testid=speed]').allTextContents();
 expect(retained).toEqual(previous);expect(errors).toEqual([]);
 const audit=await page.evaluate(()=>(window as any).__wireAudit);
 expect(audit.closeCalls).toBe(1);expect(audit.events).toBe(1);expect(upstream).toBe(beforeUpstream);
 await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toHaveText('잘못된 서버 메시지로 연결을 종료했습니다');
 await page.screenshot({path:out+'/'+info.project.name+'-retained-state-ko.png'});
 writeFileSync(out+'/'+info.project.name+'-retained-state.json',JSON.stringify({previous,retained,errors,audit,beforeUpstream,upstream},null,2));
});
