import { useId } from 'react';
import { useLocalization } from '../i18n';

/** Display adapter only. WP-09 maps Rust query results into these props.
 * Values are server-formatted decimal strings: no arithmetic, sorting or rounding here.
 * This is not a wire type or a second definition of the Rust protocol.
 */
export interface LedgerView {
  base: string;
  final: string;
  tick?: string;
  entries: readonly {
    id: string;
    labelKey: string;
    operationKey: string;
    value: string;
    accumulated: string;
    sourceKey: string | null;
  }[];
}

export function LedgerValue({ labelKey, ledger }: { labelKey: string; ledger: LedgerView }) {
  const { t } = useLocalization();
  const id = useId();
  return <details className="ledger-value">
    <summary aria-label={t('ledger-open', { label: t(labelKey) })} aria-controls={id}>
      <span>{t(labelKey)}</span> <strong>{ledger.final}</strong>
    </summary>
    <div id={id} className="ledger-tooltip">
      <h2>{t('ledger-title')}</h2>
      {ledger.tick != null && <p>{t('tick')}: {ledger.tick}</p>}
      <dl><div><dt>{t('ledger-base')}</dt><dd>{ledger.base}</dd></div><div><dt>{t('ledger-final')}</dt><dd>{ledger.final}</dd></div></dl>
      {ledger.entries.length ? <table>
        <thead><tr><th>{t('ledger-contribution')}</th><th>{t('ledger-operation')}</th><th>{t('ledger-value')}</th><th>{t('ledger-accumulated')}</th></tr></thead>
        <tbody>{ledger.entries.map(item => <tr key={item.id}>
          <td>{t(item.labelKey)}<small>{t('ledger-source', { source: t(item.sourceKey ?? 'ledger-base-source') })}</small></td>
          <td>{t(item.operationKey)}</td><td>{item.value}</td><td>{item.accumulated}</td>
        </tr>)}</tbody>
      </table> : <p>{t('ledger-empty')}</p>}
    </div>
  </details>;
}
