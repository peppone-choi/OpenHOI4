import { expect, test } from 'vitest';
import { readFileSync } from 'node:fs';
import { decodeIndex, pick, palettes, updateColors,validateWorldDisplay } from './model';
import type { WorldView } from '../proto/protocol';
const world = (): WorldView => JSON.parse(readFileSync('../target/wp09/national-wire-fixtures.json','utf8'))[0].expected.world;
test('REQ-MAP-04 server RGB colors use dense mapping, neutral sea and each mode',()=>{
 const w=world(); const p=palettes(w,'map-mode-owner');
 expect(Array.from(p.colors.slice(4,8))).toEqual([40,100,180,255]);
 expect(Array.from(p.colors.slice(16,20))).toEqual([...w.neutral_color,255]);
 expect(Array.from(palettes(w,'map-mode-control').colors.slice(4,8))).toEqual([180,70,40,255]);
 expect(Array.from(palettes(w,'map-mode-terrain').colors.slice(4,8))).toEqual([155,135,95,255]);
 expect(Array.from(palettes(w,'map-mode-state').colors.slice(4,8))).toEqual([140,95,170,255]);
 const changed=updateColors(p.colors,w,'map-mode-control');expect(changed).toEqual([{start:4,count:4}]);
 expect(updateColors(p.colors,w,'map-mode-control')).toEqual([]);
});
test('MAP display schema rejects dangling/duplicate refs before lookup publication',()=>{
 expect(()=>validateWorldDisplay(world())).not.toThrow();
 const bad=[(w:WorldView)=>w.width=0,(w:WorldView)=>w.province_ids.push(w.province_ids[0]),(w:WorldView)=>w.provinces.pop(),(w:WorldView)=>w.provinces[0].owner=65535,(w:WorldView)=>w.provinces[0].state=65535,(w:WorldView)=>w.states[0].provinces=[],(w:WorldView)=>w.provinces[0].owner_color=null];
 for(const change of bad){const w=world();change(w);expect(()=>validateWorldDisplay(w)).toThrow('map-data-error');}
});
test('REQ-MAP-05 little endian RG8 pick: top-left, sparse/high/zero real IDs and outside',()=>{
 const data=decodeIndex(new Uint8Array([0,0,1,0,2,0,3,0]).buffer,2,2,[0,32768,65535,4]);
 expect(pick(data,[0,32768,65535,4],2,2,0,0)).toBe(0);
 expect(pick(data,[0,32768,65535,4],2,2,1,0)).toBe(32768);
 expect(pick(data,[0,32768,65535,4],2,2,0,1)).toBe(65535);
 for(const [x,y] of [[-1,0],[0,-1],[2,0],[0,2],[NaN,0]])expect(pick(data,[0,32768,65535,4],2,2,x,y)).toBeNull();
 expect(()=>decodeIndex(new Uint8Array([0,1]).buffer,1,1,[0])).toThrow();
 expect(()=>decodeIndex(new Uint8Array([0]).buffer,1,1,[0])).toThrow();
});
test('REQ-MAP-05 owner/state identity lookup distinguishes null from valid ID zero',()=>{
 const w=world();w.provinces[0]={...w.provinces[0],owner:0,state:65535};
 const p=palettes(w,'map-mode-owner');
 expect(Array.from(p.nations.slice(0,4))).toEqual([0,0,255,255]);
 expect(Array.from(p.states.slice(0,4))).toEqual([255,255,255,255]);
 expect(Array.from(p.nations.slice(16,20))).toEqual([0,0,0,255]);
});
