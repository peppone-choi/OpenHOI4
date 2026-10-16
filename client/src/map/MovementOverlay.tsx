import {useEffect,useId,useRef,type RefObject} from 'react';
import {useLocalization} from '../i18n';
import type {MovementFeedback} from '../movementCommands';
import type {MovementGeometry,MovingMarker} from './movementOverlay';
export function MovementOverlay({geometry,selected,onActivate,onContextPick,host}:{geometry:MovementGeometry;selected:readonly string[];onActivate:(marker:MovingMarker)=>void;onContextPick:(id:number)=>boolean;host:RefObject<HTMLDivElement|null>}){
 const {t}=useLocalization(),prefix=useId().replace(/:/g,''),overlay=useRef<HTMLDivElement>(null);
 useEffect(()=>{
  const element=overlay.current;if(!element)return;
  const wheel=(e:WheelEvent)=>{e.preventDefault();host.current?.dispatchEvent(new WheelEvent('wheel',{clientX:e.clientX,clientY:e.clientY,deltaY:e.deltaY,deltaX:e.deltaX,deltaMode:e.deltaMode,cancelable:true}));};
  element.addEventListener('wheel',wheel,{passive:false});return()=>element.removeEventListener('wheel',wheel);
 },[host]);
 return <div className="movement-overlay" ref={overlay} data-testid="movement-overlay">
  <svg className="movement-arrows" viewBox="0 0 1000 1000" preserveAspectRatio="none" aria-hidden="true">
   <defs>{geometry.arrows.map(a=><marker key={a.division} id={`${prefix}-${a.division}`} viewBox="0 0 8 8" refX="8" refY="4" markerWidth="8" markerHeight="8" orient="auto"><path d="M0 0 L8 4 L0 8 Z" fill={`rgb(${a.color.join(',')})`}/></marker>)}</defs>
   {geometry.arrows.map(a=><polyline key={a.division} data-movement-division={a.division} data-map-points={JSON.stringify(a.points.map(({x,y})=>({x,y})))} points={a.points.map(p=>`${p.left*10},${p.top*10}`).join(' ')} fill="none" stroke={`rgb(${a.color.join(',')})`} strokeWidth="2" vectorEffect="non-scaling-stroke" markerEnd={`url(#${prefix}-${a.division})`}/>)}
  </svg>
  {geometry.moving.map(m=>m.left<0||m.left>100||m.top<0||m.top>100?null:<button key={m.division} type="button" className="division-marker moving-division-marker" data-moving-division={m.division} data-map-x={m.x} data-map-y={m.y} data-ratio={m.ratio} style={{left:`${m.left}%`,top:`${m.top}%`,borderColor:`rgb(${m.color.join(',')})`}} aria-pressed={selected.includes(m.division)} aria-label={t('map-moving-division',{division:m.division,nation:m.nation,remaining:m.remaining})} title={t('movement-remaining',{remaining:m.remaining})} onClick={()=>onActivate(m)} onContextMenu={e=>{if(onContextPick(m.province))e.preventDefault();}}>{m.division} · {t('movement-remaining',{remaining:m.remaining})}</button>)}
 </div>;
}
export function MovementControls({enabled,count,busy,targetMode,onTargetMode,onStop,feedback,reasonKey}:{enabled:boolean;count:number;busy:boolean;targetMode:boolean;onTargetMode:()=>void;onStop:()=>void;feedback:MovementFeedback[];reasonKey:string|null}){
 const {t}=useLocalization();
 return <div className="movement-controls" data-testid="movement-controls" data-busy={busy}>
  {reasonKey?<p role="status">{t(reasonKey)}</p>:enabled&&<><span>{t('movement-select',{count})}</span><button type="button" aria-pressed={targetMode} disabled={!count||busy} onClick={onTargetMode}>{t('movement-target-mode')}</button><button type="button" disabled={!count||busy} onClick={onStop}>{t('movement-stop')}</button></>}
  <div role="status" aria-live="polite">{feedback.map(row=><p key={row.division} data-movement-result={row.division} data-status={row.status}>{t(`movement-${busy&&row.status!=='not-sent'?'pending':row.status}`,{division:row.division})}{!busy&&row.reasonKey?` · ${t(row.reasonKey)}`:''}</p>)}</div>
 </div>;
}
