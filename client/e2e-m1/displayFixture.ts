import type { MapMetadata,WorldView } from '../src/proto/protocol';
/** Independent display oracle. No original-game geography or gameplay values. */
export function displayFixture(base:WorldView,meta:MapMetadata){
 const ids=[0,10,200,32768,65534,65535],width=12,height=6;
 const bytes=Buffer.alloc(width*height*2);
 for(let y=0;y<height;y++)for(let x=0;x<width;x++)bytes.writeUInt16LE(y>=4&&x<8?4:Math.floor(x/2),(y*width+x)*2);
 const owner=[70,110,180] as [number,number,number],other=[180,90,50] as [number,number,number],neutral=[20,40,60] as [number,number,number];
 const states=[0,65535,32768];
 const world:WorldView={...base,width,height,map_id:'display_fixture',province_ids:ids,neutral_color:neutral,
  nations:[{...base.nations[0],id:0,color:owner,capital:0},{...base.nations[1],id:65535,color:other,capital:32768}],
  states:states.map((id,i)=>({...base.states[i===2?1:0],id,owner:i===2?65535:0,provinces:i===0?[0,10]:i===1?[200]:[32768]})),
  provinces:ids.map((id,i)=>({...base.provinces[0],id,state:i<2?0:i===2?65535:i===3?32768:null,owner:i<3?0:i===3?65535:null,controller:i<3?0:i===3?65535:null,owner_color:i<3?owner:i===3?other:null,controller_color:i<3?owner:i===3?other:null,terrain_key:i<4?'plains':i===4?'ocean':'inland_water',terrain_color:[100+i*10,140,80],state_color:i<2?[90,120,150]:i===2?[140,95,170]:i===3?[180,155,70]:null})),
 };
 let hash=0xcbf29ce484222325n;for(const byte of bytes)hash=BigInt.asUintN(64,(hash^BigInt(byte))*0x100000001b3n);
 return {world,bytes,meta:{...meta,map_id:world.map_id,width,height,province_ids:ids,index_hash:hash.toString(16).padStart(16,'0'),byte_length:String(bytes.length)}};
}
/** Crosses LUT row boundary and requires index RG high byte=1. */
export function wideDisplayFixture(base:WorldView,meta:MapMetadata){
 const ids=Array.from({length:257},(_,i)=>i===256?65535:i),width=257,height=6,bytes=Buffer.alloc(width*height*2);
 for(let y=0;y<height;y++)for(let x=0;x<width;x++)bytes.writeUInt16LE(x,(y*width+x)*2);
 const world:WorldView={...base,map_id:'display_fixture',width,height,province_ids:ids,nations:[{...base.nations[0],id:0,capital:0}],states:[{...base.states[0],id:0,owner:0,provinces:ids}],provinces:ids.map((id,i)=>({...base.provinces[0],id,state:0,owner:0,controller:0,owner_color:i===256?[17,123,231]:[70,110,180],controller_color:[70,110,180],state_color:[90,120,150]}))};
 let hash=0xcbf29ce484222325n;for(const byte of bytes)hash=BigInt.asUintN(64,(hash^BigInt(byte))*0x100000001b3n);
 return {world,bytes,meta:{...meta,map_id:world.map_id,width,height,province_ids:ids,index_hash:hash.toString(16).padStart(16,'0'),byte_length:String(bytes.length)}};
}
