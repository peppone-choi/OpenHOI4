import { expect, test, type Page, type WebSocketRoute } from '@playwright/test';
import { encode, decode } from '@msgpack/msgpack';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { ServerMessage } from '../src/proto/protocol';
import {upgradeGate} from './support/upgradeGate';

type Reply=Extract<ServerMessage,{type:'EconomyResult'}>;
type Held={socket:WebSocketRoute;reply:Reply};
async function intercept(page:Page) {
  const held:Held[] & {latestRequest:string|null}=Object.assign([],{latestRequest:null});
  await page.routeWebSocket('**/ws',socket=>{
    const server=socket.connectToServer();
    // Pause does not stop the host's periodic Delta publication. Deliver one
    // Delta per explicit time command so delayed frames have a stable request
    // window. The first test exercises the unfiltered actual stream separately.
    let deltaForCommand=false;
    socket.onMessage(frame=>{
      const message=decode(frame as Uint8Array) as {type:string;kind?:string;request?:string};
      if(message.type==='Query'&&message.kind==='economy')held.latestRequest=message.request!;
      if(message.type==='Command')deltaForCommand=true;
      server.send(frame);
    });
    server.onMessage(frame=>{
      const message=decode(frame as Uint8Array) as ServerMessage;
      if(message.type==='Delta'){
        if(!deltaForCommand)return;
        deltaForCommand=false;
      }
      if(message.type==='EconomyResult')held.push({socket,reply:message});
      else socket.send(frame);
    });
  });
  return held;
}
const send=(held:Held,reply:Reply=held.reply)=>held.socket.send(Buffer.from(encode(reply)));
async function settled(held:Held[] & {latestRequest:string|null}) {
  await expect.poll(()=>held.at(-1)?.reply.request===held.latestRequest).toBe(true);
}
async function pause(page:Page) {
  await expect(page.getByTestId('pause')).toHaveText('Pause');
  await page.getByTestId('pause').click();
  await expect(page.getByTestId('pause')).toHaveText('Resume');
}
async function primeHeldLifecycle(page:Page,held:Held[] & {latestRequest:string|null}) {
  await expect.poll(()=>held.length).toBeGreaterThan(0);
  await settled(held);send(held.at(-1)!);
  const panel=page.getByTestId('economy-panel');
  await expect(panel).toHaveAttribute('data-status','ready');
  // New country lifecycles supersede requests; Delta updates within the same
  // lifecycle now preserve an in-flight request instead of superseding it.
  let count=held.length;
  await panel.getByRole('combobox').selectOption('2');
  await expect.poll(()=>held.length).toBeGreaterThan(count);
  await settled(held);send(held.at(-1)!);
  await expect(panel).toHaveAttribute('data-status','ready');
  count=held.length;await panel.getByRole('combobox').selectOption('1');
  await expect.poll(()=>held.length).toBeGreaterThan(count);
  await settled(held);
  await expect(panel).toHaveAttribute('data-status','loading');
}

test('actual M2 server HTTP/WS projection, six nations and both languages',async({page,request,browser},info)=>{
  const errors:string[]=[],replies:Reply[]=[];
  page.on('pageerror',error=>errors.push(error.message));
  page.on('websocket',socket=>socket.on('framereceived',frame=>{
    if(typeof frame.payload==='string')return;
    const message=decode(frame.payload) as ServerMessage;
    if(message.type==='EconomyResult')replies.push(message);
  }));
  await page.goto('/');await pause(page);
  const panel=page.getByTestId('economy-panel');
  await expect(panel).toHaveAttribute('data-status','ready');
  const response=await request.get('/');expect(response.status()).toBe(200);
  const asset=(await response.text()).match(/src="([^"]+\.js)"/)![1];
  expect(await (await request.get(asset)).body()).toEqual(readFileSync(resolve('dist'+asset)));
  for(let nation=1;nation<=6;nation++){
    await panel.getByRole('combobox').selectOption(String(nation));
    await expect(panel).toHaveAttribute('data-status','ready');
    await expect(panel.getByTestId('economy-nation-body')).toHaveAttribute('data-economy-nation',String(nation));
    const latest=replies.at(-1)!;expect(latest.supported).toBe(true);expect(latest.economy!.nations).toHaveLength(6);
    const authoritative=latest.economy!.nations.find(row=>row.nation===nation)!;
    await expect(panel.getByText('Total industrial capacity',{exact:true}).locator('..').locator('dd')).toHaveText(authoritative.ledger.total_ic.value);
    await expect(panel.locator('[data-allocation="military"] td').nth(2)).toHaveText(authoritative.ledger.allocation[2].value);
    await expect(panel).toContainText(latest.economy!.state_hash);
  }
  await page.locator('header select').selectOption('ko');
  await expect(panel.getByRole('heading',{name:'경제',exact:true})).toBeVisible();
  await expect(panel).toContainText('합성 시험국 6');
  await expect(panel).toContainText('민간 시험 법령');
  const evidence=process.env.OH_E2E_EVIDENCE??'../target/evidence/o3/economy';
  mkdirSync(evidence,{recursive:true});
  await page.screenshot({path:resolve(evidence,`${info.project.name}-economy-ko.png`),fullPage:true});
  await page.locator('header select').selectOption('en');
  await expect(panel).toContainText('Civil Test Law');
  await page.screenshot({path:resolve(evidence,`${info.project.name}-economy-en.png`),fullPage:true});
  for(const language of ['en','ko'])for(const file of ['map.ftl','national.ftl'])expect((await request.get(`/pack/localisation/${language}/${file}`)).status()).toBe(200);
  expect(errors).toEqual([]);
  writeFileSync(resolve(evidence,`${info.project.name}-economy.json`),JSON.stringify({browser:browser.version(),nation_count:6,scenario:'m2_initial',request:replies.at(-1)!.request,state_hash:replies.at(-1)!.economy!.state_hash,errors},null,2));
});

