import { useEffect, useMemo, useRef, useState } from 'react';
import { applyServerMessage, connect } from './network';
import type { ProductionCommand, ProductionView, TimeCommand, TimeState, WorldView } from './proto/protocol';
import { Localization, translatorFor, type Language, loadPackCatalogs, translatorWithPack } from './i18n';
import {ProductionPanel} from './components/ProductionPanel';
import { GameShell } from './components/GameShell';
import { NationalPanels } from './components/NationalPanels';
import { TimeControls } from './components/TimeControls';
import { MapView } from './map/MapView';
import { validateWorldDisplay } from './map/model';
export function App() {
  const [language, setLanguage] = useState<Language>('en');
  const [catalogs,setCatalogs]=useState<Record<Language,string>|null>(null);
  const t=useMemo(()=>catalogs?translatorWithPack(language,catalogs):translatorFor(language),[language,catalogs]);
  const [production,setProduction]=useState<ProductionView|null>(null);
  const [controlledNation,setControlledNation]=useState<number|null>(null);
  const [joinNation,setJoinNation]=useState<string|null>(null);
  const [world,setWorld]=useState<WorldView|null>(null);
  const latestWorld=useRef<WorldView|null>(null);
  const [packHash,setPackHash]=useState('');
  const [selected,setSelected]=useState<number|null>(null);
  const [nationId,setNationId]=useState<number>();
  const [stateId,setStateId]=useState<number>();
  const loadedCatalogs=useRef(false);
  useEffect(() => {
    document.documentElement.lang = language;
    document.title = t('title');
  }, [language, t]);
  const [connection, setConnection] = useState('connecting');
  const [state, setState] = useState<TimeState | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const socket = useRef<ReturnType<typeof connect> | null>(null);
  const sequence = useRef(0n);
  useEffect(() => {
    const client = connect(message => {
      if (message.type === 'Welcome') {
        setConnection(message.accepted ? 'connected' : 'disconnected');
        setNotice(message.reason_key);
        setPackHash(message.packs[0]?.hash??'');
      }
      if (message.type === 'Notice') setNotice(message.key);
      if (message.type === 'CommandResult') setNotice(message.reason_key);
      if(message.type==='Snapshot'||message.type==='Delta'){client.send({type:'Query',request:'world',kind:'world'});client.send({type:'Query',request:'production',kind:'production'});}
      if(message.type==='ProductionResult')setProduction(message.production);
      if(message.type==='WorldResult' && message.world){
        const incoming=message.world,previous=latestWorld.current;
        if(previous&&(incoming.map_id!==previous.map_id||incoming.width!==previous.width||incoming.height!==previous.height||JSON.stringify(incoming.province_ids)!==JSON.stringify(previous.province_ids)))throw new Error('map-data-error');
        const next=message.request==='world'||!previous?incoming:{...previous,tick:incoming.tick,nations:previous.nations.map(n=>incoming.nations.find(v=>v.id===n.id)??n),states:previous.states.map(s=>incoming.states.find(v=>v.id===s.id)??s),provinces:incoming.provinces};
        validateWorldDisplay(next);latestWorld.current=next;setWorld(next);
        if(!loadedCatalogs.current){loadedCatalogs.current=true;loadPackCatalogs().then(setCatalogs).catch(()=>setNotice('pack-localization-error'));}
      }
      setState(current => applyServerMessage(current, message));
    }, reasonKey => {
      setConnection('disconnected');
      if (reasonKey) setNotice(reasonKey);
    }, joinNation);
    socket.current = client;
    return () => { client.close(); socket.current = null; };
  }, [joinNation]);
  const command = (command: TimeCommand) => socket.current?.send({ type: 'Command', sequence: (++sequence.current).toString(), command });
  const productionCommand=(command:ProductionCommand)=>socket.current?.send({type:'ProductionCommand',sequence:(++sequence.current).toString(),command});
  const controlNation=(n:number)=>{const tag=world?.nations.find(v=>v.id===n)?.tag;if(tag){setConnection('connecting');setNotice(null);setState(null);setControlledNation(n);setProduction(null);setJoinNation(tag);}};
  const query=(kind:string)=>socket.current?.send({type:'Query',request:kind,kind});
  const select=(id:number|null)=>{setSelected(id);const p=world?.provinces.find(p=>p.id===id);setNationId(p?.owner??undefined);setStateId(p?.state??undefined);if(p?.owner!=null)query(`nation:${p.owner}`);if(p?.state!=null)query(`state:${p.state}`);};
  return <Localization.Provider value={{ language, t }}>
    <GameShell connection={connection} onLanguage={setLanguage}>
      <TimeControls state={state} connected={connection === 'connected'} onCommand={command} />
      {world && catalogs && <div className="world-layout"><MapView world={world} packHash={packHash} selected={selected} onSelect={select}/><aside className="world-detail">{selected!==null&&world.provinces.find(p=>p.id===selected)?.state==null?<p>{t('map-no-state')}</p>:<NationalPanels world={world} nationId={nationId} stateId={stateId} onNationSelect={id=>{setNationId(id);query(`nation:${id}`);}} onStateSelect={id=>{setStateId(id);const owner=world.states.find(s=>s.id===id)?.owner;setNationId(owner);query(`state:${id}`);if(owner!==undefined)query(`nation:${owner}`);}}/>}</aside></div> }
      {production&&<ProductionPanel view={production} world={world} controlledNation={controlledNation} onControl={controlNation} onCommand={productionCommand} connected={connection==='connected'}/>}
      {notice && <p role="alert">{t(notice)}</p>}
    </GameShell>
  </Localization.Provider>;
}
