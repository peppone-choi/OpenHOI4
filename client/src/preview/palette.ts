import type { WorldView } from '../proto/protocol';
import { LOOKUP_SIZE } from '../map/model';

/** Preview rows are validated in dense ID order. No ownership/game rules. */
export const previewPalette={
 palettes(world:WorldView,mode:string){
  const colors=new Uint8Array(LOOKUP_SIZE*LOOKUP_SIZE*4),nations=new Uint8Array(colors.length),states=new Uint8Array(colors.length);
  this.updateColors(colors,world,mode);
  world.provinces.forEach((_,i)=>{nations[i*4+3]=states[i*4+3]=255;});return {colors,nations,states};
 },
 updateColors(colors:Uint8Array,world:WorldView,mode:string){
  if(mode!=='map-mode-terrain'||world.nations.length||world.states.length)throw new Error('preview-data-error');
  const ranges:{start:number;count:number}[]=[];
  world.provinces.forEach((p,i)=>{
   if(p.id!==world.province_ids[i]||p.owner!==null||p.state!==null||p.controller!==null)throw new Error('preview-data-error');
   const c=[...p.terrain_color,255];if(c.some((v,j)=>colors[i*4+j]!==v)){colors.set(c,i*4);ranges.push({start:i*4,count:4});}
  });return ranges;
 }
};
