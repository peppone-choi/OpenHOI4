import {useState} from 'react';
import type {ProductionCommand,ProductionLineView,ProductionView,WorldView} from '../proto/protocol';
import {useLocalization} from '../i18n';
import {quantityBits} from '../productionValidation';
function LineControls({line,models,send,disabled}:{line:ProductionLineView,models:ProductionView['models'],send:(c:ProductionCommand)=>void,disabled:boolean}){
  const {t}=useLocalization();const [ic,setIC]=useState(line.requested_ic.value);const [model,setModel]=useState(line.model);const bits=quantityBits(ic);
  return <div data-production-line={line.id}><span>{t(models.find(m=>m.model===line.model)?.name_key??'unknown-message')} — {t('production-efficiency')}: {line.efficiency.value}; {t('production-carry')}: {line.carry.value}</span>
    <label>{t('production-ic')}<input aria-label={t('production-ic')} value={ic} onChange={e=>setIC(e.target.value)} inputMode="decimal"/></label>
    <button disabled={disabled||bits===null} onClick={()=>send({type:'SetIC',line:line.id,requested_ic_bits:bits!})}>{t('production-set-ic')}</button>
    <button disabled={disabled} onClick={()=>send({type:'Pause',line:line.id,paused:!line.paused})}>{t(line.paused?'production-resume':'production-pause')}</button>
    <label>{t('production-model')}<select aria-label={t('production-model')} value={model} onChange={e=>setModel(e.target.value)}>{models.map(m=><option key={m.model} value={m.model}>{t(m.name_key)}</option>)}</select></label>
    <button disabled={disabled} title={t('production-switch-note')} onClick={()=>send({type:'Switch',line:line.id,model})}>{t('production-switch')}</button>
    <button disabled={disabled} title={t('production-switch-note')} onClick={()=>send({type:'Cancel',line:line.id})}>{t('production-cancel')}</button>
  </div>;
}
export function ProductionPanel({view,world,controlledNation,onControl,onCommand,connected}:{view:ProductionView,world:WorldView|null,controlledNation:number|null,onControl:(n:number)=>void,onCommand:(c:ProductionCommand)=>void,connected:boolean}){
  const {t}=useLocalization();const [selected,setSelected]=useState(view.nations[0]?.nation??0);const [model,setModel]=useState(view.models[0]?.model??'');const [ic,setIC]=useState('1');
  const nation=controlledNation??selected;const input=view.nations.find(n=>n.nation===nation);const models=view.models.filter(m=>input?.allowed_models.includes(m.model));const bits=quantityBits(ic);const disabled=!connected||controlledNation===null;
  const day=view.day?.nations.find(n=>n.nation===nation);const selectedModel=models.find(m=>m.model===model);
  return <section aria-label={t('production-title')} data-testid="production-panel">
    <h2>{t('production-title')}</h2>
    {controlledNation===null&&<><label>{t('production-nation')}<select value={selected} onChange={e=>setSelected(Number(e.target.value))}>{view.nations.map(n=><option key={n.nation} value={n.nation}>{t(world?.nations.find(w=>w.id===n.nation)?.name_key??'unknown-message')}</option>)}</select></label><button disabled={!connected} onClick={()=>onControl(selected)}>{t('production-control')}</button></>}
    <p>{t('production-budget')}: {input?.military_ic.value}</p>
    <form onSubmit={e=>{e.preventDefault();if(bits!==null)onCommand({type:'Create',model,requested_ic_bits:bits});}}>
      <label>{t('production-model')}<select aria-label={t('production-new-model')} value={model} onChange={e=>setModel(e.target.value)}>{models.map(m=><option key={m.model} value={m.model}>{t(m.name_key)}</option>)}</select></label>
      <label>{t('production-ic')}<input aria-label={t('production-new-ic')} value={ic} inputMode="decimal" onChange={e=>setIC(e.target.value)}/></label>
      <button disabled={disabled||bits===null||!models.some(m=>m.model===model)}>{t('production-create')}</button>
    </form>
    {selectedModel&&<p data-testid="production-model-cost">{t('production-unit-cost')}: {selectedModel.unit_cost.value} {t('production-cost-unit')}; {selectedModel.resources.map(r=><span key={r.resource}> {t(`resource-${r.resource}`)}: {r.per_item.value} {t('production-resource-unit')}</span>)}</p>}
    <p>{t('production-switch-note')}</p>
    {view.lines.filter(l=>l.nation===nation).map(l=><LineControls key={l.id} line={l} models={models} send={onCommand} disabled={disabled}/>)}
    <h3>{t('production-stock')}</h3>{input?.stock.map(s=><p key={s.model} data-production-stock={s.model}>{t(view.models.find(m=>m.model===s.model)?.name_key??'unknown-message')}: <output>{s.available}</output></p>)}
    {day&&<><h3>{t('production-day')} {view.day!.tick}</h3><p>{t('production-unused-ic')}: {day.unused_ic.value}</p><table><thead><tr>{['production-model','production-effective-ic','production-fulfillment','production-output','production-carry'].map(k=><th key={k}>{t(k)}</th>)}</tr></thead><tbody>{day.lines.map(l=><tr key={l.starting.id} data-production-ledger={l.starting.id}><td>{t(view.models.find(m=>m.model===l.starting.model)?.name_key??'unknown-message')}</td><td>{l.effective_ic.value}</td><td>{l.fulfillment.value}</td><td>{l.output}</td><td>{l.ending_carry.value}</td></tr>)}</tbody></table><h3>{t('production-resources')}</h3><table><thead><tr>{['production-model','production-resource','production-required','production-reserved','production-debited'].map(k=><th key={k}>{t(k)}</th>)}</tr></thead><tbody>{day.lines.flatMap(l=>l.resources.map(r=><tr key={`${l.starting.id}:${r.resource}`} data-production-resource={`${l.starting.id}:${r.resource}`}><td>{t(view.models.find(m=>m.model===l.starting.model)?.name_key??'unknown-message')}</td><td>{t(`resource-${r.resource}`)}</td><td>{r.required.value}</td><td>{r.reserved.value}</td><td>{r.debited.value}</td></tr>))}</tbody></table></>}
  </section>;
}
