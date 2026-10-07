import type { ReactNode } from 'react';
import type { PreviewText } from './locales';

export type HudSlots={nation?:ReactNode;date?:ReactNode;resources?:Partial<Record<'pc'|'stability'|'mobilization'|'industry'|'manpower'|'resources',ReactNode>>;unitDisplay?:ReactNode};
/** Slots accept future authority-provided display values; no game actions here. */
export function PreviewHud({t,slots={},children}:{t:PreviewText;slots?:HudSlots;children:ReactNode}){
 return <>
  <header className="preview-header">
   <div className="preview-brand"><span className="preview-emblem" aria-hidden="true">O<span>4</span></span><div><strong>{t('preview-title')}</strong><small>{t('preview-preview')}</small></div></div>
   <div className="preview-nation"><span className="preview-diamond" aria-hidden="true"/><div>{slots.nation??t('preview-country')}<small>{t('preview-unconnected')}</small></div></div>
   <div className="preview-resources">{(['pc','stability','mobilization','industry','manpower','resources'] as const).map(key=><div key={key}><small>{t(`preview-${key}`)}</small><strong>{slots.resources?.[key]??'—'}</strong></div>)}</div>
   <div className="preview-date"><small>{t('preview-date')}</small><strong>{slots.date??'—'}</strong><small>{t('preview-unconnected')}</small></div>
  </header>
  <nav className="preview-rail" aria-label={t('preview-unconnected')}>{(['politics','research','production','construction','diplomacy','organization','agenda','army'] as const).map((key,i)=><button key={key} disabled title={t('preview-unconnected')}><span aria-hidden="true" className={`rail-symbol symbol-${i}`}/><span>{t(`preview-${key}`)}</span></button>)}<small>{t('preview-unconnected')}</small></nav>
  {children}
  <aside className="preview-units"><span className="unit-outline" aria-hidden="true"/><div><strong>{t('preview-units')}</strong><small>{slots.unitDisplay??t('preview-unitPending')}</small></div><small>{t('preview-unconnected')}</small></aside>
 </>;
}
