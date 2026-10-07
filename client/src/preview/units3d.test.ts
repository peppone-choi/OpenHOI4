import { describe,it,expect } from 'vitest';
import { inspectGlb } from './units3d';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
function fixture(json:object){
 const text=new TextEncoder().encode(JSON.stringify(json));const padded=(text.length+3)&~3;const bytes=new Uint8Array(20+padded);bytes.fill(32,20);bytes.set(text,20);
 const view=new DataView(bytes.buffer);view.setUint32(0,0x46546c67,true);view.setUint32(4,2,true);view.setUint32(8,bytes.length,true);view.setUint32(12,padded,true);view.setUint32(16,0x4e4f534a,true);return bytes;
}
describe('preview actual mesh contract',()=>{
 it('actual registered army/air/navy are embedded three-dimensional meshes',()=>{
  const expected={army:{vertices:480,triangles:160,meshes:6},air:{vertices:432,triangles:144,meshes:8},navy:{vertices:456,triangles:152,meshes:8}};
  for(const id of ['army','air','navy'] as const){
   const bytes=readFileSync(resolve('public/preview/units3d',`${id}.glb`));
   expect(inspectGlb(bytes)).toEqual(expected[id]);
  }
 });
 it('rejects sprite/empty/remote-buffer/image GLB inputs',()=>{
  expect(()=>inspectGlb(new Uint8Array([1,2]))).toThrow();
  expect(()=>inspectGlb(fixture({asset:{version:'2.0'}}))).toThrow();
  const mesh={asset:{version:'2.0'},meshes:[{primitives:[{attributes:{POSITION:0},mode:4}]}],accessors:[{type:'VEC3',count:3}],buffers:[{uri:'https://example.com/remote.bin'}]};
  expect(()=>inspectGlb(fixture(mesh))).toThrow();
  expect(()=>inspectGlb(fixture({...mesh,buffers:[],images:[{uri:'sprite.png'}]}))).toThrow();
 });
});
