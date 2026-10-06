import { useEffect, useRef, useState } from 'react';
import { applyServerMessage, connect } from './network';
import type { TimeCommand, TimeState } from './proto/protocol';
import en from '../public/locales/en.json';
import ko from '../public/locales/ko.json';
export function App() {
  const [language, setLanguage] = useState<'en' | 'ko'>('en');
  const strings = language === 'ko' ? ko : en;
  const t = (key: string) => strings[key as keyof typeof strings] ?? key;
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
    }, () => setConnection('disconnected'));
    socket.current = client;
    return () => { client.close(); socket.current = null; };
  }, []);
  const command = (command: TimeCommand) => socket.current?.send({ type: 'Command', sequence: (++sequence.current).toString(), command });
  return <main data-openhoi-shell="">
    <header><div><h1>{t('title')}</h1><p>{t('subtitle')}</p></div>
      <label>{t('language')} <select value={language} onChange={event => setLanguage(event.target.value as 'en' | 'ko')}><option value="en">{t('language-en')}</option><option value="ko">{t('language-ko')}</option></select></label>
    </header>
    <p className="connection" data-testid="connection" role="status">{t(connection)}</p>
    <section aria-label={t('controls')}>
      <dl>
        <div><dt>{t('date')}</dt><dd data-testid="date">{state?.date ?? t('waiting')}</dd></div>
        <div><dt>{t('hour')}</dt><dd data-testid="hour">{state?.hour ?? '—'}</dd></div>
        <div><dt>{t('tick')}</dt><dd data-testid="tick">{state?.tick ?? '—'}</dd></div>
        <div><dt>{t('speed')}</dt><dd data-testid="speed">{state?.speed ?? '—'}</dd></div>
      </dl>
      <div className="controls"><button data-testid="pause" disabled={!state || connection !== 'connected'} onClick={() => command({ type: 'Pause', paused: !state!.paused })}>{t(state?.paused ? 'resume' : 'pause')}</button>
        {[1, 2, 3, 4, 5].map(speed => <button key={speed} data-testid={`speed-${speed}`} disabled={!state || connection !== 'connected'} aria-pressed={state?.speed === speed} onClick={() => command({ type: 'SetSpeed', speed })}>{speed}</button>)}
      </div>
    </section>
    {notice && <p role="alert">{t(notice)}</p>}
  </main>;
}
