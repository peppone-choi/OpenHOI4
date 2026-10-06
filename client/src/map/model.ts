import type { WorldView } from '../proto/protocol';
// These sizes/bit masks are the RG8/u16 wire format, not gameplay constants.
export const LOOKUP_SIZE=256;
export function decodeIndex(buffer:ArrayBuffer,width:number,height:number,ids:number[]):Uint16Array {
 if(!Number.isSafeInteger(width*height)||width<=0||height<=0||buffer.byteLength!==width*height*2||new Set(ids).size!==ids.length||ids.length>65536)throw new Error('map-data-error');
 const view=new DataView(buffer);const result=new Uint16Array(width*height);
 for(let i=0;i<result.length;i++){result[i]=view.getUint16(i*2,true);if(result[i]>=ids.length)throw new Error('map-data-error');}
 return result;
}
export function pick(index:Uint16Array,ids:number[],width:number,height:number,x:number,y:number):number|null {
 if(!Number.isFinite(x)||!Number.isFinite(y)||x<0||y<0||x>=width||y>=height)return null;
 return ids[index[Math.floor(y)*width+Math.floor(x)]]??null;
}
function colorAt(world:WorldView,mode:string,id:number){
 const p=world.provinces.find(p=>p.id===id);if(!p)throw new Error('map-data-error');
 switch(mode){case 'map-mode-owner':return p.owner_color??world.neutral_color;case 'map-mode-control':return p.controller_color??world.neutral_color;case 'map-mode-terrain':return p.terrain_color;case 'map-mode-state':return p.state_color??world.neutral_color;default:throw new Error('map-data-error');}
}
export function updateColors(colors:Uint8Array,world:WorldView,mode:string) {
 const ranges:{start:number;count:number}[]=[];
 world.province_ids.forEach((id,i)=>{const c=[...colorAt(world,mode,id),255];if(c.some((v,j)=>colors[i*4+j]!==v)){colors.set(c,i*4);ranges.push({start:i*4,count:4});}});
 return ranges;
}
export function palettes(world:WorldView,mode:string){
 const colors=new Uint8Array(LOOKUP_SIZE*LOOKUP_SIZE*4),nations=new Uint8Array(colors.length),states=new Uint8Array(colors.length);
 updateColors(colors,world,mode);
 world.province_ids.forEach((id,i)=>{
  const p=world.provinces.find(p=>p.id===id);if(!p)throw new Error('map-data-error');
  // B channel carries validity so nation/state ID=0 and null never collide.
  for(const [target,ref] of [[nations,p.owner],[states,p.state]] as const)target.set([ref===null?0:ref&255,ref===null?0:ref>>>8,ref===null?0:255,255],i*4);
 });return {colors,nations,states};
}
export function fnv(bytes:Uint8Array){let h=0xcbf29ce484222325n;for(const byte of bytes)h=BigInt.asUintN(64,(h^BigInt(byte))*0x100000001b3n);return h.toString(16).padStart(16,'0');}
