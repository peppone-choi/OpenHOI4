import { useEffect, useMemo, useRef, useState } from 'react';
import { applyServerMessage, connect } from './network';
import type { TimeCommand, TimeState, WorldView } from './proto/protocol';
import { Localization, translatorFor, type Language, loadPackCatalogs, translatorWithPack } from './i18n';
import { GameShell } from './components/GameShell';
import { NationalPanels } from './components/NationalPanels';
import { TimeControls } from './components/TimeControls';
export function App() {
  const [language, setLanguage] = useState<Language>('en');
  const [catalogs,setCatalogs]=useState<Record<Language,string>|null>(null);
  const t=useMemo(()=>catalogs?translatorWithPack(language,catalogs):translatorFor(language),[language,catalogs]);
  const [world,setWorld]=useState<WorldView|null>(null);
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
      }
      if (message.type === 'Notice') setNotice(message.key);
      if (message.type === 'CommandResult') setNotice(message.reason_key);
      if(message.type==='Snapshot'||message.type==='Delta') client.send({type:'Query',request:'world',kind:'world'});
      if(message.type==='WorldResult' && message.world){
        setWorld(message.world);
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
  return <Localization.Provider value={{ language, t }}>
    <GameShell connection={connection} onLanguage={setLanguage}>
      <TimeControls state={state} connected={connection === 'connected'} onCommand={command} />
      {world && catalogs && <NationalPanels world={world}/> }
      {notice && <p role="alert">{t(notice)}</p>}
    </GameShell>
  </Localization.Provider>;
}
