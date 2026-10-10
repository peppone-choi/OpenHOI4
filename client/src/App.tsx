import { useEffect, useMemo, useRef, useState } from 'react';
import { applyServerMessage, connect } from './network';
import type { ProductionCommand, ProductionView, TimeCommand, TimeState, WorldView } from './proto/protocol';
import { Localization, translatorFor, type Language, loadPackCatalogs, translatorWithPack } from './i18n';
import {ProductionPanel} from './components/ProductionPanel';
import { EconomyPanel } from './components/EconomyPanel';
import { EconomyResponses, type EconomyDisplay } from './economyResponses';
import { MilitaryResponses, type MilitaryDisplay } from './militaryResponses';
import { MilitaryPanel } from './components/MilitaryPanel';
import {CommandLedger} from './commandLedger';
import {MilitaryCommands,emptyTrainingFeedback,type TrainingCommand} from './militaryCommands';
import {MilitaryLedgerResponses,emptyMilitaryLedger,type MilitaryLedgerDisplay} from './militaryLedgerResponses';
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
  const [economy,setEconomy]=useState<EconomyDisplay>({view:null,status:'loading',reasonKey:null});
  const [military,setMilitary]=useState<MilitaryDisplay>({view:null,status:'disconnected',reasonKey:null});
  const [militaryOpen,setMilitaryOpen]=useState(false);
  const militaryOpenRef=useRef(false);
  const militaryResponses=useRef(new MilitaryResponses());
  const militaryCommands=useRef(new MilitaryCommands());
  const commandLedger=useRef(new CommandLedger());
  const [trainingFeedback,setTrainingFeedback]=useState(emptyTrainingFeedback);
  const militaryLedgerResponses=useRef(new MilitaryLedgerResponses());
  const [militaryLedger,setMilitaryLedger]=useState<MilitaryLedgerDisplay>(emptyMilitaryLedger);
  const militaryToggle=useRef<HTMLButtonElement|null>(null);
  const [militaryNation,setMilitaryNation]=useState<number|null>(null);
  const economyResponses=useRef(new EconomyResponses());
  const economyNation=useRef<number|null>(null);
  const activeConnection=useRef<object|null>(null);
  const productionConnection=useRef<object|null>(null);
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
  const [connectionAttempt,setConnectionAttempt]=useState(0);
  const [state, setState] = useState<TimeState | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const socket = useRef<ReturnType<typeof connect> | null>(null);

  const transmitEconomy=(client:ReturnType<typeof connect>,request:string)=>client.send({type:'Query',request,kind:'economy'});
  const queryEconomy=(client:ReturnType<typeof connect>)=>{
    economyResponses.current.issue(client,request=>transmitEconomy(client,request));
  };
  const transmitMilitary=(client:ReturnType<typeof connect>,request:string)=>client.send({type:'Query',request,kind:'military'});
  const queryMilitaryLedger=(client:ReturnType<typeof connect>)=>militaryLedgerResponses.current.issue(client,(request,kind)=>client.send({type:'Query',request,kind}));
  const selectMilitaryLedger=(template:string|null)=>{
    militaryLedgerResponses.current.select(template);setMilitaryLedger({...militaryLedgerResponses.current.display});
    const client=socket.current;if(client&&activeConnection.current===client)queryMilitaryLedger(client);
  };
  const selectMilitaryNation=(nation:number|null)=>{
    setMilitaryNation(nation);const client=socket.current;
    militaryCommands.current.close();setTrainingFeedback(emptyTrainingFeedback());
    setMilitary({view:null,status:'loading',reasonKey:null});
    if(client&&activeConnection.current===client){militaryCommands.current.begin(client,controlledNation,militaryResponses.current.issuedSerial);queryMilitary(client);}
    militaryLedgerResponses.current.close();
    if(client&&activeConnection.current===client){militaryLedgerResponses.current.begin(client);militaryLedgerResponses.current.setBase(military.view);}
    setMilitaryLedger(emptyMilitaryLedger());
  };
  const queryMilitary=(client:ReturnType<typeof connect>)=>{
    if(militaryOpenRef.current)militaryResponses.current.issue(client,request=>transmitMilitary(client,request));
  };
  const closeMilitary=()=>{
    militaryOpenRef.current=false;setMilitaryOpen(false);militaryResponses.current.close();
    militaryCommands.current.close();setTrainingFeedback(emptyTrainingFeedback());
    militaryLedgerResponses.current.close();setMilitaryLedger(emptyMilitaryLedger());
    setMilitary({view:null,status:'disconnected',reasonKey:null});setMilitaryNation(null);
    militaryToggle.current?.focus();
  };
  const openMilitary=()=>{
    militaryOpenRef.current=true;setMilitaryOpen(true);setMilitaryNation(null);
    const client=socket.current;
    const active=client&&activeConnection.current===client;
    setMilitary({view:null,status:active?'loading':'disconnected',reasonKey:null});
    if(active){militaryCommands.current.begin(client,controlledNation);setTrainingFeedback(emptyTrainingFeedback());militaryResponses.current.begin(client);militaryLedgerResponses.current.begin(client);setMilitaryLedger(emptyMilitaryLedger());queryMilitary(client);}
  };
  const resetMilitary=()=>{
    militaryResponses.current.close();setMilitaryNation(null);
    militaryCommands.current.close();setTrainingFeedback(emptyTrainingFeedback());
    militaryLedgerResponses.current.close();setMilitaryLedger(emptyMilitaryLedger());
    setMilitary({view:null,status:'loading',reasonKey:null});
  };
  const selectEconomyNation=(id:number|null)=>{
    economyNation.current=id;setNationId(id??undefined);
    setEconomy({view:null,status:activeConnection.current?'loading':'disconnected',reasonKey:null});
    const client=socket.current;
    if(client&&activeConnection.current===client){economyResponses.current.begin(client,id);queryEconomy(client);}
  };
  useEffect(() => {
    resetMilitary();
    setEconomy({view:null,status:'loading',reasonKey:null});
    productionConnection.current=null;setProduction(null);
    const client = connect(message => {
      if(activeConnection.current!==client)return;
      if (message.type === 'Welcome') {
        setConnection(message.accepted ? 'connected' : 'disconnected');
        setNotice(message.reason_key);
        setPackHash(message.packs[0]?.hash??'');
      }
      if (message.type === 'Notice') setNotice(message.key);
      if(message.type==='CommandResult'){
        const kind=commandLedger.current.accept(client,message);
        if(kind){
          setNotice(message.reason_key);
          if(kind==='MilitaryCommand'&&militaryCommands.current.accept(client,message,militaryResponses.current.issuedSerial)){
            setTrainingFeedback({...militaryCommands.current.feedback});queryMilitary(client);
          }
        }
      }
      if(message.type==='Snapshot'||message.type==='Delta'){client.send({type:'Query',request:'world',kind:'world'});client.send({type:'Query',request:'production',kind:'production'});queryEconomy(client);queryMilitary(client);queryMilitaryLedger(client);}
      if(message.type==='MilitaryResult'||message.type==='QueryResult'){
        const accepted=militaryResponses.current.accept(client,message);
        if(accepted){militaryCommands.current.setAuthority(client,accepted.view,militaryResponses.current.issuedSerial);setTrainingFeedback({...militaryCommands.current.feedback});setMilitary(accepted);setMilitaryLedger({...militaryLedgerResponses.current.setBase(accepted.view)});queryMilitaryLedger(client);militaryResponses.current.followUp(client,request=>transmitMilitary(client,request));}
      }
      if(message.type==='MilitaryNormalLedgerCapabilityResult'||message.type==='MilitaryNormalLedgerResult'||message.type==='QueryResult'){
        const accepted=militaryLedgerResponses.current.accept(client,message);
        if(accepted)setMilitaryLedger({...accepted});
        queryMilitaryLedger(client);
      }
      if(message.type==='EconomyResult'||message.type==='QueryResult'){
        const accepted=economyResponses.current.accept(client,message);
        if(accepted){
          setEconomy(accepted);
          economyResponses.current.followUp(client,request=>transmitEconomy(client,request));
        }
      }
      if(message.type==='ProductionResult'&&message.request==='production'){
        productionConnection.current=message.supported?client:null;
        setProduction(message.production);
      }
      if(message.type==='WorldResult' && message.world){
        const incoming=message.world,previous=latestWorld.current;
        if(previous&&(incoming.map_id!==previous.map_id||incoming.width!==previous.width||incoming.height!==previous.height||JSON.stringify(incoming.province_ids)!==JSON.stringify(previous.province_ids)))throw new Error('map-data-error');
        const next=message.request==='world'||!previous?incoming:{...previous,tick:incoming.tick,nations:previous.nations.map(n=>incoming.nations.find(v=>v.id===n.id)??n),states:previous.states.map(s=>incoming.states.find(v=>v.id===s.id)??s),provinces:incoming.provinces};
        validateWorldDisplay(next);latestWorld.current=next;setWorld(next);
        if(!loadedCatalogs.current){loadedCatalogs.current=true;loadPackCatalogs().then(setCatalogs).catch(()=>setNotice('pack-localization-error'));}
      }
      setState(current => applyServerMessage(current, message));
    }, reasonKey => {
      if(activeConnection.current!==client)return;
      activeConnection.current=null;
      productionConnection.current=null;
      militaryResponses.current.end(client);militaryCommands.current.end(client);setTrainingFeedback(emptyTrainingFeedback());commandLedger.current.end(client);
      militaryLedgerResponses.current.end(client);setMilitaryLedger({...militaryLedgerResponses.current.display});
      setMilitary(current=>({...current,status:current.view?'stale':'disconnected'}));
      economyResponses.current.end(client);
      setEconomy(current=>({...current,status:current.view?'stale':'disconnected'}));
      setConnection('disconnected');
      if (reasonKey) setNotice(reasonKey);
    }, joinNation);
    socket.current = client;
    activeConnection.current=client;commandLedger.current.begin(client);
    if(militaryOpenRef.current)militaryCommands.current.begin(client,controlledNation);
    if(militaryOpenRef.current){militaryResponses.current.begin(client);militaryLedgerResponses.current.begin(client);}
    economyResponses.current.begin(client,economyNation.current);
    return () => {
      if(activeConnection.current===client)activeConnection.current=null;
      if(productionConnection.current===client)productionConnection.current=null;
      economyResponses.current.end(client);
      militaryResponses.current.end(client);militaryCommands.current.end(client);setTrainingFeedback(emptyTrainingFeedback());commandLedger.current.end(client);
      militaryLedgerResponses.current.end(client);
      client.close();
      if(socket.current===client)socket.current=null;
    };
  }, [joinNation,connectionAttempt]);
  const reconnect=()=>{
    resetMilitary();
    activeConnection.current=null;
    productionConnection.current=null;setProduction(null);
    if(socket.current)economyResponses.current.end(socket.current);
    setEconomy({view:null,status:'loading',reasonKey:null});
    setConnection('connecting');setNotice(null);setState(null);
    setConnectionAttempt(attempt=>attempt+1);
  };
  const command = (command: TimeCommand) => {const client=socket.current;if(client&&activeConnection.current===client)commandLedger.current.issue(client,{type:'Command',command},message=>client.send(message));};
  const productionCommand=(command:ProductionCommand)=>{
    const client=socket.current;
    if(client&&activeConnection.current===client&&productionConnection.current===client&&connection==='connected'&&controlledNation!==null){
      commandLedger.current.issue(client,{type:'ProductionCommand',command},message=>client.send(message));
    }
  };
  const trainingCommand=(command:TrainingCommand)=>{
    const client=socket.current;
    if(client&&activeConnection.current===client&&connection==='connected'){
      militaryCommands.current.issue(client,militaryNation,command,()=>commandLedger.current.issue(client,{type:'MilitaryCommand',command},message=>client.send(message)));
      setTrainingFeedback({...militaryCommands.current.feedback});
    }
  };
  const canTrain=(command:TrainingCommand)=>{const client=socket.current;return !!client&&activeConnection.current===client&&connection==='connected'&&militaryCommands.current.can(client,militaryNation,command);};
  const controlNation=(n:number)=>{const tag=world?.nations.find(v=>v.id===n)?.tag;if(tag){
    resetMilitary();
    activeConnection.current=null;
    productionConnection.current=null;
    if(socket.current)economyResponses.current.end(socket.current);
    economyNation.current=n;setNationId(n);setEconomy({view:null,status:'loading',reasonKey:null});
    setConnection('connecting');setNotice(null);setState(null);setControlledNation(n);setProduction(null);setJoinNation(tag);
  }};
  const query=(kind:string)=>socket.current?.send({type:'Query',request:kind,kind});
  const select=(id:number|null)=>{setSelected(id);const p=world?.provinces.find(p=>p.id===id);selectEconomyNation(p?.owner??null);setStateId(p?.state??undefined);if(p?.owner!=null)query(`nation:${p.owner}`);if(p?.state!=null)query(`state:${p.state}`);};
  return <Localization.Provider value={{ language, t }}>
    <GameShell connection={connection} onLanguage={setLanguage}>
      <TimeControls state={state} connected={connection === 'connected'} onCommand={command} />
      {world && catalogs && <div className="world-layout"><MapView world={world} packHash={packHash} selected={selected} onSelect={select}/><aside className="world-detail">{selected!==null&&world.provinces.find(p=>p.id===selected)?.state==null?<p>{t('map-no-state')}</p>:<NationalPanels world={world} nationId={nationId} stateId={stateId} onNationSelect={id=>{selectEconomyNation(id);query(`nation:${id}`);}} onStateSelect={id=>{setStateId(id);const owner=world.states.find(s=>s.id===id)?.owner;selectEconomyNation(owner??null);query(`state:${id}`);if(owner!==undefined)query(`nation:${owner}`);}}/>}</aside></div> }
      <EconomyPanel display={economy} world={world} nationId={nationId??null} onNationSelect={selectEconomyNation}/>
      <button ref={militaryToggle} aria-expanded={militaryOpen} aria-controls="military-panel" onClick={militaryOpen?closeMilitary:openMilitary}>{t(militaryOpen?'military-close':'military-open')}</button>
      {militaryOpen&&<MilitaryPanel display={military} world={world} nationId={militaryNation} onNationSelect={selectMilitaryNation} onClose={closeMilitary} ledger={militaryLedger} onLedgerSelect={selectMilitaryLedger} controls={{nation:controlledNation,feedback:trainingFeedback,can:canTrain,send:trainingCommand}}/>}
      {connection==='disconnected'&&<button onClick={reconnect}>{t('economy-reconnect')}</button>}
      {production&&<ProductionPanel view={production} world={world} controlledNation={controlledNation} onControl={controlNation} onCommand={productionCommand} connected={connection==='connected'&&productionConnection.current===activeConnection.current}/>}
      {notice && <p role="alert">{t(notice)}</p>}
    </GameShell>
  </Localization.Provider>;
}
