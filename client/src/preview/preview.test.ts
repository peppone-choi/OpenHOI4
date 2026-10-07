import { describe,it,expect } from 'vitest';
import { validatePreview,previewWorld,sha256,type PreviewMetadata } from './model';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { previewKeys,previewTranslator } from './locales';
import { translatorFor } from '../i18n';

describe('world preview baked input',()=>{
 it('every preview display key resolves through real ko/en Fluent catalogs',()=>{
  for(const language of ['ko','en'] as const){
   const t=previewTranslator(language),unknown=translatorFor(language)('unknown-message');
   for(const key of previewKeys){expect(t(key)).not.toEqual(unknown);expect(t(key)).not.toEqual(key);}
  }
 });
 const root=resolve('public/preview/world');
 it('rejects unknown schema/IDs/ref/hash declarations',()=>{
  expect(()=>validatePreview({schema:'other'} as unknown as PreviewMetadata,[])).toThrow();
  const meta=JSON.parse(readFileSync(resolve(root,'metadata.json'),'utf8'));
  const provinces=JSON.parse(readFileSync(resolve(root,'provinces.json'),'utf8'));
  expect(()=>validatePreview({...meta,province_ids:[1,1]},provinces)).toThrow();
  expect(()=>validatePreview({...meta,hashes:{'index.bin':'bad'}},provinces)).toThrow();
  expect(()=>validatePreview(meta,[{...provinces[0],kind:'fake'},...provinces.slice(1)])).toThrow();
 });
 it('rejects missing region presets before a user can select them',()=>{
  const meta=JSON.parse(readFileSync(resolve(root,'metadata.json'),'utf8'));
  const provinces=JSON.parse(readFileSync(resolve(root,'provinces.json'),'utf8'));
  const regions={...meta.regions};delete regions.korea;
  expect(()=>validatePreview({...meta,regions},provinces)).toThrow();
 });
 it('rejects raster upper bounds and one-based ID overflow',()=>{
  const meta=JSON.parse(readFileSync(resolve(root,'metadata.json'),'utf8'));
  const provinces=JSON.parse(readFileSync(resolve(root,'provinces.json'),'utf8'));
  expect(()=>validatePreview({...meta,width:8193},provinces)).toThrow();
  expect(()=>validatePreview({...meta,province_ids:Array.from({length:65536},(_,i)=>i+1)},provinces)).toThrow();
 });
 it('hashes a view using its byte offset without copying the entire backing store',async()=>{
  expect(await sha256(new Uint8Array([0,97,98,99,0]).subarray(1,4))).toBe('ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
 });
 it('baked SHA is exact and display has no authority or invented ownership',async()=>{
  const meta=JSON.parse(readFileSync(resolve(root,'metadata.json'),'utf8'));
  const bytes=readFileSync(resolve(root,'index.bin'));
  expect(await sha256(new Uint8Array(bytes))).toBe(meta.hashes['index.bin']);
  const provinces=JSON.parse(readFileSync(resolve(root,'provinces.json'),'utf8'));
  validatePreview(meta,provinces);
  const world=previewWorld(meta,provinces,'geography');
  expect(world.nations).toEqual([]);expect(world.states).toEqual([]);
  expect(world.provinces.every(p=>p.owner===null&&p.controller===null&&p.state===null)).toBe(true);
 });
});
