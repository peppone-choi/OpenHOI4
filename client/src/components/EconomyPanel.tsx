import type { ReactNode } from 'react';
import type { EconomyNationView, FixedValue, LawSelection, WorldView } from '../proto/protocol';
import type { EconomyDisplay } from '../economyResponses';
import { useLocalization } from '../i18n';

const roles = [
  {role:'consumer',label:'economy-consumer'},
  {role:'construction',label:'economy-construction'},
  {role:'military',label:'economy-military'},
  {role:'export',label:'economy-export'},
] as const;
const statuses = {
  loading:'economy-status-loading',ready:'economy-status-ready',stale:'economy-status-stale',
  unsupported:'economy-status-unsupported',disconnected:'economy-status-disconnected',
} satisfies Record<EconomyDisplay['status'],string>;
function Fixed({value}: {value:FixedValue}) {
  const {t} = useLocalization();
  return <abbr data-bits={value.bits} title={`${t('economy-raw-bits')}: ${value.bits}; ${t('economy-fractional-bits')}: ${value.fractional_bits}`}>{value.value}</abbr>;
}
function Fields({rows}: {rows:Array<[string,ReactNode]>}) {
  const {t} = useLocalization();
  return <dl>{rows.map(([key,value])=><div key={key}><dt>{t(key)}</dt><dd>{value}</dd></div>)}</dl>;
}
function Table({columns,rows}: {columns:string[];rows:ReactNode[][]}) {
  const {t} = useLocalization();
  return rows.length === 0 ? <p>{t('economy-empty')}</p> : <div className="economy-table"><table><thead><tr>{columns.map(key=><th key={key} scope="col">{t(key)}</th>)}</tr></thead><tbody>{rows.map((row,index)=><tr key={index}>{row.map((cell,column)=><td key={column}>{cell}</td>)}</tr>)}</tbody></table></div>;
}
function Laws({laws}: {laws:LawSelection[]}) {
  const {t} = useLocalization();
  const label = (key:string) => {const translated=t(key);return translated===t('unknown-message')?key:translated;};
  return <Table columns={['economy-category','economy-law','economy-name']} rows={laws.map(law=>[law.category,law.law,<span title={law.name_key}>{label(law.name_key)}</span>])}/>;
}
function Nation({view}: {view:EconomyNationView}) {
  const {t} = useLocalization(); const ledger=view.ledger, construction=view.construction;
  return <div data-testid="economy-nation-body" data-economy-nation={view.nation}>
    <Fields rows={[
      ['economy-political-capital',<Fixed value={view.political_capital}/>],['economy-stability',<Fixed value={view.stability}/>],['economy-mobilization',<Fixed value={view.mobilization}/>],
      ['economy-capacity',view.capacity],['economy-committed',view.committed],['economy-reserved',view.reserved],['economy-available',view.available],['economy-overcommitted',view.overcommitted],
    ]}/>
    <h3>{t('economy-laws')}</h3><Laws laws={view.laws}/>
    <h3>{t('economy-industry')}</h3>
    <Fields rows={[
      ['economy-ledger-tick',ledger.tick],['economy-total-ic',<Fixed value={ledger.total_ic}/>],['economy-minimum',<Fixed value={ledger.minimum}/>],['economy-consumer-residual',<Fixed value={ledger.consumer_residual}/>],
      ['economy-capacity',ledger.capacity],['economy-population',ledger.population],['economy-stability',<Fixed value={ledger.stability}/>],
    ]}/>
    <div className="economy-table"><table data-testid="economy-allocation"><thead><tr>{['economy-role','economy-ratio','economy-ledger-ratio','economy-allocation'].map(key=><th key={key}>{t(key)}</th>)}</tr></thead>
      <tbody>{roles.map(({role,label},index)=><tr key={role} data-allocation={role}><th scope="row">{t(label)}</th><td><Fixed value={view.ratios[index]}/></td><td><Fixed value={ledger.ratios[index]}/></td><td><Fixed value={ledger.allocation[index]}/></td></tr>)}</tbody></table></div>
    <h4>{t('economy-resources')}</h4><Table columns={['economy-resource','economy-flow']} rows={ledger.resources.map(resource=>[resource.resource,resource.value])}/>
    <details><summary>{t('economy-industry-detail')}</summary>
      <h4>{t('economy-laws')}</h4><Laws laws={ledger.laws}/>
      <h4>{t('economy-contributions')}</h4><Table columns={['economy-state','economy-building','economy-levels','economy-unit-ic','economy-value']} rows={ledger.contributions.map(entry=>[entry.state,entry.building,entry.levels,<Fixed value={entry.unit_ic}/>,<Fixed value={entry.value}/>])}/>
      <h4>{t('economy-multipliers')}</h4><Table columns={['economy-source','economy-factor','economy-applied']} rows={ledger.multipliers.map(entry=>[entry.source,<Fixed value={entry.factor}/>,<Fixed value={entry.applied}/>])}/>
      <h4>{t('economy-state-flows')}</h4>{ledger.state_flows.length===0?<p>{t('economy-empty')}</p>:ledger.state_flows.map(flow=><div key={flow.state}><Fields rows={[['economy-state',flow.state],['economy-population',flow.population]]}/><Table columns={['economy-resource','economy-flow']} rows={flow.resources.map(resource=>[resource.resource,resource.value])}/></div>)}
    </details>
    <h3>{t('economy-projects')}</h3><Table columns={['economy-project','economy-state','economy-building','economy-target','economy-progress','economy-dormancy']} rows={view.projects.map(project=>[project.id,project.state,project.building,project.target,<Fixed value={project.progress}/>,project.dormancy?t(project.dormancy):t('economy-none')])}/>
    <details data-testid="economy-construction"><summary>{t('economy-construction-detail')}</summary>
      <Fields rows={[['economy-ledger-tick',construction.tick],['economy-budget',<Fixed value={construction.budget}/>],['economy-unused',<Fixed value={construction.unused}/>]]}/>
      <Table columns={['economy-project','economy-state','economy-owner','economy-building','economy-target','economy-starting-progress','economy-cost','economy-daily-cap','economy-infrastructure','economy-factor-evaluated','economy-factor','economy-consumed','economy-applied','economy-discarded','economy-completed','economy-dormancy']} rows={construction.entries.map(entry=>[entry.project,entry.state,entry.owner,entry.building,entry.target,<Fixed value={entry.starting_progress}/>,<Fixed value={entry.cost}/>,<Fixed value={entry.daily_cap}/>,<Fixed value={entry.infrastructure}/>,t(entry.factor_evaluated?'economy-yes':'economy-no'),<Fixed value={entry.factor}/>,<Fixed value={entry.consumed}/>,<Fixed value={entry.applied}/>,<Fixed value={entry.discarded}/>,t(entry.completed?'economy-yes':'economy-no'),entry.dormancy?t(entry.dormancy):t('economy-none')])}/>
    </details>
  </div>;
}

