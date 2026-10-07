//! Explicit economy inputs. Ratios are Fx, accumulated quantities are Qty.
use crate::{national::LoadedNational, trigger::Condition, valid_id};
use oh_core::{Fx, Qty};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureFactor {
    pub infrastructure: String,
    pub factor: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Building {
    pub name_key: String,
    pub costs: Vec<String>,
    pub slots: Vec<i64>,
    pub daily_cap: String,
    pub ic_per_level: String,
    pub infrastructure_factors: Vec<InfrastructureFactor>,
    // Explicit per-level infrastructure targets. None denotes an ordinary building.
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "Vec<String>")]
    pub infrastructure_levels: Option<Vec<String>>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Law {
    pub name_key: String,
    pub category: String,
    pub step: u32,
    pub cost: String,
    pub condition: Condition,
    pub ic_multiplier: String,
    pub conscription_ratio: String,
    pub consumer_base: String,
    pub instability_slope: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NationInput {
    pub laws: BTreeMap<String, String>,
    pub political_capital: String,
    pub political_capital_cap: String,
    pub political_capital_daily: String,
    pub stability: String,
    pub mobilization: String,
    pub allocation: [String; 4],
    pub ic_multiplier: String,
    pub committed: i64,
    pub reserved: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub buildings: BTreeMap<String, Building>,
    pub laws: BTreeMap<String, Law>,
    pub economy_category: String,
    pub conscription_category: String,
    pub nations: BTreeMap<u16, NationInput>,
    pub state_slots: BTreeMap<u16, i64>,
}
pub fn schema() -> schemars::Schema {
    schemars::schema_for!(Definition)
}
pub fn ratio(s: &str) -> Result<Fx, String> {
    let value = s
        .parse::<Fx>()
        .map_err(|_| format!("invalid Fx decimal:{s}"))?;
    if value < Fx::ZERO {
        return Err("negative Fx".into());
    }
    Ok(value)
}
pub fn unit_ratio(s: &str) -> Result<Fx, String> {
    let value = ratio(s)?;
    if value > Fx::ONE {
        return Err("ratio outside 0..1".into());
    }
    Ok(value)
}
pub fn quantity(s: &str) -> Result<Qty, String> {
    let value = s
        .parse::<Qty>()
        .map_err(|_| format!("invalid Qty decimal:{s}"))?;
    if value < Qty::ZERO {
        return Err("negative Qty".into());
    }
    Ok(value)
}
impl Definition {
    pub fn validate(&self, loaded: &LoadedNational) -> Result<(), String> {
        let mut categories = BTreeSet::new();
        let mut steps = BTreeSet::new();
        for (id, b) in &self.buildings {
            if !valid_id(id)
                || b.name_key.is_empty()
                || !loaded.map.building_ids().contains(id)
                || b.costs.is_empty()
                || b.costs.len() != b.slots.len()
            {
                return Err(format!("building invalid/reference:{id}"));
            }
            for cost in &b.costs {
                if quantity(cost)? <= Qty::ZERO {
                    return Err("building cost must be positive".into());
                }
            }
            if b.slots.iter().any(|v| *v < 0) {
                return Err("negative building slot".into());
            }
            quantity(&b.daily_cap)?;
            quantity(&b.ic_per_level)?;
            let mut levels = BTreeSet::new();
            if b.infrastructure_factors.is_empty() {
                return Err("missing infrastructure factors".into());
            }
            for entry in &b.infrastructure_factors {
                let level = ratio(&entry.infrastructure)?;
                ratio(&entry.factor)?;
                if !levels.insert(level.to_bits()) {
                    return Err("duplicate infrastructure factor".into());
                }
            }
            if let Some(targets) = &b.infrastructure_levels {
                if targets.len() != b.costs.len() {
                    return Err("infrastructure targets count".into());
                }
                for value in targets {
                    ratio(value)?;
                }
            }
        }
        for (id, l) in &self.laws {
            if !valid_id(id)
                || l.name_key.is_empty()
                || !valid_id(&l.category)
                || !steps.insert((&l.category, l.step))
            {
                return Err(format!("law invalid/duplicate step:{id}"));
            }
            categories.insert(l.category.clone());
            if quantity(&l.cost)? <= Qty::ZERO {
                return Err("law cost must be positive".into());
            }
            ratio(&l.ic_multiplier)?;
            unit_ratio(&l.conscription_ratio)?;
            unit_ratio(&l.consumer_base)?;
            ratio(&l.instability_slope)?;
            // Full AST references and host capabilities are checked after loading.
            crate::trigger::condition_shape(&l.condition, 1)?;
        }
        if !categories.contains(&self.economy_category)
            || !categories.contains(&self.conscription_category)
        {
            return Err("missing economic/conscription category".into());
        }
        if self.nations.keys().copied().collect::<BTreeSet<_>>()
            != loaded.nations.iter().map(|n| n.id).collect()
            || self.state_slots.keys().copied().collect::<BTreeSet<_>>()
                != loaded.map.states.iter().map(|s| s.id).collect()
        {
            return Err("economy must cover every nation/state exactly".into());
        }
        for n in self.nations.values() {
            if n.laws.keys().cloned().collect::<BTreeSet<_>>() != categories {
                return Err("missing selected law category".into());
            }
            for (category, id) in &n.laws {
                if self.laws.get(id).is_none_or(|l| &l.category != category) {
                    return Err("selected law reference/category".into());
                }
            }
            let pc = quantity(&n.political_capital)?;
            let cap = quantity(&n.political_capital_cap)?;
            quantity(&n.political_capital_daily)?;
            if pc > cap {
                return Err("initial PC exceeds cap".into());
            }
            unit_ratio(&n.stability)?;
            unit_ratio(&n.mobilization)?;
            ratio(&n.ic_multiplier)?;
            if n.committed < 0 || n.reserved < 0 || n.committed.checked_add(n.reserved).is_none() {
                return Err("invalid committed/reserved".into());
            }
            let allocation = n
                .allocation
                .iter()
                .map(|s| unit_ratio(s))
                .collect::<Result<Vec<_>, _>>()?;
            if allocation
                .iter()
                .map(|v| i128::from(v.to_bits()))
                .sum::<i128>()
                != i128::from(Fx::ONE.to_bits())
            {
                return Err("allocation exact sum must be one".into());
            }
        }
        for s in &loaded.map.states {
            if s.population < 0
                || s.resources.values().any(|v| *v < 0)
                || self.state_slots[&s.id] < 0
            {
                return Err("negative population/resource/slot".into());
            }
            let mut occupied = 0i64;
            for (id, level) in &s.buildings {
                let b = self
                    .buildings
                    .get(id)
                    .ok_or_else(|| format!("missing economy building:{id}"))?;
                let level = usize::try_from(*level).map_err(|_| "negative building level")?;
                if level > b.costs.len() {
                    return Err("building exceeds stage cap".into());
                }
                for slots in &b.slots[..level] {
                    occupied = occupied.checked_add(*slots).ok_or("slot overflow")?;
                }
            }
            if occupied > self.state_slots[&s.id] {
                return Err("initial slot cap exceeded".into());
            }
        }
        crate::trigger::economy_conditions(loaded)?;
        Ok(())
    }
}
