import { useEffect, useMemo, useRef, useState } from 'react';
import { applyServerMessage, connect } from './network';
import type { TimeCommand, TimeState, WorldView } from './proto/protocol';
import { Localization, translatorFor, type Language, loadPackCatalogs, translatorWithPack } from './i18n';
import { GameShell } from './components/GameShell';
import { NationalPanels } from './components/NationalPanels';
import { TimeControls } from './components/TimeControls';
import { MapView } from './map/MapView';
export function App() {
  const [language, setLanguage] = useState<Language>('en');
  const [catalogs,setCatalogs]=useState<Record<Language,string>|null>(null);
  const t=useMemo(()=>catalogs?translatorWithPack(language,catalogs):translatorFor(language),[language,catalogs]);
  const [world,setWorld]=useState<WorldView|null>(null);
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
      if(message.type==='Snapshot'||message.type==='Delta') client.send({type:'Query',request:'world',kind:'world'});
      if(message.type==='WorldResult' && message.world){
        if(message.request==='world')setWorld(message.world);
        else setWorld(previous=>previous?{...previous,tick:message.world!.tick,nations:previous.nations.map(n=>message.world!.nations.find(v=>v.id===n.id)??n),states:previous.states.map(s=>message.world!.states.find(v=>v.id===s.id)??s),provinces:message.world!.provinces}:message.world);
        if(!loadedCatalogs.current){loadedCatalogs.current=true;loadPackCatalogs().then(setCatalogs).catch(()=>setNotice('pack-localization-error'));}
      }
      setState(current => applyServerMessage(current, message));
    }, reasonKey => {
      setConnection('disconnected');
      if (reasonKey) setNotice(reasonKey);
    });
    socket.current = client;
    return () => { client.close(); socket.current = null; };
  }, []);
  const command = (command: TimeCommand) => socket.current?.send({ type: 'Command', sequence: (++sequence.current).toString(), command });
  const query=(kind:string)=>socket.current?.send({type:'Query',request:kind,kind});
  const select=(id:number|null)=>{setSelected(id);const p=world?.provinces.find(p=>p.id===id);setNationId(p?.owner??undefined);setStateId(p?.state??undefined);if(p?.owner!=null)query(`nation:${p.owner}`);if(p?.state!=null)query(`state:${p.state}`);};
  return <Localization.Provider value={{ language, t }}>
    <GameShell connection={connection} onLanguage={setLanguage}>
      <TimeControls state={state} connected={connection === 'connected'} onCommand={command} />
      {world && catalogs && <div className="world-layout"><MapView world={world} packHash={packHash} selected={selected} onSelect={select}/><aside className="world-detail"><NationalPanels world={world} nationId={nationId} stateId={stateId} onNationSelect={id=>{setNationId(id);query(`nation:${id}`);}} onStateSelect={id=>{setStateId(id);query(`state:${id}`);}}/></aside></div> }
      {notice && <p role="alert">{t(notice)}</p>}
    </GameShell>
  </Localization.Provider>;
}