export function EconomyPanel({display,world,nationId,onNationSelect}: {display:EconomyDisplay;world:WorldView|null;nationId:number|null;onNationSelect:(id:number)=>void}) {
  const {t} = useLocalization(); const view=display.view;
  const selected=nationId===null?view?.nations[0]:view?.nations.find(nation=>nation.nation===nationId);
  const nationName=(id:number)=>{const key=world?.nations.find(nation=>nation.id===id)?.name_key;const name=key?t(key):null;return name&&name!==t('unknown-message')?`${name} (${id})`:String(id);};
  return <section className="economy-panel" aria-label={t('economy-title')} data-testid="economy-panel" data-status={display.status}>
    <h2>{t('economy-title')}</h2>
    <p role="status">{t(statuses[display.status])}</p>
    {display.reasonKey&&<p>{t(display.reasonKey)}</p>}
    {view&&<>
      {view.nations.length>0&&<label>{t('economy-nation')}<select aria-label={t('economy-nation')} value={selected?.nation??''} onChange={event=>{const nation=view.nations.find(row=>String(row.nation)===event.target.value);if(nation)onNationSelect(nation.nation);}}>{!selected&&<option value="">{t('economy-none')}</option>}{view.nations.map(nation=><option key={nation.nation} value={nation.nation}>{nationName(nation.nation)}</option>)}</select></label>}
      {selected?<Nation view={selected}/>:<p>{t('economy-empty')}</p>}
      <details><summary>{t('economy-authority-detail')}</summary><Fields rows={[['economy-state-hash',view.state_hash],['economy-definitions-hash',view.definitions_hash]]}/>
        <h3>{t('economy-pending')}</h3><Table columns={['economy-tick','economy-nation','economy-sequence','economy-command']} rows={view.pending.map(pending=>[pending.tick,nationName(pending.nation),pending.sequence,<code>{JSON.stringify(pending.command)}</code>])}/>
        <h3>{t('economy-industrial-scores')}</h3>{view.industrial_scores===null?<p>{t('economy-scores-unavailable')}</p>:<Table columns={['economy-nation','economy-tick','economy-input-tick','economy-weight','economy-input','economy-term']} rows={view.industrial_scores.map(score=>[nationName(score.nation),score.tick,score.input_tick,<Fixed value={score.weight}/>,<Fixed value={score.input}/>,<Fixed value={score.term}/>])}/>}
      </details>
    </>}
  </section>;
}
