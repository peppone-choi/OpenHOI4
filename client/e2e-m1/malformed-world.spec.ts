import { expect,test } from '@playwright/test';
import { encode,decode } from '@msgpack/msgpack';
import { mkdirSync,writeFileSync } from 'node:fs';
const evidence=process.env.OH_MALFORMED_EVIDENCE??'../target/wp09/malformed';mkdirSync(evidence,{recursive:true});
test('M1 malformed decoded ledger preserves last valid panel and closes connection',async({page},info)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));let worlds=0;
 await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{
  const message=decode(data as Buffer) as {type:string;world:{states:{infrastructure:{final_value:unknown}}[]} };
  if(message.type==='WorldResult'&&message.world&&++worlds>=3){message.world.states[0].infrastructure.final_value={invalid:true};route.send(Buffer.from(encode(message)));}else route.send(data);
 });});
 await page.goto('/');await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
 await expect(page.getByTestId('connection')).toHaveText('Disconnected');await expect(page.getByTestId('pause')).toBeDisabled();
 await expect(page.getByRole('alert')).toHaveText('Invalid server message; connection closed');expect(errors).toEqual([]);
 await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('1');
 await page.screenshot({path:`${evidence}/${info.project.name}-world-malformed.png`,fullPage:true});
 await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toHaveText('잘못된 서버 메시지로 연결을 종료했습니다');
 writeFileSync(`${evidence}/${info.project.name}-world-malformed.json`,JSON.stringify({errors,worlds,closed:true},null,2));
});
