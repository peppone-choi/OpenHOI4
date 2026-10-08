//! Explicit production inputs; numerical tuning is read from required defines.
use crate::economy::{quantity, unit_ratio};
use crate::{DefineValue, Defines, Number, national::LoadedNational, valid_id};
use oh_core::Fx;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub name_key: String,
    pub family: String,
    pub generation: u32,
    pub unit_cost: String,
    pub resources_per_item: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NationInput {
    pub allowed_models: BTreeSet<String>,
    pub stock: BTreeMap<String, i64>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tuning {
    pub initial_efficiency: Fx,
    pub efficiency_cap: Fx,
    pub daily_efficiency_growth: Fx,
    pub same_family_newer_retention: Fx,
    pub other_retention: Fx,
    pub stability_output_low: Fx,
    pub stability_output_high: Fx,
    pub inventory_count_limit: i64,
    pub lines_max: i64,
}
impl Tuning {
    pub fn from_defines(d: &Defines) -> Result<Self, String> {
        let number = |key: &str| match d.get(&format!("production.{key}")) {
            Some(DefineValue::Number(Number::Fixed(v))) => Ok(*v),
            Some(DefineValue::Number(Number::Integer(v))) => {
                Fx::checked_from_num(*v).ok_or_else(|| format!("production define overflow:{key}"))
            }
            _ => Err(format!("missing production define:{key}")),
        };
        let integer = |key: &str| match d.get(&format!("production.{key}")) {
            Some(DefineValue::Number(Number::Integer(v))) if *v > 0 => Ok(*v),
            _ => Err(format!("invalid production integer define:{key}")),
        };
        let t = Self {
            initial_efficiency: number("initial_efficiency")?,
            efficiency_cap: number("efficiency_cap")?,
            daily_efficiency_growth: number("daily_efficiency_growth")?,
            same_family_newer_retention: number("same_family_newer_retention")?,
            other_retention: number("other_retention")?,
            stability_output_low: number("stability_output_low")?,
            stability_output_high: number("stability_output_high")?,
            inventory_count_limit: integer("inventory_count_limit")?,
            lines_max: integer("lines_max")?,
        };
        if t.initial_efficiency <= Fx::ZERO
            || t.initial_efficiency > t.efficiency_cap
            || t.efficiency_cap > Fx::ONE
            || t.daily_efficiency_growth <= Fx::ZERO
            || t.daily_efficiency_growth > Fx::ONE
            || t.other_retention < Fx::ZERO
            || t.other_retention >= t.same_family_newer_retention
            || t.same_family_newer_retention >= Fx::ONE
            || t.stability_output_low < Fx::ZERO
            || t.stability_output_high < t.stability_output_low
        {
            return Err("invalid production tuning bounds".into());
        }
        Ok(t)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub models: BTreeMap<String, Model>,
    pub nations: BTreeMap<u16, NationInput>,
    #[serde(skip)]
    #[schemars(skip)]
    pub tuning: Option<Tuning>,
}
pub fn schema() -> schemars::Schema {
    schemars::schema_for!(Definition)
}
impl Definition {
    pub fn identity(&self) -> Result<u64, String> {
        oh_core::state_hash(&(&self.models, &self.nations, &self.tuning)).map_err(|e| e.to_string())
    }
    pub fn validate(&self, l: &LoadedNational) -> Result<(), String> {
        let t = self.tuning.as_ref().ok_or("missing production tuning")?;
        if l.economy.as_ref().is_none_or(|e| e.is_empty())
            || self.models.is_empty()
            || self.nations.len() != l.nations.len()
        {
            return Err("production requires real economy and explicit models/nations".into());
        }
        for (id, m) in &self.models {
            if !valid_id(id)
                || !valid_id(&m.family)
                || m.name_key.is_empty()
                || quantity(&m.unit_cost)? <= oh_core::Qty::ZERO
            {
                return Err("invalid production model/cost".into());
            }
            for (r, cost) in &m.resources_per_item {
                if !l.map.resource_ids().contains(r) || quantity(cost)? <= oh_core::Qty::ZERO {
                    return Err("invalid production resource reference/cost".into());
                }
            }
        }
        for n in &l.nations {
            let input = self.nations.get(&n.id).ok_or("production missing nation")?;
            if input
                .allowed_models
                .iter()
                .any(|id| !self.models.contains_key(id))
                || input.stock.keys().collect::<BTreeSet<_>>()
                    != input.allowed_models.iter().collect()
            {
                return Err("production stock/allowed-model reference mismatch".into());
            }
            if input
                .stock
                .values()
                .any(|v| *v < 0 || *v > t.inventory_count_limit)
            {
                return Err("production stock bounds".into());
            }
        }
        // Keep the shared decimal-domain validator explicit at the data boundary.
        unit_ratio(&t.initial_efficiency.to_string())?;
        Ok(())
    }
}
