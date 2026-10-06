import { useState } from 'react';
import type { WorldView, LedgerView as WireLedger } from '../proto/protocol';
import { LedgerValue, type LedgerView } from './LedgerValue';
import { useLocalization } from '../i18n';
/** Field adapter only: server values, row order and ticks pass through unchanged. */
export function ledgerProps(ledger: WireLedger): LedgerView {
 return {base:ledger.base,final:ledger.final_value,tick:ledger.tick,entries:ledger.entries.map(e=>({id:e.id,labelKey:e.label_key,operationKey:e.operation_key,value:e.value,accumulated:e.accumulated,sourceKey:e.source_key}))};
}
/** WP-08 may supply selected IDs from map picking; local buttons also use real IDs. */
export function NationalPanels({world,nationId,stateId,onNationSelect,onStateSelect}: {
 world:WorldView;nationId?:number;stateId?:number;onNationSelect?:(id:number)=>void;onStateSelect?:(id:number)=>void;
}) {
 const {t}=useLocalization();const [localNation,setNation]=useState<number>();const [localState,setState]=useState<number>();
 const nation=world.nations.find(n=>n.id===(nationId??localNation))??world.nations[0];
 const state=world.states.find(s=>s.id===(stateId??localState))??world.states[0];
 const nationName=(id:number|null)=>t(world.nations.find(n=>n.id===id)?.name_key??'no-value');
 return <div className="national-panels" data-testid="national-panels" data-world-tick={world.tick}>
  <section aria-label={t('country-panel')} data-testid="country-panel">
   <h2>{t('country-panel')}</h2><nav>{world.nations.map(n=><button key={n.id} onClick={()=>{setNation(n.id);onNationSelect?.(n.id);}} aria-pressed={n.id===nation?.id}>{t(n.name_key)}</button>)}</nav>
   {nation&&<><h3>{t(nation.name_key)}</h3><dl>
    <div><dt>{t('nation-tag')}</dt><dd>{nation.tag}</dd></div>
    <div><dt>{t('capital-province')}</dt><dd>{nation.capital}</dd></div>
    <div><dt>{t('nation-government')}</dt><dd>{t(nation.government_key)}</dd></div>
   </dl><h4>{t('ideology-support')}</h4><dl>{nation.support.map(s=><div key={s.name_key}><dt>{t(s.name_key)}</dt><dd>{s.value}</dd></div>)}</dl></>}
  </section>
  <section aria-label={t('state-panel')} data-testid="state-panel">
   <h2>{t('state-panel')}</h2><nav>{world.states.map(s=><button key={s.id} onClick={()=>{setState(s.id);onStateSelect?.(s.id);}} aria-pressed={s.id===state?.id}>{t(s.name_key)}</button>)}</nav>
   {state&&<><h3>{t(state.name_key)}</h3><dl>
    <div><dt>{t('owner')}</dt><dd>{nationName(state.owner)}</dd></div>
    <div><dt>{t('population')}</dt><dd>{state.population}</dd></div>
    {[...state.resources,...state.buildings].map(s=><div key={s.name_key}><dt>{t(s.name_key)}</dt><dd>{s.value}</dd></div>)}
   </dl><LedgerValue labelKey="infrastructure" ledger={ledgerProps(state.infrastructure)}/>
   <table><thead><tr><th>{t('province-label')}</th><th>{t('owner')}</th><th>{t('controller')}</th></tr></thead>
    <tbody>{world.provinces.filter(p=>p.state===state.id).map(p=><tr key={p.id} data-testid={`province-${p.id}`}><td>{p.id}</td><td>{nationName(p.owner)}</td><td>{nationName(p.controller)}</td></tr>)}</tbody>
   </table></>}
  </section>
 </div>;
}
