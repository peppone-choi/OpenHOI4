import type { ReactNode } from 'react';
import type { PreviewText } from './locales';

export type HudSlots={nation?:ReactNode;date?:ReactNode;resources?:Partial<Record<'pc'|'stability'|'mobilization'|'industry'|'manpower'|'resources',ReactNode>>;unitDisplay?:ReactNode};
/** Slots accept future authority-provided display values; no game actions here. */
export function PreviewHud({t,slots={},children}:{t:PreviewText;slots?:HudSlots;children:ReactNode}){
 return <>
  <header className="preview-header">
   <div className="preview-brand"><span className="preview-emblem" aria-hidden="true">O<span>4</span></span><div><strong>{t('title')}</strong><small>{t('preview')}</small></div></div>
   <div className="preview-nation"><span className="preview-diamond" aria-hidden="true"/><div>{slots.nation??t('country')}<small>{t('unconnected')}</small></div></div>
   <div className="preview-resources">{(['pc','stability','mobilization','industry','manpower','resources'] as const).map(key=><div key={key}><small>{t(key)}</small><strong>{slots.resources?.[key]??'—'}</strong></div>)}</div>
   <div className="preview-date"><small>{t('date')}</small><strong>{slots.date??'—'}</strong><small>{t('unconnected')}</small></div>
  </header>
  <nav className="preview-rail" aria-label={t('unconnected')}>{(['politics','research','production','construction','diplomacy','organization','agenda','army'] as const).map((key,i)=><button key={key} disabled title={t('unconnected')}><span aria-hidden="true" className={`rail-symbol symbol-${i}`}/><span>{t(key)}</span></button>)}<small>{t('unconnected')}</small></nav>
  {children}
  <aside className="preview-units"><span className="unit-outline" aria-hidden="true"/><div><strong>{t('units')}</strong><small>{slots.unitDisplay??t('unitPending')}</small></div><small>{t('unconnected')}</small></aside>
 </>;
}
