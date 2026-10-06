import type { ReactNode } from 'react';
import { useLocalization, type Language } from '../i18n';

export function GameShell({ onLanguage, connection, children }: {
  onLanguage: (language: Language) => void;
  connection: string;
  children: ReactNode;
}) {
  const { language, t } = useLocalization();
  return <main data-openhoi-shell="">
    <header><div><h1>{t('title')}</h1><p>{t('subtitle')}</p></div>
      <label>{t('language')} <select value={language} onChange={event => onLanguage(event.target.value as Language)}>
        <option value="en">{t('language-en')}</option><option value="ko">{t('language-ko')}</option>
      </select></label>
    </header>
    <p className="connection" data-testid="connection" role="status">{t(connection)}</p>
    {children}
    <footer><a href="/fonts/OFL.txt">{t('font-license')}</a></footer>
  </main>;
}
