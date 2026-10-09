//! Pure normal-template arithmetic. No live military state or equipment-derived combat.
use oh_core::Qty;
use oh_data::military_templates::{Definitions, Stats};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Normal {
    pub stats: Stats,
    pub manpower: i64,
    /// Whole item requirements, keyed by the explicitly authored model.
    pub equipment: BTreeMap<String, i64>,
    /// Raw Qty numerator remainder, denominator manpower. Informational, not a carry state.
    pub weighted_remainders: BTreeMap<String, i64>,
}
/// Quantize each declared input once; weighted numerators divide only once at the end.
/// Empty/zero-weight composition is an undefined arithmetic result, not an editor rule.
pub fn aggregate(definitions: &Definitions, template: &str) -> Result<Normal, String> {
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
    let sum = |field: fn(&Stats) -> Qty| -> Result<Qty, String> {
        components.iter().try_fold(Qty::ZERO, |a, c| {
            a.checked_add(field(&c.stats))
                .ok_or_else(|| "Overflow: Qty sum".into())
        })
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
    Ok(Normal {
        stats: Stats {
            strength: sum(|s| s.strength)?,
            soft_fire: sum(|s| s.soft_fire)?,
            hard_fire: sum(|s| s.hard_fire)?,
            defense: sum(|s| s.defense)?,
            breakthrough: sum(|s| s.breakthrough)?,
            frontage: sum(|s| s.frontage)?,
            supply_use: sum(|s| s.supply_use)?,
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
    })
}
