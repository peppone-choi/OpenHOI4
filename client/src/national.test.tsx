import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test, vi } from 'vitest';
import { NationalPanels } from './components/NationalPanels';
import { readFileSync } from 'node:fs';
import { Localization, translatorWithPack } from './i18n';
import { connect, isServerMessage } from './network';
import { encode, decode } from '@msgpack/msgpack';
import type { WorldView } from './proto/protocol';
const wire = () => JSON.parse(readFileSync('../target/wp09/national-wire-fixtures.json','utf8'));
test('AC-M1-03 nation/state panel adapts actual Rust world without calculating values', () => {
 const view: WorldView = wire()[0].expected.world;
 const catalogs={en:readFileSync('../data/packs/testland/localisation/en/map.ftl','utf8')+readFileSync('../data/packs/testland/localisation/en/national.ftl','utf8'),ko:readFileSync('../data/packs/testland/localisation/ko/map.ftl','utf8')+readFileSync('../data/packs/testland/localisation/ko/national.ftl','utf8')};
 const html=renderToStaticMarkup(<Localization.Provider value={{language:'en',t:translatorWithPack('en',catalogs)}}><NationalPanels world={view} /></Localization.Provider>);
 expect(html).toContain('Northern Test Nation'); expect(html).toContain('Northern Test State');
 expect(html).toContain('Infrastructure'); expect(html).toContain('Southern Test Nation');
 expect(html).toContain('0.75');
});
test('M1 runtime rejects each malformed nested field in Rust world variant',()=>{
 const fixture=wire()[0];const message=decode(Uint8Array.from(fixture.bytes));expect(message).toEqual(fixture.expected);expect(isServerMessage(message)).toBe(true);
 function visit(value:unknown,path:(string|number)[]=[]){
  if (value===null || typeof value!=='object')return;
  for(const [key,child] of Object.entries(value)){
   const route=[...path,Array.isArray(value)?Number(key):key];
   const copy=structuredClone(message);let parent=copy as Record<string|number,unknown>;
   for(const part of route.slice(0,-1))parent=parent[part] as Record<string|number,unknown>;
   parent[route.at(-1)!]={ invalid:true };expect(isServerMessage(copy),route.join('.')).toBe(false);
   visit(child,route);
  }
 }visit(message);
});

test('M1 malformed WorldResult closes the actual decode boundary before publication',()=>{
 class Socket {
  static OPEN=1;static latest:Socket;readyState=1;binaryType='';
  onopen:(()=>void)|null=null;onmessage:((e:{data:unknown})=>void)|null=null;onclose:(()=>void)|null=null;onerror:(()=>void)|null=null;
  closed=0;constructor(){Socket.latest=this;}send(){}close(){this.closed++;}
 }
 vi.stubGlobal('WebSocket',Socket);vi.stubGlobal('location',{protocol:'http:',host:'localhost'});
 try {
  const good=wire()[0].expected;
  for(const change of [(m:typeof good)=>m.world.states[0].infrastructure.final_value={invalid:true},(m:typeof good)=>m.world.provinces[0].owner_color=[256,0,0],(m:typeof good)=>m.world.nations[0].support[0].value=0.75]){
   const receive=vi.fn(),close=vi.fn();connect(receive,close);const bad=structuredClone(good);change(bad);
   Socket.latest.onmessage?.({data:encode(bad).slice().buffer});
   expect(receive).not.toHaveBeenCalled();expect(close).toHaveBeenCalledExactlyOnceWith('invalid-server-message');expect(Socket.latest.closed).toBe(1);
  }
 }finally{vi.unstubAllGlobals();}
});
