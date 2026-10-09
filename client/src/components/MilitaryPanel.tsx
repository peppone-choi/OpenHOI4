import type {ReactNode} from 'react';
import type {FixedValue,MilitaryEquipmentView,MilitaryNormalView,WorldView} from '../proto/protocol';
import type {MilitaryDisplay} from '../militaryResponses';
import {useLocalization} from '../i18n';

const normalFields=['strength','soft_fire','hard_fire','defense','breakthrough','frontage','supply_use','organization','armor','piercing','speed_kmh'] as const;
function Fixed({value}:{value:FixedValue}){
  return <span data-bits={value.bits} data-fractional-bits={value.fractional_bits}>{value.value}</span>;
}
function Fields({rows}:{rows:[string,ReactNode][]}){
  const {t}=useLocalization();
  return <dl>{rows.map(([key,value])=><div key={key}><dt>{t(`military-${key}`)}</dt><dd>{value??t('military-none')}</dd></div>)}</dl>;
}
function Equipment({items}:{items:MilitaryEquipmentView[]}){
  const {t}=useLocalization();
  return items.length?<ul>{items.map(item=><li key={item.model}><code>{item.model}</code>: <output>{item.count}</output></li>)}</ul>:<span>{t('military-empty')}</span>;
}
function Normal({normal}:{normal:MilitaryNormalView}){
  const {t}=useLocalization();
  return <details><summary>{t('military-normal')}</summary>
    <Fields rows={[
      ['manpower',normal.manpower],['equipment',<Equipment items={normal.equipment}/>],
      ...normalFields.map(field=>[field,<Fixed value={normal[field]}/>] as [string,ReactNode]),
    ]}/>
    <h4>{t('military-remainders')}</h4>
    {normal.weighted_remainders.length?<ul>{normal.weighted_remainders.map(row=><li key={row.field}><code>{row.field}</code>: {row.numerator_remainder}</li>)}</ul>:<p>{t('military-empty')}</p>}
  </details>;
}
function Records({name,children,empty}:{name:string;children:ReactNode;empty:boolean}){
  const {t}=useLocalization();
  return <section data-testid={`military-${name}`}><h3>{t(`military-${name}`)}</h3>{empty?<p>{t('military-empty')}</p>:<div className="military-records">{children}</div>}</section>;
}
export function MilitaryPanel({display,world,nationId,onNationSelect,onClose}:{display:MilitaryDisplay;world:WorldView|null;nationId:number|null;onNationSelect:(nation:number|null)=>void;onClose:()=>void}){
  const {t}=useLocalization(),view=display.view;
  const belongs=(row:{nation:number})=>nationId===null||row.nation===nationId;
  // A display filter only. It never changes Join, ownership or query authority.
  const nations=world?.nations.map(row=>row.id)??(view?[...new Set([...view.armies,...view.divisions,...view.jobs,...view.background].map(row=>row.nation))].sort((a,b)=>a-b):[]);
  const name=(nation:number)=>{const key=world?.nations.find(row=>row.id===nation)?.name_key;return key?`${t(key)} (${nation})`:String(nation);};
  const armies=view?.armies.filter(belongs)??[],divisions=view?.divisions.filter(belongs)??[],jobs=view?.jobs.filter(belongs)??[],background=view?.background.filter(belongs)??[],pending=view?.pending.filter(belongs)??[];
  return <section id="military-panel" className="military-panel" aria-label={t('military-title')} data-testid="military-panel" data-status={display.status}>
    <h2>{t('military-title')}</h2><button onClick={onClose}>{t('military-close')}</button>
    <p role="status">{t(`military-${display.status}`)}</p>
    {display.reasonKey&&<p>{t(display.reasonKey)}</p>}
    {view&&<>
      <label>{t('military-nation')}<select aria-label={t('military-nation')} value={nationId??''} onChange={event=>onNationSelect(event.target.value===''?null:Number(event.target.value))}>
        <option value="">{t('military-all-nations')}</option>
        {nationId!==null&&!nations.includes(nationId)&&<option value={nationId}>{name(nationId)}</option>}
        {nations.map(nation=><option key={nation} value={nation}>{name(nation)}</option>)}
      </select></label>
      <Records name="templates" empty={!view.templates.length}>{view.templates.map(row=><article key={row.template} data-military-template={row.template}>
        <Fields rows={[[ 'template',row.template],['training-days',row.training_days]]}/><Normal normal={row.normal}/>
      </article>)}</Records>
      <Records name="armies" empty={!armies.length}>{armies.map(row=><article key={row.id} data-military-army={row.id}>
        <Fields rows={[[ 'id',row.id],['nation',name(row.nation)],['general',row.general],['division-limit',row.division_limit],['priority',row.priority],['division-ids',row.divisions.length?row.divisions.join(', '):null]]}/>
      </article>)}</Records>
      <Records name="divisions" empty={!divisions.length}>{divisions.map(row=><article key={row.id} data-military-division={row.id}>
        <Fields rows={[[ 'id',row.id],['nation',name(row.nation)],['army',row.army],['province',row.province],['template',row.template],['manpower',row.manpower],['equipment',<Equipment items={row.equipment}/>]]}/><Normal normal={row.normal}/>
      </article>)}</Records>
      <Records name="jobs" empty={!jobs.length}>{jobs.map(row=><article key={row.id} data-military-job={row.id}>
        <Fields rows={[[ 'id',row.id],['nation',name(row.nation)],['template',row.template],['job-status',t(`military-status-${row.status}`)],['training-days',row.training_days],['progress-days',row.progress_days],['start-tick',row.start_tick],['reserved-manpower',row.reserved_manpower],['equipment',<Equipment items={row.equipment}/>],['division',row.division]]}/><Normal normal={row.normal}/>
      </article>)}</Records>
      <Records name="background" empty={!background.length}>{background.map(row=><article key={row.nation}><Fields rows={[[ 'nation',name(row.nation)],['committed',row.committed],['reserved-manpower',row.reserved]]}/></article>)}</Records>
      <details><summary>{t('military-authority')}</summary>
        <Fields rows={[[ 'state-hash',view.state_hash],['definitions-hash',view.definitions_hash],['next-job-id',view.next_job_id],['next-division-id',view.next_division_id]]}/>
        <Records name="pending" empty={!pending.length}>{pending.map(row=><article key={`${row.tick}:${row.nation}:${row.sequence}`}><Fields rows={[[ 'tick',row.tick],['nation',name(row.nation)],['sequence',row.sequence],['command',<code>{JSON.stringify(row.command)}</code>]]}/></article>)}</Records>
      </details>
    </>}
  </section>;
}
