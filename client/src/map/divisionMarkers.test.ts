import {expect,test} from 'vitest';
import {buildDivisionMarkers,provinceAnchors,projectMarker} from './divisionMarkers';
import {pick} from './model';
import type {MilitaryView,WorldView} from '../proto/protocol';
const world:WorldView={tick:'0',map_id:'fixture',width:3,height:3,province_ids:[0,99],neutral_color:[0,0,0],mode_keys:[],states:[],provinces:[0,99].map(id=>({id,state:null,owner:null,controller:null,owner_color:null,controller_color:null,terrain_key:'terrain',terrain_color:[0,0,0],state_color:null})),nations:[1,2].map(id=>({id,tag:`N${id}`,name_key:'nation',color:[id,2,3],capital:0,government_key:'government',support:[]}))};
// The ring's arithmetic mean is in the OTHER province. Anchors must be pixel centres.
const index=new Uint16Array([0,0,0,0,1,0,0,0,0]);
const view=(rows:Array<[number,number]>):MilitaryView=>({state_hash:'',definitions_hash:'',next_job_id:'0',next_division_id:'0',templates:[],armies:[],jobs:[],background:[],pending:[],divisions:rows.map(([province,nation],i)=>({id:String(i),province,nation,army:'0',template:'0',manpower:'0',equipment:[],normal:{} as MilitaryView['divisions'][number]['normal']}))});
const markers=(v:MilitaryView|null,w=world,i=index)=>buildDivisionMarkers(v,w,provinceAnchors(w,i));
test('empty/null data has no markers; same province+nation aggregates and uses server colour',()=>{
 expect(markers(null)).toEqual([]);expect(markers(view([]))).toEqual([]);
 expect(markers(view([[0,1],[0,1]]))).toEqual([{province:0,nation:1,count:2,x:1.5,y:.5,color:[1,2,3]}]);
});
test('two nations remain separate, sorted by province/nation regardless of input order',()=>{
 const rows:Array<[number,number]>=[[99,1],[0,2],[0,1],[0,2]];
 const a=markers(view(rows));expect(a.map(m=>[m.province,m.nation,m.count])).toEqual([[0,1,1],[0,2,2],[99,1,1]]);
 expect(markers(view([...rows].reverse()))).toEqual(a);
});
test('non-convex ring and disconnected province anchors always pick their actual province',()=>{
 for(const data of [index,new Uint16Array([0,1,1,1,1,1,1,1,0])]){
  for(const [id,p] of provinceAnchors(world,data))expect(pick(data,world.province_ids,3,3,p.x,p.y)).toBe(id);
 }
});
test('unknown province/nation or absent province pixels is map-data-error, never partial markers',()=>{
 expect(()=>markers(view([[0,1],[7,1]]))).toThrow('map-data-error');
 expect(()=>markers(view([[0,7]]))).toThrow('map-data-error');
 expect(()=>markers(view([[99,1]]),world,new Uint16Array(9))).toThrow('map-data-error');
 expect(()=>provinceAnchors(world,new Uint16Array([2]))).toThrow('map-data-error');
});
test('camera projection follows panning and zoom; dimensions remain percentages on resize',()=>{
 const p={x:1.5,y:.5};const projected=projectMarker(p,{x:1.5,y:1.5,width:3,height:3});expect(projected.left).toBe(50);expect(projected.top).toBeCloseTo(100/6);
 expect(projectMarker(p,{x:.5,y:.5,width:2,height:2})).toEqual({left:100,top:50});
});
