import {expect,test} from 'vitest';
import type {MovementView,WorldView} from '../proto/protocol';
import {buildMovementOverlay} from './movementOverlay';
import {pick} from './model';
import {provinceAnchors} from './divisionMarkers';
const world={width:3,height:1,province_ids:[1,14,40],provinces:[1,14,40].map(id=>({id})),nations:[{id:1,color:[10,20,30]},{id:2,color:[30,20,10]}]} as WorldView;
const index=new Uint16Array([0,1,2]),anchors=provinceAnchors(world,index),camera={x:1.5,y:.5,width:3,height:1};
const fixed=(bits:string,value=bits)=>({bits,value,fractional_bits:32});
const unit=(division='0',nation=1)=>({division,nation,province:1,route:[14,40],elapsed:fixed('1'),current_leg_hours:fixed('4'),remaining_hours:fixed('99','server-hours')});
const view=(units=[unit()]):MovementView=>({state_hash:'s',tick:'0',units});
test('server route order, destination real pixel, own arrows only; all nations move',()=>{
 const result=buildMovementOverlay(view([unit(),unit('1',2)]),world,anchors,camera,1);
 expect(result.arrows).toHaveLength(1);expect(result.arrows[0].points.map((p:{x:number})=>p.x)).toEqual([.5,1.5,2.5]);
 const end=result.arrows[0].points.at(-1)!;expect(pick(index,world.province_ids,3,1,end.x,end.y)).toBe(40);
 expect(result.moving).toHaveLength(2);expect([...result.excluded]).toEqual(['0','1']);
 expect(result.moving[0]).toMatchObject({x:.75,y:.5,remaining:'server-hours',ratio:.25,color:[10,20,30]});
 expect(buildMovementOverlay(view(),world,anchors,camera,null).arrows).toEqual([]);
});
test('fixed BigInt ratio clamps, zero leg stays static, no route means no arrow',()=>{
 const u=unit();u.elapsed=fixed('9223372036854775807');u.current_leg_hours=fixed('9223372036854775807');
 expect(buildMovementOverlay(view([u]),world,anchors,camera,1).moving[0].ratio).toBe(1);
 u.elapsed=fixed('-1');expect(buildMovementOverlay(view([u]),world,anchors,camera,1).moving[0].ratio).toBe(0);
 u.current_leg_hours=fixed('0');expect(buildMovementOverlay(view([u]),world,anchors,camera,1).excluded.size).toBe(0);
 u.route=[];expect(buildMovementOverlay(view([u]),world,anchors,camera,1).arrows).toEqual([]);
});
test('camera changes projection only; server refresh removes completed legs and arrival',()=>{
 const a=buildMovementOverlay(view(),world,anchors,camera,1),b=buildMovementOverlay(view(),world,anchors,{...camera,x:.5,width:2},1);
 expect(a.arrows[0].points[0].x).toBe(b.arrows[0].points[0].x);expect(a.arrows[0].points[0].left).not.toBe(b.arrows[0].points[0].left);
 const u=unit();u.province=14;u.route=[40];expect(buildMovementOverlay(view([u]),world,anchors,camera,1).arrows[0].points).toHaveLength(2);
 u.province=40;u.route=[];const arrived=buildMovementOverlay(view([u]),world,anchors,camera,1);expect(arrived.arrows).toEqual([]);expect(arrived.moving).toEqual([]);
});
test('any unknown province, route, nation or missing anchor fails whole overlay',()=>{
 for(const patch of [{province:99},{route:[99]},{nation:99}])expect(()=>buildMovementOverlay(view([{...unit(),...patch}]),world,anchors,camera,1)).toThrow('map-data-error');
 expect(()=>buildMovementOverlay(view(),world,new Map(),camera,1)).toThrow('map-data-error');
});
