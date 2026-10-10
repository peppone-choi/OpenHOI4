//! Pure normal-template arithmetic. No live military state or equipment-derived combat.
use oh_core::Qty;
use oh_data::military_templates::{Definitions, Stats};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Normal {
    pub stats: Stats,
    pub manpower: i64,
    /// Whole item requirements, keyed by the explicitly authored model.
    pub equipment: BTreeMap<String, i64>,
    /// Raw Qty numerator remainder, denominator manpower. Informational, not a carry state.
    pub weighted_remainders: BTreeMap<String, i64>,
}

/// Diagnostic contributions to the seven additive normal-template Qty fields.
/// This is deliberately not serializable and is never part of a stored Normal,
/// military state, definition identity, canonical hash or save body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalQtyLedger {
    pub fields: Vec<NormalQtyLedgerField>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalQtyLedgerField {
    pub field: &'static str,
    pub base: Qty,
    pub value: Qty,
    pub entries: Vec<NormalQtyLedgerEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalQtyLedgerEntry {
    /// Stable field/role/position identity, including repeated component uses.
    pub id: String,
    pub role: &'static str,
    /// Zero-based position after the data loader's existing per-role sorting.
    pub position: u8,
    pub component: String,
    pub value: Qty,
    pub accumulated: Qty,
}

/// Quantize each declared input once; weighted numerators divide only once at the end.
/// Empty/zero-weight composition is an undefined arithmetic result, not an editor rule.
pub fn aggregate(definitions: &Definitions, template: &str) -> Result<Normal, String> {
    aggregate_with_ledger(definitions, template).map(|(normal, _)| normal)
}

/// Emit contributions from the actual checked sum loop. All historical error
/// strings and evaluation precedence remain intact; errors return no partial
/// result. Weighted averages and minimum speed retain their existing arithmetic
/// and do not acquire an Add/Mul modifier interpretation or a diagnostic carry.
pub fn aggregate_with_ledger(
    definitions: &Definitions,
    template: &str,
) -> Result<(Normal, NormalQtyLedger), String> {
    let template = definitions
        .templates()
        .get(template)
        .ok_or("InvalidReference: template ID")?;
    let components = template
        .combat
        .iter()
        .chain(&template.support)
        .map(|id| &definitions.components()[id])
        .collect::<Vec<_>>();
    let occurrences = template
        .combat
        .iter()
        .enumerate()
        .map(|(position, id)| ("combat", position, id))
        .chain(
            template
                .support
                .iter()
                .enumerate()
                .map(|(position, id)| ("support", position, id)),
        )
        .collect::<Vec<_>>();
    let manpower = components.iter().try_fold(0i64, |a, c| {
        a.checked_add(c.manpower).ok_or("Overflow: manpower")
    })?;
    let speed_kmh = components
        .iter()
        .map(|c| c.stats.speed_kmh)
        .min()
        .ok_or("UndefinedArithmetic: empty composition has no minimum speed")?;
    if manpower == 0 {
        return Err("UndefinedArithmetic: zero personnel weight".into());
    }
    let mut fields = Vec::with_capacity(7);
    let mut sum = |name: &'static str, field: fn(&Stats) -> Qty| -> Result<Qty, String> {
        let mut entries = Vec::with_capacity(components.len());
        let value = components
            .iter()
            .enumerate()
            .try_fold(Qty::ZERO, |a, (i, c)| {
                let value = field(&c.stats);
                let accumulated = a.checked_add(value).ok_or("Overflow: Qty sum")?;
                let (role, position, component) = occurrences[i];
                entries.push(NormalQtyLedgerEntry {
                    id: format!("{name}:{role}:{position}"),
                    role,
                    // Definitions enforce 12 combat / 4 support before aggregation.
                    position: u8::try_from(position).expect("validated composition bound"),
                    component: component.clone(),
                    value,
                    accumulated,
                });
                Ok::<Qty, String>(accumulated)
            })?;
        fields.push(NormalQtyLedgerField {
            field: name,
            base: Qty::ZERO,
            value,
            entries,
        });
        Ok(value)
    };
    let weighted = |field: fn(&Stats) -> Qty| -> Result<(Qty, i64), String> {
        let numerator = components.iter().try_fold(0i128, |a, c| {
            let term = i128::from(field(&c.stats).to_bits())
                .checked_mul(i128::from(c.manpower))
                .ok_or("Overflow: weighted product")?;
            a.checked_add(term).ok_or("Overflow: weighted numerator")
        })?;
        let denominator = i128::from(manpower);
        let quotient =
            i64::try_from(numerator / denominator).map_err(|_| "Overflow: weighted quotient")?;
        let remainder =
            i64::try_from(numerator % denominator).map_err(|_| "Overflow: weighted remainder")?;
        Ok((Qty::from_bits(quotient), remainder))
    };
    let (organization, org_remainder) = weighted(|s| s.organization)?;
    let (armor, armor_remainder) = weighted(|s| s.armor)?;
    let (piercing, piercing_remainder) = weighted(|s| s.piercing)?;
    let mut equipment: BTreeMap<String, i64> = BTreeMap::new();
    for c in &components {
        for e in c.equipment.values() {
            let count = equipment.entry(e.model.clone()).or_default();
            *count = count
                .checked_add(e.items)
                .ok_or("Overflow: equipment count")?;
        }
    }
    let normal = Normal {
        stats: Stats {
            strength: sum("strength", |s| s.strength)?,
            soft_fire: sum("soft_fire", |s| s.soft_fire)?,
            hard_fire: sum("hard_fire", |s| s.hard_fire)?,
            defense: sum("defense", |s| s.defense)?,
            breakthrough: sum("breakthrough", |s| s.breakthrough)?,
            frontage: sum("frontage", |s| s.frontage)?,
            supply_use: sum("supply_use", |s| s.supply_use)?,
            organization,
            armor,
            piercing,
            speed_kmh,
        },
        manpower,
        equipment,
        weighted_remainders: [
            ("organization".into(), org_remainder),
            ("armor".into(), armor_remainder),
            ("piercing".into(), piercing_remainder),
        ]
        .into(),
    };
    Ok((normal, NormalQtyLedger { fields }))
}
