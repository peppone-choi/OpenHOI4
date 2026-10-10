//! Versioned, selected-template diagnostics; legacy military contracts stay unchanged.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryNormalLedgerEntry {
    pub id: String,
    pub role: String,
    pub position: u8,
    pub component: String,
    pub value: crate::FixedValue,
    pub accumulated: crate::FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryNormalLedgerField {
    pub field: String,
    pub base: crate::FixedValue,
    pub value: crate::FixedValue,
    pub entries: Vec<MilitaryNormalLedgerEntry>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryNormalLedgerView {
    pub definitions_hash: String,
    pub state_hash: String,
    pub tick: String,
    pub template: String,
    pub fields: Vec<MilitaryNormalLedgerField>,
}
impl MilitaryNormalLedgerView {
    pub fn from_sim(s: &oh_sim::Simulation, template: &str) -> Result<Self, &'static str> {
        let m = s.military().ok_or("unsupported-query")?;
        let world = s.world().ok_or("unsupported-query")?;
        let d = world.defs().military().ok_or("unsupported-query")?;
        if !d.training_days.contains_key(template) {
            return Err("unknown-template");
        }
        let (_, ledger) =
            oh_sim::military_templates::aggregate_with_ledger(d.templates(), template)
                .map_err(|_| "ledger-aggregation-failed")?;
        Ok(Self {
            definitions_hash: format!("{:016x}", m.definitions_hash()),
            state_hash: format!("{:016x}", s.state_hash().map_err(|_| "simulation-error")?),
            tick: s.snapshot().tick().to_string(),
            template: template.into(),
            fields: ledger
                .fields
                .into_iter()
                .map(|field| MilitaryNormalLedgerField {
                    field: field.field.into(),
                    base: crate::FixedValue::qty(field.base),
                    value: crate::FixedValue::qty(field.value),
                    entries: field
                        .entries
                        .into_iter()
                        .map(|entry| MilitaryNormalLedgerEntry {
                            id: entry.id,
                            role: entry.role.into(),
                            position: entry.position,
                            component: entry.component,
                            value: crate::FixedValue::qty(entry.value),
                            accumulated: crate::FixedValue::qty(entry.accumulated),
                        })
                        .collect(),
                })
                .collect(),
        })
    }
}
pub(crate) fn declarations() -> Vec<String> {
    let c = ts_rs::Config::default();
    vec![
        MilitaryNormalLedgerEntry::decl(&c),
        MilitaryNormalLedgerField::decl(&c),
        MilitaryNormalLedgerView::decl(&c),
    ]
}