test('reversed and unissued future replies are ignored; nation switch clears and latest unsupported stays empty',async({page})=>{
  const held=await intercept(page);await page.goto('/');await pause(page);
  await primeHeldLifecycle(page,held);
  await settled(held);
  const panel=page.getByTestId('economy-panel'),latest=held.at(-1)!,old=held[0];
  send(latest,{...latest.reply,request:'economy:999999'});
  await expect(panel).toHaveAttribute('data-status','loading');
  send(latest);
  try { await expect(panel).toHaveAttribute('data-status','ready'); }
  catch(error) { console.log({sent:latest.reply.request,newest:held.latestRequest,received:held.map(row=>row.reply.request)});throw error; }
  const oldChanged=structuredClone(old.reply);for(const nation of oldChanged.economy!.nations)nation.capacity='9007199254740993';
  send(old,oldChanged);await expect(panel).not.toContainText('9007199254740993');
  const count=held.length;
  await panel.getByRole('combobox').selectOption('2');
  await expect(panel.getByTestId('economy-nation-body')).toHaveCount(0);
  await expect.poll(()=>held.length).toBeGreaterThan(count);
  await settled(held);
  const selected=held.at(-1)!;
  send(latest);await expect(panel).toHaveAttribute('data-status','loading');
  send(selected,{type:'EconomyResult',request:selected.reply.request,supported:false,reason_key:'unsupported-query',economy:null});
  await expect(panel).toHaveAttribute('data-status','unsupported');
  send(latest);await expect(panel.getByTestId('economy-nation-body')).toHaveCount(0);
});

test('malformed frame retains stale verified view; reconnect clears it and only fresh known response restores it',async({page})=>{
  const held=await intercept(page);await page.goto('/');await pause(page);
  await primeHeldLifecycle(page,held);
  await settled(held);
  const panel=page.getByTestId('economy-panel'),verified=held.at(-1)!;
  send(verified);await expect(panel).toHaveAttribute('data-status','ready');
  const count=held.length;
  await page.getByTestId('speed-2').click();await expect.poll(()=>held.length).toBeGreaterThan(count);
  await settled(held);
  const malformed=held.at(-1)!,bad=structuredClone(malformed.reply);
  bad.economy!.nations[0].political_capital.value='NaN';
  send(malformed,bad);
  await expect(panel).toHaveAttribute('data-status','stale');
  await expect(panel).toContainText(verified.reply.economy!.state_hash);
  const beforeReconnect=held.length;
  await page.getByRole('button',{name:'Reconnect',exact:true}).click();
  await expect(panel.getByTestId('economy-nation-body')).toHaveCount(0);
  await pause(page);await expect.poll(()=>held.length).toBeGreaterThan(beforeReconnect);
  await settled(held);
  const fresh=held.at(-1)!;
  send(fresh,{...verified.reply});await expect(panel).toHaveAttribute('data-status','loading');
  send(fresh);await expect(panel).toHaveAttribute('data-status','ready');
  await expect(panel).toContainText(fresh.reply.economy!.state_hash);
});

