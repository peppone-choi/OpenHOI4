import type {MilitaryLedgerDisplay} from '../militaryLedgerResponses';
import {useLocalization} from '../i18n';
import type {FixedValue} from '../proto/protocol';
function Qty({v}:{v:FixedValue}){return <output data-bits={v.bits} data-fractional-bits={v.fractional_bits}>{v.value}</output>;}
/** Displays Rust Qty authority verbatim; there is no Fx conversion or arithmetic. */
export function MilitaryQtyLedger({display}:{display:MilitaryLedgerDisplay}){
  const {t}=useLocalization();
  return <section className="military-qty-ledger" data-testid="military-qty-ledger" data-status={display.status} aria-label={t('military-ledger-title')}>
    <p role="status">{t(`military-ledger-${display.status}`)}</p>
    {display.reasonKey&&<p>{t(display.reasonKey)}</p>}
    {display.ledger&&<>
      <p>{t('military-ledger-declared')}</p>
      {display.ledger.fields.map(field=><details key={field.field} data-qty-ledger-field={field.field}>
        <summary>{t(`military-${field.field}`)}: <Qty v={field.value}/></summary>
        <div className="military-ledger-tooltip" role="group" aria-label={t(`military-${field.field}`)}>
          <p>{t('military-ledger-base')}: <Qty v={field.base}/></p>
          <ol>{field.entries.map(entry=><li key={entry.id} data-contribution-id={entry.id}>
            <code>{entry.component}</code> · {t(`military-ledger-role-${entry.role}`)} {entry.position+1} · {t('military-ledger-add')} <Qty v={entry.value}/> · {t('military-ledger-accumulated')} <Qty v={entry.accumulated}/>
          </li>)}</ol>
          <p>{t('military-ledger-final')}: <Qty v={field.value}/></p>
        </div>
      </details>)}
      <details><summary>{t('military-authority')}</summary><dl>
        <dt>{t('military-definitions-hash')}</dt><dd><code>{display.ledger.definitions_hash}</code></dd>
        <dt>{t('military-state-hash')}</dt><dd><code>{display.ledger.state_hash}</code></dd>
        <dt>{t('military-tick')}</dt><dd>{display.ledger.tick}</dd>
      </dl></details>
    </>}
  </section>;
}
