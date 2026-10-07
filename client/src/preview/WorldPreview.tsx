import { useEffect,useMemo,useRef,useState } from 'react';
import { createMap,type MapCamera } from '../map/renderer';
import type { MapMetadata } from '../proto/protocol';
import { loadPreview,previewWorld,type PreviewData,type PreviewMode } from './model';
import { PreviewHud } from './PreviewHud';
import { en,ko,type PreviewText } from './locales';
import { loadUnitAssets,unitScene,disposeUnits,type UnitAsset,type UnitSample,type UnitLighting } from './units3d';
import './preview.css';

type MapInstance=Awaited<ReturnType<typeof createMap>>;
type UnitConfig={schema:string;models:UnitAsset[];samples:UnitSample[];lighting:UnitLighting};
type LoadedUnits={models:Awaited<ReturnType<typeof loadUnitAssets>>;config:UnitConfig};
export function WorldPreview(){
 const [language,setLanguage]=useState<'ko'|'en'>('ko');const t:PreviewText=key=>(language==='ko'?ko:en)[key];
 const [data,setData]=useState<PreviewData|null>(null),[error,setError]=useState(false),[ready,setReady]=useState(false);
 const [mode,setMode]=useState<PreviewMode>('geography'),[borders,setBorders]=useState(true),[selected,setSelected]=useState<number|null>(null),[hover,setHover]=useState<number|null>(null),[region,setRegion]=useState('world'),[revision,setRevision]=useState(0);
 const [units,setUnits]=useState<LoadedUnits|null>(null),[showUnits,setShowUnits]=useState(true),[unitError,setUnitError]=useState(false);
 const host=useRef<HTMLDivElement>(null),instance=useRef<MapInstance|null>(null),savedCamera=useRef<MapCamera|undefined>(undefined);
 const world=useMemo(()=>data?previewWorld(data.meta,data.provinces,mode):null,[data,mode]);
 const current=useRef({world,selected});current.current={world,selected};
 useEffect(()=>{const controller=new AbortController();loadPreview(controller.signal).then(setData).catch(()=>{if(!controller.signal.aborted)setError(true);});return()=>controller.abort();},[]);
 useEffect(()=>{
  const controller=new AbortController();let loaded:LoadedUnits|null=null;
  async function load(){
   const response=await fetch('/preview/world/units.json',{signal:controller.signal,mode:'same-origin',redirect:'error'});
   if(!response.ok)throw new Error('preview-unit-data-error');const config=await response.json() as UnitConfig;
   if(config.schema!=='world-preview-unit-display-v1'||!Array.isArray(config.samples)||config.samples.some(s=>!['army','air','navy'].includes(s.asset)||![s.longitude,s.latitude,s.heading_degrees,s.scale,s.height,s.tilt_degrees].every(Number.isFinite)||Math.abs(s.latitude)>90||Math.abs(s.longitude)>180||s.scale<=0||s.height<0))throw new Error('preview-unit-data-error');
   const models=await loadUnitAssets(config.models,controller.signal);loaded={models,config};if(!controller.signal.aborted)setUnits(loaded);else disposeUnits(models.values());
  }
  void load().catch(()=>{if(!controller.signal.aborted)setUnitError(true);});
  return()=>{controller.abort();if(loaded)disposeUnits(loaded.models.values());};
 },[]);
 useEffect(()=>{document.title=t('title');document.documentElement.lang=language;},[language]);
 const keepCamera=()=>{
  const raw=host.current?.dataset.camera;
  if(raw&&data){const c=JSON.parse(raw);savedCamera.current={x:c.x-data.meta.width/2,y:data.meta.height/2-c.y,zoom:c.zoom};}
 };
 useEffect(()=>{
  if(!data||!world||!host.current)return;
  let stopped=false;const controller=new AbortController();setReady(false);setError(false);
  const meta:MapMetadata={schema_version:1,map_id:data.meta.map_id,width:data.meta.width,height:data.meta.height,province_ids:data.meta.province_ids,pack_hash:'preview',index_hash:data.meta.hashes['index.bin'],byte_length:String(data.bytes.length),style:{...data.meta.style,province_width_milli:borders?data.meta.style.province_width_milli:0}};
  const chosen=data.meta.regions[region];
  const camera=savedCamera.current??{x:chosen.longitude/360*meta.width,y:chosen.latitude/180*meta.height,zoom:chosen.zoom};
  async function start(forceWebGL:boolean,cam:MapCamera=camera){
   if(host.current){host.current.dataset.unitSamples='0';host.current.dataset.unitTriangles='0';}
   const result=await createMap(host.current!,meta,data!.bytes,data!.index,current.current.world!,setHover,setSelected,()=>{
    if(stopped)return;setReady(false);keepCamera();instance.current?.dispose();instance.current=null;
    if(!forceWebGL)void start(true,savedCamera.current).catch(()=>setError(true));else setError(true);
   },next=>{
    if(stopped)return;setReady(false);instance.current?.dispose();instance.current=null;
    void start(true,next).catch(()=>{if(!stopped)setError(true);});
   },forceWebGL,{camera:cam,mode:'map-mode-terrain',selected:current.current.selected,signal:controller.signal,
    sceneExtension:showUnits&&units?unitScene(units.models,units.config.samples,meta.width,meta.height,units.config.lighting,(count,triangles)=>{if(host.current){host.current.dataset.unitSamples=String(count);host.current.dataset.unitTriangles=String(triangles);}}):undefined});
   if(stopped){result.dispose();return;}instance.current=result;result.update(current.current.world!,'map-mode-terrain',current.current.selected);setReady(true);
  }
  const force=new URLSearchParams(location.search).get('forceWebGL')==='1'||!navigator.gpu||!window.isSecureContext;
  void start(force).catch(()=>{if(!stopped){if(!force)void start(true).catch(()=>{if(!stopped)setError(true);});else setError(true);}});
  return()=>{stopped=true;controller.abort();instance.current?.dispose();instance.current=null;};
 },[data,borders,region,revision,units,showUnits]);
 useEffect(()=>{if(world)instance.current?.update(world,'map-mode-terrain',selected);},[world,selected,ready]);
 const p=data?.provinces[(hover??selected??0)-1];
 return <main className="world-preview">
  <div className="preview-map" ref={host} data-testid="world-map" aria-label={t('map')}/>
  <PreviewHud t={t} slots={{unitDisplay:units?t('sample'):unitError?t('error'):t('unitPending')}}>
   <section className="preview-toolbar"><div className="preview-modes">{(['geography','provinces','terrain'] as const).map(key=><button key={key} disabled={key==='terrain'&&!data?.meta.elevation} aria-pressed={mode===key} onClick={()=>setMode(key)}>{t(key)}</button>)}</div><button aria-pressed={borders} onClick={()=>{keepCamera();setBorders(!borders);}}>{t('borders')}</button><button aria-pressed={showUnits} disabled={!units} onClick={()=>{keepCamera();setShowUnits(!showUnits);}}>{t('markers')}</button><button aria-label={t('locale')} onClick={()=>setLanguage(language==='ko'?'en':'ko')}>{language==='ko'?'EN':'한국어'}</button></section>
   <section className="preview-intro"><small>{t('explore')}</small><h1>{t('intro')}</h1><p>{t('description')}</p></section>
   <aside className="preview-inspector"><header><span className="inspector-dot"/>{t('selection')}</header><div className="inspector-body"><small>{p?t(p.kind):t('selectHelp')}</small><h2>{p?`#${p.id.toLocaleString(language)}`:t('nothing')}</h2>{p?<><div className="coordinate-grid"><span><small>{t('latitude')}</small>{p.latitude.toFixed(3)}°</span><span><small>{t('longitude')}</small>{p.longitude.toFixed(3)}°</span></div><div className="inspector-row"><span>{t('elevation')}</span><span>{p.elevation_m??t('pending')}</span></div><button disabled={selected===null} onClick={()=>setSelected(null)}>{t('close')}</button></>:<div className="inspector-placeholder" aria-hidden="true"><i/><i/><i/></div>}</div><details><summary>{t('details')}</summary><dl><dt>{t('source')}</dt><dd>Natural Earth {data?.meta.sources.find(s=>s.id==='land')?.version??'—'}</dd><dt>{t('resolution')}</dt><dd>{data?`${data.meta.width} × ${data.meta.height}`:'—'}</dd><dt>{t('count')}</dt><dd>{data?.provinces.length.toLocaleString(language)??'—'}</dd></dl><p>{t('limits')}</p></details></aside>
   <nav className="preview-regions" aria-label={t('zoom')}>{(['world','europe','korea','himalaya','americas'] as const).map(key=><button key={key} aria-pressed={region===key} onClick={()=>{savedCamera.current=undefined;setRegion(key);setRevision(n=>n+1);}}>{t(key)}</button>)}</nav>
   <footer className="preview-footer"><span className="status-dot"/>{t('notGame')}<span>{t('help')}</span></footer>
  </PreviewHud>
  {error?<div className="preview-message" role="alert">{t('error')}</div>:!ready?<div className="preview-message" role="status">{t('loading')}</div>:null}
 </main>;
}