test('large canonical strings and valid empty arrays display without precision loss or stale remnants',async({page})=>{
  const held=await intercept(page);await page.goto('/');await pause(page);
  await primeHeldLifecycle(page,held);
  await settled(held);
  const latest=held.at(-1)!,large=structuredClone(latest.reply);
  large.economy!.nations[0].capacity='9223372036854775807';
  large.economy!.nations[0].reserved='9007199254740993';
  large.economy!.nations[0].ledger.tick='18446744073709551615';
  send(latest,large);
  const panel=page.getByTestId('economy-panel');await expect(panel).toHaveAttribute('data-status','ready');
  for(const value of ['9223372036854775807','9007199254740993','18446744073709551615'])await expect(panel).toContainText(value);
  const count=held.length;await panel.getByRole('combobox').selectOption('2');await expect.poll(()=>held.length).toBeGreaterThan(count);
  await settled(held);
  const empty=held.at(-1)!,reply=structuredClone(empty.reply);
  reply.economy!.nations=[];reply.economy!.pending=[];reply.economy!.industrial_scores=[];
  send(empty,reply);await expect(panel).toHaveAttribute('data-status','ready');
  await expect(panel.getByTestId('economy-nation-body')).toHaveCount(0);
  await expect(panel).toContainText('No economic data');
  await expect(panel).not.toContainText('9007199254740993');
});

test('actual frozen M1 generic unsupported clears economic display without protocol changes',async({page})=>{
  const replies:Extract<ServerMessage,{type:'QueryResult'}>[]=[];
  page.on('websocket',socket=>socket.on('framereceived',frame=>{
    if(typeof frame.payload==='string')return;
    const message=decode(frame.payload) as ServerMessage;
    if(message.type==='QueryResult'&&message.request.startsWith('economy:'))replies.push(message);
  }));
  await page.goto('/');await pause(page);
  await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
  const panel=page.getByTestId('economy-panel');
  await expect(panel).toHaveAttribute('data-status','unsupported');
  await expect(panel.getByTestId('economy-nation-body')).toHaveCount(0);
  expect(replies.at(-1)).toMatchObject({supported:false,reason_key:'unsupported-query',state:null});
});

test('continuous actual Delta stream preserves a delayed economic query and coalesces one follow-up',async({page})=>{
  const held:Held[]=[],requests:string[]=[];
  let deltas=0;
  await page.routeWebSocket('**/ws',socket=>{
    const server=socket.connectToServer();
    socket.onMessage(frame=>{
      const message=decode(frame as Uint8Array) as {type:string;kind?:string;request?:string};
      if(message.type==='Query'&&message.kind==='economy')requests.push(message.request!);
      server.send(frame);
    });
    server.onMessage(frame=>{
      const message=decode(frame as Uint8Array) as ServerMessage;
      if(message.type==='Delta')deltas++;
      if(message.type==='EconomyResult')held.push({socket,reply:message});
      else socket.send(frame);
    });
  });
  await page.goto('/');await pause(page);
  await expect.poll(()=>held.length).toBeGreaterThan(0);
  await expect.poll(()=>deltas).toBeGreaterThanOrEqual(4);
  expect(requests).toHaveLength(1);
  send(held[0]);
  const panel=page.getByTestId('economy-panel');
  await expect(panel).toHaveAttribute('data-status','ready');
  await expect(panel).toContainText(held[0].reply.economy!.state_hash);
  await expect.poll(()=>held.length).toBe(2);
  const before=deltas;await expect.poll(()=>deltas).toBeGreaterThanOrEqual(before+4);
  expect(requests).toHaveLength(2);
  send(held[1]);
  await expect(panel).toHaveAttribute('data-status','ready');
  await expect(panel).toContainText(held[1].reply.economy!.state_hash);
  await expect.poll(()=>held.length).toBe(3);
});

