import { expect, test } from 'vitest';
import { readFileSync } from 'node:fs';
import { deepStrictEqual } from 'node:assert/strict';
import { decodeIndex, pick, palettes, updateColors,validateWorldDisplay } from './model';
import type { WorldView } from '../proto/protocol';
const world = (): WorldView => JSON.parse(readFileSync('../target/wp09/national-wire-fixtures.json','utf8'))[0].expected.world;
// Frozen find-based reference preserves the pre-optimization byte/range contract.
function referenceUpdate(colors:Uint8Array,w:WorldView,mode:string){
 const ranges:{start:number;count:number}[]=[];
 w.province_ids.forEach((id,i)=>{
  const p=w.provinces.find(p=>p.id===id);if(!p)throw new Error('map-data-error');
  let rgb:[number,number,number];
  switch(mode){case 'map-mode-owner':rgb=p.owner_color??w.neutral_color;break;case 'map-mode-control':rgb=p.controller_color??w.neutral_color;break;case 'map-mode-terrain':rgb=p.terrain_color;break;case 'map-mode-state':rgb=p.state_color??w.neutral_color;break;default:throw new Error('map-data-error');}
  const c=[...rgb,255];if(c.some((v,j)=>colors[i*4+j]!==v)){colors.set(c,i*4);ranges.push({start:i*4,count:4});}
 });return ranges;
}
function referencePalettes(w:WorldView,mode:string){
 const colors=new Uint8Array(256*256*4),nations=new Uint8Array(colors.length),states=new Uint8Array(colors.length);
 referenceUpdate(colors,w,mode);
 w.province_ids.forEach((id,i)=>{
  const p=w.provinces.find(p=>p.id===id);if(!p)throw new Error('map-data-error');
  for(const [target,ref] of [[nations,p.owner],[states,p.state]] as const)target.set([ref===null?0:ref&255,ref===null?0:ref>>>8,ref===null?0:255,255],i*4);
 });return {colors,nations,states};
}
test('palette missing ID keeps map-data-error and writes only preceding dense rows',()=>{
 const w=world();w.provinces=w.provinces.filter(p=>p.id!==w.province_ids[1]);
 const colors=new Uint8Array(256*256*4),expected=colors.slice();
 expect(()=>referenceUpdate(expected,w,'map-mode-terrain')).toThrow('map-data-error');
 expect(()=>updateColors(colors,w,'map-mode-terrain')).toThrow('map-data-error');
 expect(colors[3]).toBe(255);expect(colors.subarray(4).every(v=>v===0)).toBe(true);
 deepStrictEqual(colors,expected);
 expect(()=>palettes(w,'map-mode-terrain')).toThrow('map-data-error');
});
test('palette duplicate IDs keep first row colors and nation/state identities',()=>{
 const w=world(),first=w.provinces.find(p=>p.id===w.province_ids[0])!;
 w.provinces.push({...first,owner:65535,state:65535,owner_color:[1,2,3],controller_color:[4,5,6],terrain_color:[7,8,9],state_color:[10,11,12]});
 for(const mode of w.mode_keys){
  const actual=palettes(w,mode);deepStrictEqual(actual,referencePalettes(w,mode));
  const colors=new Uint8Array(actual.colors.length),expected=colors.slice();
  expect(updateColors(colors,w,mode)).toEqual(referenceUpdate(expected,w,mode));deepStrictEqual(colors,expected);
 }
});
test('four palette modes preserve all bytes and changed ranges without stale world lookup',()=>{
 const w=world(),modes=['map-mode-owner','map-mode-control','map-mode-terrain','map-mode-state'];
 const colors=new Uint8Array(256*256*4),expected=colors.slice();
 for(const mode of modes){
  deepStrictEqual(palettes(w,mode),referencePalettes(w,mode));
  expect(updateColors(colors,w,mode)).toEqual(referenceUpdate(expected,w,mode));deepStrictEqual(colors,expected);
  expect(updateColors(colors,w,mode)).toEqual([]);
 }
 w.provinces=w.provinces.map(p=>({...p,terrain_color:[1,2,3]}));
 expect(updateColors(colors,w,'map-mode-terrain')).toEqual(referenceUpdate(expected,w,'map-mode-terrain'));deepStrictEqual(colors,expected);
 deepStrictEqual(palettes(w,'map-mode-terrain'),referencePalettes(w,'map-mode-terrain'));
});
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
