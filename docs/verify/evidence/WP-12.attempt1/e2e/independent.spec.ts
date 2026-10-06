import { expect, test } from 'E:/openhoi/.orchestrator/wt/WP-12-verify/client/node_modules/@playwright/test/index.mjs';
import { encode, decode } from 'E:/openhoi/.orchestrator/wt/WP-12-verify/client/node_modules/@msgpack/msgpack/dist.esm/index.mjs';
import { writeFileSync } from 'node:fs';
const out = 'E:/openhoi/.orchestrator/wt/WP-12-verify/target/wp12-verify';
test('independent loading, unknown notice and disconnect stay localized', async ({page}) => {
 let release: (()=>void)|undefined;
 await page.routeWebSocket('**/ws', route => {
  release=()=>{route.send(Buffer.from(encode({type:'Notice',key:'verification-unregistered-key'})));};
 });
 await page.goto('/');
 await expect(page.getByTestId('connection')).toHaveText('Connecting');
 await expect(page.getByTestId('date')).toHaveText('Waiting for server');
 await expect(page.getByTestId('pause')).toBeDisabled();
 await page.getByRole('combobox').selectOption('ko');
 await expect(page.getByTestId('connection')).toHaveText('연결 중');
 await expect(page.getByTestId('date')).toHaveText('서버 응답 대기');
 release!();
 await expect(page.getByRole('alert')).toHaveText('서버 메시지를 표시할 수 없습니다');
 await page.screenshot({path:out+'/loading-ko.png'});
 await page.getByRole('combobox').selectOption('en');
 await expect(page.getByRole('alert')).toHaveText('The server sent an unavailable message');
});
test('independent malformed MessagePack Snapshot must preserve shell and close connection', async ({page}) => {
 const errors: string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.routeWebSocket('**/ws', route => {
  const server=route.connectToServer();
  server.onMessage(data=>{
   const m=decode(data as Buffer) as any;
   if(m.type==='Snapshot')route.send(Buffer.from(encode({...m,state:{...m.state,date:{invalid:true}}})));
   else if(m.type!=='Delta')route.send(data);
  });
 });
 await page.goto('/');
 await page.waitForTimeout(1000);
 const observed={errors,shell:await page.locator('main[data-openhoi-shell]').count(),text:await page.locator('body').innerText()};
 writeFileSync(out+'/malformed-snapshot.json',JSON.stringify(observed,null,2));
 await page.screenshot({path:out+'/malformed-snapshot.png'});
 expect(errors,'Invalid server structure must not crash React').toEqual([]);
 await expect(page.getByTestId('connection')).toHaveText('Disconnected');
 await expect(page.getByTestId('pause')).toBeDisabled();
});