test('immediate reconnect removes old production and blocks commands until current-session verification',async({page})=>{
  // Existing real Rust production projection supplies a UI lifecycle control;
  // the initial M2 economy pack itself has no production authority. No pack or
  // server rule is changed, and no production integration is claimed here.
  const production=JSON.parse(readFileSync(resolve('../target/wp15/production-wire-fixture.json'),'utf8')) as Extract<ServerMessage,{type:'ProductionResult'}>;
  const sockets:WebSocketRoute[]=[],commands:unknown[]=[],heldProduction:Array<{socket:WebSocketRoute;request:string}>=[];
  let holdProduction=false;
  await page.routeWebSocket('**/ws',socket=>{
    sockets.push(socket);const server=socket.connectToServer();
    socket.onMessage(frame=>{
      const message=decode(frame as Uint8Array) as {type:string};
      if(message.type==='ProductionCommand')commands.push(message);
      server.send(frame);
    });
    server.onMessage(frame=>{
      const message=decode(frame as Uint8Array) as ServerMessage;
      if(message.type==='ProductionResult'||(message.type==='QueryResult'&&message.request==='production')){
        if(holdProduction)heldProduction.push({socket,request:message.request});
        else socket.send(Buffer.from(encode({...production,request:message.request})));
      }else socket.send(frame);
    });
  });
  await page.goto('/');await pause(page);
  const panel=page.getByTestId('production-panel');
  await expect(panel).toBeVisible();
  await panel.getByRole('button',{name:'Start as selected nation',exact:true}).click();
  await pause(page);
  await expect(panel.getByRole('button',{name:'Create line',exact:true})).toBeEnabled();
  const old=sockets.at(-1)!;
  old.send(Buffer.from([193]));
  await expect(page.getByRole('button',{name:'Reconnect',exact:true})).toBeVisible();
  holdProduction=true;
  await page.getByRole('button',{name:'Reconnect',exact:true}).click();
  await pause(page);
  await expect.poll(()=>heldProduction.length).toBeGreaterThan(0);
  await expect(panel).toHaveCount(0);
  // Enter cannot dispatch a stale form on the new, connected session either.
  await page.keyboard.press('Enter');expect(commands).toHaveLength(0);
  const fresh=heldProduction.at(-1)!;
  fresh.socket.send(Buffer.from(encode({...production,request:'unissued-production'})));
  await expect(panel).toHaveCount(0);
  fresh.socket.send(Buffer.from(encode({...production,request:fresh.request})));
  await expect(panel.getByRole('button',{name:'Create line',exact:true})).toBeEnabled();
  await panel.getByRole('button',{name:'Create line',exact:true}).click();
  await expect.poll(()=>commands.length).toBe(1);
});

test('CONNECTING country selections through actual OPEN use the latest scope without unsent pending or duplicate requests',async({page,baseURL})=>{
  const gate=await upgradeGate(baseURL!);
  const requests:string[][]=[];
  page.on('websocket',socket=>{
    const sent:string[]=[];requests.push(sent);
    socket.on('framesent',frame=>{
      if(typeof frame.payload==='string')return;
      const message=decode(frame.payload) as {type:string;kind?:string;request?:string};
      if(message.type==='Query'&&message.kind==='economy')sent.push(message.request!);
    });
  });
  await page.addInitScript(()=>{
    const sockets:WebSocket[]=[];
    const Native=window.WebSocket;
    window.WebSocket=class extends Native {constructor(url:string|URL,protocols?:string|string[]){super(url,protocols);sockets.push(this);}};
    (window as unknown as {probeSockets:WebSocket[]}).probeSockets=sockets;
  });
  try {
    await page.goto(gate.url);await pause(page);
    const panel=page.getByTestId('economy-panel');await expect(panel).toHaveAttribute('data-status','ready');
    await page.evaluate(()=>{(window as unknown as {probeSockets:WebSocket[]}).probeSockets.at(-1)!.close(1000,'connecting regression');});
    await expect(page.getByRole('button',{name:'Reconnect',exact:true})).toBeVisible();
    gate.hold();await page.getByRole('button',{name:'Reconnect',exact:true}).click();
    await expect.poll(()=>gate.count()).toBe(1);
    expect(await page.evaluate(()=>(window as unknown as {probeSockets:WebSocket[]}).probeSockets.at(-1)!.readyState)).toBe(0);
    await page.getByTestId('country-panel').getByRole('button').nth(1).click();
    await page.getByTestId('country-panel').getByRole('button').nth(3).click();
    await expect(panel.getByTestId('economy-nation-body')).toHaveCount(0);
    expect(requests.at(-1)).toEqual([]);
    gate.release();
    await expect.poll(()=>page.evaluate(()=>(window as unknown as {probeSockets:WebSocket[]}).probeSockets.at(-1)!.readyState)).toBe(1);
    await expect(panel).toHaveAttribute('data-status','ready');
    await expect(panel.getByTestId('economy-nation-body')).toHaveAttribute('data-economy-nation','4');
    await pause(page);
    await panel.getByRole('combobox').selectOption('2');
    await expect(panel).toHaveAttribute('data-status','ready');
    await expect(panel.getByTestId('economy-nation-body')).toHaveAttribute('data-economy-nation','2');
    const sent=requests.at(-1)!;expect(sent.length).toBeGreaterThan(0);expect(new Set(sent).size).toBe(sent.length);
  } finally {await gate.close();}
});
