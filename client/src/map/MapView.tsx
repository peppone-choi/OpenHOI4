import { useEffect,useRef,useState } from 'react';
import type { MapMetadata,WorldView } from '../proto/protocol';
import { useLocalization } from '../i18n';
import { decodeIndex,fnv } from './model';
import { createMap } from './renderer';
type MapInstance=Awaited<ReturnType<typeof createMap>>;
export function MapView({world,packHash,selected,onSelect}:{world:WorldView;packHash:string;selected:number|null;onSelect:(id:number|null)=>void}){
 const {t}=useLocalization();const host=useRef<HTMLDivElement>(null),instance=useRef<MapInstance|null>(null);
 const current=useRef({world,selected,onSelect});current.current={world,selected,onSelect};
 const [mode,setMode]=useState('map-mode-owner'),[hover,setHover]=useState<number|null>(null),[error,setError]=useState<string|null>(null),[ready,setReady]=useState(false);
 useEffect(()=>{
  let stopped=false;const controller=new AbortController();setReady(false);setError(null);
  async function load(){
   try{
    const response=await fetch(`/maps/${encodeURIComponent(world.map_id)}/metadata`,{signal:controller.signal});if(!response.ok)throw new Error('map-data-error');
    const meta=await response.json() as MapMetadata;
    if(meta.pack_hash!==packHash||meta.map_id!==world.map_id||meta.width!==world.width||meta.height!==world.height||JSON.stringify(meta.province_ids)!==JSON.stringify(world.province_ids))throw new Error('map-data-error');
    const raw=await fetch(`/maps/${encodeURIComponent(world.map_id)}/index.bin?pack=${encodeURIComponent(packHash)}`,{signal:controller.signal});if(!raw.ok)throw new Error('map-data-error');
    const buffer=await raw.arrayBuffer();const bytes=new Uint8Array(buffer);if(buffer.byteLength!==Number(meta.byte_length)||fnv(bytes)!==meta.index_hash||raw.headers.get('x-pack-hash')!==packHash)throw new Error('map-data-error');
    const index=decodeIndex(buffer,world.width,world.height,world.province_ids);
    if(stopped)return;
    const force=new URLSearchParams(location.search).get('forceWebGL')==='1';
    async function start(forceWebGL:boolean){
     const result=await createMap(host.current!,meta,bytes,index,current.current.world,setHover,id=>current.current.onSelect(id),()=>{
      instance.current?.dispose();instance.current=null;
      if(!forceWebGL&&!stopped)start(true).catch(()=>setError('map-backend-unavailable'));else if(!stopped)setError('map-backend-unavailable');
     },forceWebGL);
     if(stopped){result.dispose();return;}instance.current=result;result.update(current.current.world,'map-mode-owner',current.current.selected);setReady(true);
    }
    try{await start(force);}catch{if(force)throw new Error('map-backend-unavailable');await start(true).catch(()=>{throw new Error('map-backend-unavailable');});}
   }catch(e){if(!stopped)setError(e instanceof Error&&e.message==='map-backend-unavailable'?'map-backend-unavailable':'map-data-error');}
  }void load();return()=>{stopped=true;controller.abort();instance.current?.dispose();instance.current=null;};
 },[world.map_id,packHash]);
 useEffect(()=>{instance.current?.update(world,mode,selected);},[world,mode,selected,ready]);
 const p=world.provinces.find(p=>p.id===(hover??selected));
 return <section className="map-section" aria-label={t('map-title')}>
  <nav className="map-modes" aria-label={t('map-modes')}>{world.mode_keys.map(key=><button key={key} aria-pressed={mode===key} onClick={()=>setMode(key)}>{t(key)}</button>)}<button onClick={()=>instance.current?.reset()}>{t('map-reset')}</button></nav>
  <div ref={host} className="province-map" data-testid="province-map" aria-label={t('map-title')} />
  {error?<p role="alert">{t(error)}</p>:!ready?<p role="status">{t('map-loading')}</p>:null}
  <div className="map-selection"><span>{t('province-label')}: <strong data-testid="selected-province">{selected??t('no-value')}</strong></span><span data-testid="map-hover">{p?`${t('province-label')} ${p.id} · ${t(p.terrain_key)}`:t('map-help')}</span><button disabled={selected===null} onClick={()=>onSelect(null)}>{t('map-clear')}</button></div>
 </section>;
}
