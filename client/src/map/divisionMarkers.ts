import type {MilitaryView,WorldView} from '../proto/protocol';
export type MapPresentation={x:number;y:number;width:number;height:number};
type Point={x:number;y:number};
export type DivisionMarker=Point&{province:number;nation:number;count:number;color:[number,number,number]};
/** Two linear passes, once per decoded map: nearest real pixel centre to the
 * centroid, with row-major tie breaking. Works for holes and disconnected land. */
export function provinceAnchors(world:WorldView,index:Uint16Array):Map<number,Point>{
 const {width,height,province_ids:ids}=world;
 if(width<=0||height<=0||!Number.isSafeInteger(width*height)||index.length!==width*height)throw new Error('map-data-error');
 const count=new Float64Array(ids.length),x=new Float64Array(ids.length),y=new Float64Array(ids.length);
 for(let pixel=0;pixel<index.length;pixel++){
  const dense=index[pixel];if(dense>=ids.length)throw new Error('map-data-error');
  count[dense]++;x[dense]+=pixel%width+.5;y[dense]+=Math.floor(pixel/width)+.5;
 }
 for(let dense=0;dense<ids.length;dense++)if(count[dense]){x[dense]/=count[dense];y[dense]/=count[dense];}
 const distances=new Float64Array(ids.length).fill(Infinity),points=new Map<number,Point>();
 for(let pixel=0;pixel<index.length;pixel++){
  const dense=index[pixel],px=pixel%width+.5,py=Math.floor(pixel/width)+.5,distance=(px-x[dense])**2+(py-y[dense])**2;
  if(distance<distances[dense]){distances[dense]=distance;points.set(ids[dense],{x:px,y:py});}
 }return points;
}
/** Display only: references/colours come from world, never inferred game rules. */
export function buildDivisionMarkers(view:MilitaryView|null,world:WorldView,anchors:Map<number,Point>):DivisionMarker[]{
 if(!view)return [];
 const provinces=new Set(world.provinces.map(p=>p.id)),nations=new Map(world.nations.map(n=>[n.id,n]));
 const groups=new Map<string,DivisionMarker>();
 for(const division of view.divisions){
  const point=anchors.get(division.province),nation=nations.get(division.nation);
  if(!provinces.has(division.province)||!point||!nation)throw new Error('map-data-error');
  const key=`${division.province}:${division.nation}`,existing=groups.get(key);
  if(existing)existing.count++;else groups.set(key,{province:division.province,nation:division.nation,count:1,...point,color:nation.color});
 }return [...groups.values()].sort((a,b)=>a.province-b.province||a.nation-b.nation);
}
export function projectMarker(point:Point,camera:MapPresentation){
 return {left:50+(point.x-camera.x)/camera.width*100,top:50+(point.y-camera.y)/camera.height*100};
}
