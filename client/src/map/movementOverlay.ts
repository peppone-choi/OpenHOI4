import type {MovementView,WorldView} from '../proto/protocol';
import {projectMarker,type MapPresentation} from './divisionMarkers';
type Point={x:number;y:number};
type Projected=Point&ReturnType<typeof projectMarker>;
export type MovementArrow={division:string;nation:number;color:[number,number,number];points:Projected[]};
export type MovingMarker=Projected&{division:string;nation:number;province:number;color:[number,number,number];ratio:number;remaining:string};
export type MovementGeometry={arrows:MovementArrow[];moving:MovingMarker[];excluded:Set<string>};
/** Display only. Positions and leg times are server-owned; there is no clock. */
export function buildMovementOverlay(view:MovementView|null,world:WorldView,anchors:Map<number,Point>,camera:MapPresentation,controlledNation:number|null):MovementGeometry{
 const arrows:MovementArrow[]=[],moving:MovingMarker[]=[],excluded=new Set<string>();
 const provinces=new Set(world.province_ids),nations=new Map(world.nations.map(n=>[n.id,n]));
 const point=(id:number)=>{const p=anchors.get(id);if(!provinces.has(id)||!p)throw Error('map-data-error');return p;};
 const project=(p:Point):Projected=>({...p,...projectMarker(p,camera)});
 for(const unit of view?.units??[]){
  const nation=nations.get(unit.nation);if(!nation)throw Error('map-data-error');
  const from=point(unit.province),route=unit.route.map(point);
  if(!route.length)continue;
  if(unit.nation===controlledNation)arrows.push({division:unit.division,nation:unit.nation,color:nation.color,points:[from,...route].map(project)});
  const hours=BigInt(unit.current_leg_hours.bits);if(hours<=0n)continue;
  const elapsed=BigInt(unit.elapsed.bits),bounded=elapsed<0n?0n:elapsed>hours?hours:elapsed;
  // Keep division in BigInt before a bounded display conversion (not elapsed time).
  const ratio=Number(bounded*1000000000n/hours)/1000000000;
  const p={x:from.x+(route[0].x-from.x)*ratio,y:from.y+(route[0].y-from.y)*ratio};
  moving.push({...project(p),division:unit.division,nation:unit.nation,province:unit.province,color:nation.color,ratio,remaining:unit.remaining_hours.value});excluded.add(unit.division);
 }return {arrows,moving,excluded};
}
