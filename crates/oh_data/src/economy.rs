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
    /// Exact empty declaration preserves presence; it does not create zero-valued nation producers.
    pub fn is_empty(&self) -> bool {
        self.buildings.is_empty()
            && self.laws.is_empty()
            && self.economy_category.is_empty()
            && self.conscription_category.is_empty()
            && self.nations.is_empty()
            && self.state_slots.is_empty()
    }
    pub fn validate(&self, loaded: &LoadedNational) -> Result<(), String> {
        if self.is_empty() {
            return Ok(());
        }
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
        // Static initial-state arithmetic validation. Runtime authority uses oh_sim::formula.
        for (id, n) in &self.nations {
            let nation = loaded
                .nations
                .iter()
                .find(|v| v.id == *id)
                .ok_or("missing initial nation")?;
            let mut population = 0i64;
            let mut resources = BTreeMap::new();
            let mut ic = 0i64;
            for state in loaded
                .map
                .states
                .iter()
                .filter(|s| loaded.scenario.ownership.get(&s.id) == Some(&nation.tag))
            {
                population = population
                    .checked_add(state.population)
                    .ok_or("initial population sum overflow")?;
                for (key, value) in &state.resources {
                    let entry = resources.entry(key).or_insert(0i64);
                    *entry = entry
                        .checked_add(*value)
                        .ok_or("initial resource sum overflow")?;
                }
                for (key, levels) in &state.buildings {
                    let unit = quantity(&self.buildings[key].ic_per_level)?;
                    let raw = i128::from(unit.to_bits()) * i128::from(*levels);
                    ic = ic
                        .checked_add(
                            i64::try_from(raw).map_err(|_| "initial building IC overflow")?,
                        )
                        .ok_or("initial IC sum overflow")?;
                }
            }
            for factor in std::iter::once(&n.ic_multiplier)
                .chain(n.laws.values().map(|id| &self.laws[id].ic_multiplier))
            {
                let raw = (i128::from(ic) * i128::from(ratio(factor)?.to_bits())) >> 32;
                ic = i64::try_from(raw).map_err(|_| "initial IC multiplier overflow")?;
            }
            let law = &self.laws[&n.laws[&self.economy_category]];
            let product = ratio(&law.instability_slope)?
                .checked_mul(Fx::ONE - unit_ratio(&n.stability)?)
                .ok_or("initial consumer slope overflow")?;
            let minimum = unit_ratio(&law.consumer_base)?
                .checked_add(product)
                .ok_or("initial consumer minimum overflow")?
                .min(Fx::ONE);
            if unit_ratio(&n.allocation[0])? < minimum {
                return Err("initial allocation below consumer minimum".into());
            }
            let conscription = &self.laws[&n.laws[&self.conscription_category]];
            i64::try_from(
                (i128::from(population)
                    * i128::from(unit_ratio(&conscription.conscription_ratio)?.to_bits()))
                    >> 32,
            )
            .map_err(|_| "initial manpower capacity overflow")?;
        }
        crate::trigger::economy_conditions(loaded)?;
        Ok(())
    }
}
