import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { MapMetadata,WorldView } from '../src/proto/protocol';
import { validatePreview,type PreviewMetadata,type PreviewProvince } from '../src/preview/model';

export const fixedIndexHash='3aafdad84138670ea627535952b987d16e79519fe33fe617a6c7321917358706';
export const scope={nonProduct:true,productComplete:false,actualProductPack:'NOT_RUN',gameplay:'NOT_RUN',M1:'NOT_RUN',AC10:'NOT_RUN',productQA:'NOT_RUN',divisionMarkers:'NOT_RUN(Q1)',movement:'NOT_RUN(Q1)',stateBorders:'NOT_RUN(Q1)',stateFilter:'NOT_MEASURED(Q1)'};
export function verifyPayload(name:string,bytes:Uint8Array,expected:string){
 const actual=createHash('sha256').update(bytes).digest('hex');
 if(!/^[a-f0-9]{64}$/.test(expected??'')||actual!==expected)throw new Error(`${name}: sha256 mismatch (${actual})`);
 return actual;
}
/** Independent uint32-limb FNV-1a; never imports the product hash. */
export function fixtureFNV(bytes:Uint8Array){
 let hi=0xcbf29ce4,lo=0x84222325;
 for(const byte of bytes){lo=(lo^byte)>>>0;const product=lo*0x1b3;hi=(Math.imul(hi,0x1b3)+Math.imul(lo,0x100)+Math.floor(product/0x100000000))>>>0;lo=product>>>0;}
 return hi.toString(16).padStart(8,'0')+lo.toString(16).padStart(8,'0');
}
export function loadWorldScale(){
 const directory=resolve(import.meta.dirname,'../public/preview/world');
 const metadataBytes=readFileSync(resolve(directory,'metadata.json'));
 const preview=JSON.parse(metadataBytes.toString()) as PreviewMetadata;
 const bytes=readFileSync(resolve(directory,'index.bin')),provinceBytes=readFileSync(resolve(directory,'provinces.json'));
 const hashes={metadataObserved:createHash('sha256').update(metadataBytes).digest('hex'),index:verifyPayload('index.bin',bytes,preview.hashes['index.bin']),provinces:verifyPayload('provinces.json',provinceBytes,preview.hashes['provinces.json'])};
 if(hashes.index!==fixedIndexHash)throw new Error('index.bin: committed input changed');
 const provinces=JSON.parse(provinceBytes.toString()) as PreviewProvince[];
 validatePreview(preview,provinces);
 if(preview.width!==8192||preview.height!==4096||provinces.length!==24889||preview.counts.land!==13882||preview.counts.sea!==6954||preview.counts.lake!==4053)throw new Error('metadata.json: committed dimensions/counts changed');
 return {preview,provinces,bytes,hashes,indexHash:fixtureFNV(bytes)};
}
export type WorldScaleInput=ReturnType<typeof loadWorldScale>;
export function worldScaleFixture(input:WorldScaleInput,base:WorldView,meta:MapMetadata){
 const {preview,provinces,bytes,indexHash}=input;
 const world:WorldView={tick:base.tick,neutral_color:base.neutral_color,mode_keys:base.mode_keys,map_id:'world_scale_harness',width:preview.width,height:preview.height,province_ids:preview.province_ids,nations:[],states:[],provinces:provinces.map(p=>({id:p.id,state:null,owner:null,controller:null,owner_color:null,controller_color:null,state_color:null,terrain_key:base.provinces[0].terrain_key,terrain_color:p.geography_color}))};
 return {world,bytes,meta:{...meta,map_id:world.map_id,width:world.width,height:world.height,province_ids:world.province_ids,byte_length:String(bytes.length),index_hash:indexHash}};
}
