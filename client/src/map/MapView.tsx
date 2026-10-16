import { useEffect,useMemo,useRef,useState } from 'react';
import type { MilitaryView,WorldView } from '../proto/protocol';
import { useLocalization } from '../i18n';
import { decodeIndex,fnv,validateWorldDisplay } from './model';
import { createMap,type MapCamera } from './renderer';
import { isMapMetadata } from '../network';
import {DivisionMarkers} from './DivisionMarkersOverlay';
import {buildDivisionMarkers,divisionIdsForMarker,provinceAnchors,type MapPresentation,type DivisionMarker} from './divisionMarkers';
import {MovementOverlay,MovementControls} from './MovementOverlay';
import {buildMovementOverlay,type MovingMarker} from './movementOverlay';
import type {MovementDisplay,MovementFeedback} from '../movementCommands';
type MapInstance=Awaited<ReturnType<typeof createMap>>;
/** Reject redirects before any target request; map files belong to this host. */
async function fetchMapResource(path:string,signal:AbortSignal){
 const response=await fetch(new URL(path,location.origin).href,{signal,mode:'same-origin',redirect:'error'});
 if(!response.ok)throw new Error('map-data-error');
 return response;
}
export function MapView({world,packHash,selected,onSelect,divisions=null,controlledNation=null,movement=null,movementBusy=false,movementFeedback=[],onMovement=()=>false}:{world:WorldView;packHash:string;selected:number|null;onSelect:(id:number|null)=>void;divisions?:MilitaryView|null;controlledNation?:number|null;movement?:MovementDisplay|null;movementBusy?:boolean;movementFeedback?:MovementFeedback[];onMovement?:(ids:readonly string[],destination:number|null)=>boolean}){
 const {t}=useLocalization();const host=useRef<HTMLDivElement>(null),instance=useRef<MapInstance|null>(null);
 const [mode,setMode]=useState('map-mode-owner'),[hover,setHover]=useState<number|null>(null),[error,setError]=useState<string|null>(null),[ready,setReady]=useState(false);
 const [mapIndex,setMapIndex]=useState<{mapId:string;packHash:string;anchors:ReturnType<typeof provinceAnchors>}|null>(null),[camera,setCamera]=useState<MapPresentation|null>(null);
 const [selectedDivisions,setSelectedDivisions]=useState<string[]>([]),[targetMode,setTargetMode]=useState(false);
 useEffect(()=>{setSelectedDivisions([]);setTargetMode(false);},[controlledNation]);
 useEffect(()=>{const own=new Set(divisions?.divisions.filter(d=>d.nation===controlledNation).map(d=>d.id));setSelectedDivisions(old=>old.every(id=>own.has(id))?old:old.filter(id=>own.has(id)));if(!divisions)setTargetMode(false);},[divisions,controlledNation]);
 useEffect(()=>{if(movement?.status!=='ready'||!selectedDivisions.length)setTargetMode(false);},[movement?.status,selectedDivisions.length]);
 useEffect(()=>{const escape=(e:KeyboardEvent)=>{if(e.key==='Escape'){setSelectedDivisions([]);setTargetMode(false);}};document.addEventListener('keydown',escape);return()=>document.removeEventListener('keydown',escape);},[]);
 const movementReady=!!divisions&&movement?.status==='ready';
 const markers=useMemo(()=>{
  if(!mapIndex||mapIndex.mapId!==world.map_id||mapIndex.packHash!==packHash||!camera)return {items:[],geometry:null,error:null};
  try{const geometry=buildMovementOverlay(movementReady?movement!.view:null,world,mapIndex.anchors,camera,controlledNation);return {items:buildDivisionMarkers(divisions,world,mapIndex.anchors,geometry.excluded),geometry,error:null};}catch{return {items:[],geometry:null,error:'map-data-error'};}
 },[divisions,world,mapIndex,packHash,camera,movementReady,movement,controlledNation]);
 const sendMove=(id:number|null)=>{
  if(id===null||!world.province_ids.includes(id)||!movementReady||movementBusy||!selectedDivisions.length||markers.error)return false;
  const sent=onMovement(selectedDivisions,id);if(sent)setTargetMode(false);return sent;
 };
 const pickProvince=(id:number|null)=>{if(id===null){setSelectedDivisions([]);setTargetMode(false);}else if(targetMode&&sendMove(id))return;onSelect(id);};
 const pickMarker=(marker:DivisionMarker)=>{if(targetMode&&sendMove(marker.province))return;onSelect(marker.province);const ids=divisionIdsForMarker(divisions,marker,controlledNation);if(ids.length)setSelectedDivisions(ids);};
 const pickMoving=(marker:MovingMarker)=>{if(targetMode&&sendMove(marker.province))return;onSelect(marker.province);if(marker.nation===controlledNation)setSelectedDivisions([marker.division]);};
 const isSelected=(marker:DivisionMarker)=>divisionIdsForMarker(divisions,marker,controlledNation).some(id=>selectedDivisions.includes(id));
 const current=useRef({world,selected,onSelect:pickProvince,onContextPick:sendMove,mode});current.current={world,selected,onSelect:pickProvince,onContextPick:sendMove,mode};
 useEffect(()=>{
  let stopped=false;const controller=new AbortController();setReady(false);setError(null);setMapIndex(null);setCamera(null);
  async function load(){
   try{
    validateWorldDisplay(world);
    const response=await fetchMapResource(`/maps/${encodeURIComponent(world.map_id)}/metadata`,controller.signal);
    const candidate:unknown=await response.json();
    if(!isMapMetadata(candidate)||candidate.pack_hash!==packHash||candidate.map_id!==world.map_id||candidate.width!==world.width||candidate.height!==world.height||JSON.stringify(candidate.province_ids)!==JSON.stringify(world.province_ids))throw new Error('map-data-error');
    const meta=candidate;
    const raw=await fetchMapResource(`/maps/${encodeURIComponent(world.map_id)}/index.bin?pack=${encodeURIComponent(packHash)}`,controller.signal);
    const buffer=await raw.arrayBuffer();const bytes=new Uint8Array(buffer);if(buffer.byteLength!==Number(meta.byte_length)||fnv(bytes)!==meta.index_hash||raw.headers.get('x-pack-hash')!==packHash)throw new Error('map-data-error');
    const index=decodeIndex(buffer,world.width,world.height,world.province_ids);
    if(stopped)return;
    setMapIndex({mapId:world.map_id,packHash,anchors:provinceAnchors(world,index)});
    const force=new URLSearchParams(location.search).get('forceWebGL')==='1'||!window.isSecureContext||!navigator.gpu;
    async function start(forceWebGL:boolean,camera?:MapCamera){
     const result=await createMap(host.current!,meta,bytes,index,current.current.world,setHover,id=>current.current.onSelect(id),()=>{
      setReady(false);
      instance.current?.dispose();instance.current=null;
      if(!forceWebGL&&!stopped)start(true).catch(()=>setError('map-backend-unavailable'));else if(!stopped)setError('map-backend-unavailable');
     },nextCamera=>{
      if(stopped)return;setReady(false);setHover(null);instance.current?.dispose();instance.current=null;
      start(true,nextCamera).catch(()=>{if(!stopped)setError('map-backend-unavailable');});
     },forceWebGL,{onContextPick:id=>current.current.onContextPick(id),onCamera:next=>{if(!stopped)setCamera(next);},camera,mode:current.current.mode,selected:current.current.selected,signal:controller.signal});
     if(stopped){result.dispose();return;}instance.current=result;result.update(current.current.world,current.current.mode,current.current.selected);setReady(true);
    }
    try{await start(force);}catch{if(force)throw new Error('map-backend-unavailable');await start(true).catch(()=>{throw new Error('map-backend-unavailable');});}
   }catch(e){if(!stopped)setError(e instanceof Error&&e.message==='map-backend-unavailable'?'map-backend-unavailable':'map-data-error');}
  }void load();return()=>{stopped=true;controller.abort();instance.current?.dispose();instance.current=null;};
 },[world.map_id,packHash]);
 useEffect(()=>{instance.current?.update(world,mode,selected);},[world,mode,selected,ready]);
 const p=world.provinces.find(p=>p.id===(hover??selected));
 return <section className="map-section" aria-label={t('map-title')}>
  <nav className="map-modes" aria-label={t('map-modes')}>{world.mode_keys.map(key=><button key={key} aria-pressed={mode===key} onClick={()=>setMode(key)}>{t(key)}</button>)}<button onClick={()=>instance.current?.reset()}>{t('map-reset')}</button></nav>
  <div className="province-map-container"><div ref={host} className="province-map" data-testid="province-map" aria-label={t('map-title')} />{ready&&!error&&!markers.error&&camera&&<DivisionMarkers markers={markers.items} camera={camera} onSelect={pickMarker} host={host} isSelected={isSelected} onContextPick={sendMove}/>}
   {ready&&!error&&!markers.error&&movementReady&&markers.geometry&&<MovementOverlay geometry={markers.geometry} selected={selectedDivisions} onActivate={pickMoving} onContextPick={sendMove} host={host}/>}</div>
  {movement&&<MovementControls enabled={movementReady&&!error&&!markers.error} count={selectedDivisions.length} busy={movementBusy} targetMode={targetMode} onTargetMode={()=>setTargetMode(v=>!v)} onStop={()=>{if(!movementBusy&&movementReady&&onMovement(selectedDivisions,null))setTargetMode(false);}} feedback={movementFeedback} reasonKey={movement.status==='unsupported'?movement.reasonKey:null}/>}
  {error||markers.error?<p role="alert">{t(error??markers.error!)}</p>:!ready?<p role="status">{t('map-loading')}</p>:null}
  <div className="map-selection"><span>{t('province-label')}: <strong data-testid="selected-province">{selected??t('no-value')}</strong></span><span data-testid="map-hover">{p?`${t('province-label')} ${p.id} · ${t(p.terrain_key)}`:t('map-help')}</span><button disabled={selected===null} onClick={()=>pickProvince(null)}>{t('map-clear')}</button></div>
 </section>;
}
