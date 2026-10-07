import type { MapStyle,WorldView } from '../proto/protocol';
import { decodeIndex,validateWorldDisplay } from '../map/model';

export type PreviewProvince={id:number;kind:'land'|'sea'|'lake';pixels:number;bounds:[number,number,number,number];longitude:number;latitude:number;geography_color:[number,number,number];province_color:[number,number,number];elevation_m:number|null};
export type PreviewMetadata={schema:'world-preview-v1';map_id:string;width:number;height:number;province_ids:number[];counts:Record<PreviewProvince['kind'],number>;style:MapStyle;regions:Record<string,{longitude:number;latitude:number;zoom:number}>;hashes:Record<string,string>;sources:{id:string;version:string;license:string;sha256:string}[];elevation:boolean;limitations:string[]};
export type PreviewData={meta:PreviewMetadata;provinces:PreviewProvince[];bytes:Uint8Array;index:Uint16Array};
export type PreviewMode='geography'|'provinces'|'terrain';
export async function sha256(bytes:Uint8Array){
 const digest=await crypto.subtle.digest('SHA-256',new Uint8Array(bytes).buffer);
 return Array.from(new Uint8Array(digest),v=>v.toString(16).padStart(2,'0')).join('');
}
const rgb=(value:unknown)=>Array.isArray(value)&&value.length===3&&value.every(n=>Number.isInteger(n)&&n>=0&&n<=255);
export function validatePreview(meta:PreviewMetadata,provinces:PreviewProvince[]){
 const fail=()=>{throw new Error('preview-data-error');};
 if(meta?.schema!=='world-preview-v1'||!Number.isInteger(meta.width)||!Number.isInteger(meta.height)||meta.width<1||meta.height<1||meta.width*meta.height>64*1024*1024)fail();
 if(!Array.isArray(meta.province_ids)||meta.province_ids.length<1||meta.province_ids.length>65536||meta.province_ids.some((id,i)=>id!==i+1)||provinces.length!==meta.province_ids.length)fail();
 for(const file of ['index.bin','provinces.json','adjacency.json'])if(!/^[a-f0-9]{64}$/.test(meta.hashes?.[file]??''))fail();
 for(const [i,p] of provinces.entries())if(p.id!==i+1||!['land','sea','lake'].includes(p.kind)||!rgb(p.geography_color)||!rgb(p.province_color)||!Number.isInteger(p.pixels)||p.pixels<1||!Number.isFinite(p.longitude)||!Number.isFinite(p.latitude)||Math.abs(p.longitude)>180||Math.abs(p.latitude)>90||(!meta.elevation&&p.elevation_m!==null))fail();
 if(!meta.sources?.length||meta.sources.some(s=>!s.version||!s.license||!/^[a-f0-9]{64}$/.test(s.sha256)))fail();
 const colors=['background','nation_border','state_border','province_border','selected','hovered'] as const;
 if(!meta.style||colors.some(key=>!rgb(meta.style[key])))fail();
 for(const key of ['nation_width_milli','state_width_milli','province_width_milli','highlight_milli','fit_milli','zoom_min_milli','zoom_max_milli','wheel_milli','drag_threshold'] as const)if(!Number.isFinite(meta.style[key])||meta.style[key]<0)fail();
 if(meta.style.fit_milli<=0||meta.style.zoom_min_milli<=0||meta.style.zoom_max_milli<meta.style.zoom_min_milli||!meta.regions?.world)fail();
 for(const region of Object.values(meta.regions))if(!Number.isFinite(region.longitude)||!Number.isFinite(region.latitude)||!Number.isFinite(region.zoom)||Math.abs(region.longitude)>180||Math.abs(region.latitude)>90||region.zoom<=0)fail();
 const count={land:0,sea:0,lake:0};provinces.forEach(p=>count[p.kind]++);
 if(Object.entries(count).some(([kind,n])=>meta.counts?.[kind as keyof typeof count]!==n)||provinces.reduce((n,p)=>n+p.pixels,0)!==meta.width*meta.height)fail();
}
/** Display-only adaptation: ownership, nations, states and game rules absent. */
export function previewWorld(meta:PreviewMetadata,provinces:PreviewProvince[],mode:PreviewMode):WorldView{
 const world:WorldView={tick:'0',map_id:meta.map_id,width:meta.width,height:meta.height,province_ids:meta.province_ids,neutral_color:meta.style.background,mode_keys:['map-mode-terrain'],nations:[],states:[],provinces:provinces.map(p=>({id:p.id,state:null,owner:null,controller:null,owner_color:null,controller_color:null,state_color:null,terrain_key:p.kind,terrain_color:mode==='provinces'?p.province_color:p.geography_color}))};
 validateWorldDisplay(world);return world;
}
export async function loadPreview(signal:AbortSignal):Promise<PreviewData>{
 const read=async(name:string)=>{
  const response=await fetch(`/preview/world/${name}`,{signal,mode:'same-origin',redirect:'error'});
  if(!response.ok)throw new Error('preview-data-error');return new Uint8Array(await response.arrayBuffer());
 };
 const meta=JSON.parse(new TextDecoder().decode(await read('metadata.json'))) as PreviewMetadata;
 const [bytes,provinceBytes]=await Promise.all([read('index.bin'),read('provinces.json')]);
 if(await sha256(bytes)!==meta.hashes?.['index.bin']||await sha256(provinceBytes)!==meta.hashes?.['provinces.json'])throw new Error('preview-data-error');
 const provinces=JSON.parse(new TextDecoder().decode(provinceBytes)) as PreviewProvince[];validatePreview(meta,provinces);
 const index=decodeIndex(new Uint8Array(bytes).buffer,meta.width,meta.height,meta.province_ids);
 return {meta,provinces,bytes,index};
}
