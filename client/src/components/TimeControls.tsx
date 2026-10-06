import type { TimeCommand, TimeState } from '../proto/protocol';
import { useLocalization } from '../i18n';

export function TimeControls({ state, connected, onCommand }: {
  state: TimeState | null;
  connected: boolean;
  onCommand: (command: TimeCommand) => void;
}) {
  const { t } = useLocalization();
  return <section aria-label={t('controls')}>
    <dl>
      <div><dt>{t('date')}</dt><dd data-testid="date">{state?.date ?? t('waiting')}</dd></div>
      <div><dt>{t('hour')}</dt><dd data-testid="hour">{state?.hour ?? t('no-value')}</dd></div>
      <div><dt>{t('tick')}</dt><dd data-testid="tick">{state?.tick ?? t('no-value')}</dd></div>
      <div><dt>{t('speed')}</dt><dd data-testid="speed">{state?.speed ?? t('no-value')}</dd></div>
    </dl>
    <div className="controls">
      <button data-testid="pause" disabled={!state || !connected} onClick={() => onCommand({ type: 'Pause', paused: !state!.paused })}>{t(state?.paused ? 'resume' : 'pause')}</button>
      {[1, 2, 3, 4, 5].map(speed => <button key={speed} data-testid={`speed-${speed}`} disabled={!state || !connected}
        aria-label={t('speed-choice', { speed })} aria-pressed={state?.speed === speed}
        onClick={() => onCommand({ type: 'SetSpeed', speed })}>{speed}</button>)}
    </div>
  </section>;
}
