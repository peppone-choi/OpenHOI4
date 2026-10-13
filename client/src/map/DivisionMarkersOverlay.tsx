import {useEffect,useRef,type RefObject} from 'react';
import {useLocalization} from '../i18n';
import {projectMarker,type DivisionMarker,type MapPresentation} from './divisionMarkers';
export function DivisionMarkers({markers,camera,onSelect,host}:{markers:DivisionMarker[];camera:MapPresentation;onSelect:(id:number)=>void;host:RefObject<HTMLDivElement|null>}){
 const {t}=useLocalization(),overlay=useRef<HTMLDivElement>(null);
 // The overlay is a sibling of the renderer-owned host. Keep native wheel zoom
 // available over its buttons without changing the renderer's input handlers.
 useEffect(()=>{
  const element=overlay.current;if(!element)return;
  const wheel=(event:WheelEvent)=>{event.preventDefault();host.current?.dispatchEvent(new WheelEvent('wheel',{clientX:event.clientX,clientY:event.clientY,deltaY:event.deltaY,deltaX:event.deltaX,deltaMode:event.deltaMode,cancelable:true}));};
  element.addEventListener('wheel',wheel,{passive:false});return()=>element.removeEventListener('wheel',wheel);
 },[host]);
 const groups=new Map<number,DivisionMarker[]>();for(const marker of markers){const group=groups.get(marker.province)??[];group.push(marker);groups.set(marker.province,group);}
 return <div ref={overlay} className="division-markers" data-testid="division-markers">{[...groups].map(([province,group])=>{
  const point=projectMarker(group[0],camera);
  if(point.left<0||point.left>100||point.top<0||point.top>100)return null;
  return <div key={province} className="division-marker-group" style={{left:`${point.left}%`,top:`${point.top}%`}}>{group.map(marker=><button key={marker.nation} type="button" className="division-marker" data-division-province={province} data-division-nation={marker.nation} data-map-x={marker.x} data-map-y={marker.y} style={{borderColor:`rgb(${marker.color.join(',')})`}} aria-label={t('map-division-marker',{province,nation:marker.nation,count:marker.count})} onClick={()=>onSelect(province)}>{marker.count}</button>)}</div>;
 })}</div>;
}
