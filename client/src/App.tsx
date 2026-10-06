import { useEffect, useRef, useState } from 'react';
import { applyServerMessage, connect } from './network';
import type { TimeCommand, TimeState } from './proto/protocol';
import { Localization, translatorFor, type Language } from './i18n';
import { GameShell } from './components/GameShell';
import { TimeControls } from './components/TimeControls';
export function App() {
  const [language, setLanguage] = useState<Language>('en');
  const t = translatorFor(language);
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
      {notice && <p role="alert">{t(notice)}</p>}
    </GameShell>
  </Localization.Provider>;
}
