//! Opt-in immutable authored military authority. No editor or combat policy.
use crate::{military_templates, national::LoadedNational};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArmyInput {
    pub id: u64,
    pub nation: u16,
    pub general: String,
    pub capacity: u32,
    pub priority: u16,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DivisionInput {
    pub id: u32,
    pub army: u64,
    pub template: String,
    pub province: u16,
    pub manpower: i64,
    pub equipment: BTreeMap<String, i64>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Background {
    pub committed: i64,
    pub reserved: i64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Input {
    version: u16,
    normal_templates: String,
    training_days: BTreeMap<String, u32>,
    background: BTreeMap<u16, Background>,
    armies: Vec<ArmyInput>,
    divisions: Vec<DivisionInput>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Definition {
    pub(crate) templates: military_templates::Definitions,
    pub training_days: BTreeMap<String, u32>,
    pub background: BTreeMap<u16, Background>,
    pub armies: BTreeMap<u64, ArmyInput>,
    pub divisions: BTreeMap<u32, DivisionInput>,
}
impl Definition {
    pub fn templates(&self) -> &military_templates::Definitions {
        &self.templates
    }
    pub fn identity(&self) -> Result<u64, String> {
        oh_core::state_hash(self).map_err(|e| e.to_string())
    }
    pub fn validate(&self, l: &LoadedNational) -> Result<(), String> {
        if l.economy.is_none() || l.production.is_none() {
            return Err("MilitaryRequiresEconomyAndProduction".into());
        }
        let ns = l.nations.iter().map(|n| n.id).collect::<BTreeSet<_>>();
        if self.background.keys().copied().collect::<BTreeSet<_>>() != ns {
            return Err("InvalidMilitaryBackgroundNations".into());
        }
        let mut generals = BTreeSet::new();
        for a in self.armies.values() {
            if !ns.contains(&a.nation)
                || a.capacity == 0
                || (a.general.len() > 64 || !crate::valid_id(&a.general))
                || !generals.insert((a.nation, a.general.clone()))
            {
                return Err("InvalidMilitaryArmy".into());
            }
        }
        for t in self.templates.templates().keys() {
            if !self.training_days.contains_key(t) {
                return Err("MissingTrainingDays".into());
            }
        }
        for (t, days) in &self.training_days {
            if *days == 0 || !self.templates.templates().contains_key(t) {
                return Err("InvalidTrainingDays".into());
            }
        }
        for d in self.divisions.values() {
            let a = self.armies.get(&d.army).ok_or("InvalidDivisionArmy")?;
            self.templates.validate_bindings(l, a.nation, &d.template)?;
            let province = l
                .map
                .provinces
                .iter()
                .find(|p| p.id == d.province)
                .ok_or("InvalidDivisionProvince")?;
            if province.kind != crate::map::ProvinceKind::Land {
                return Err("InvalidDivisionLand".into());
            }
        }
        for n in ns {
            for t in self.templates.templates().keys() {
                self.templates.validate_bindings(l, n, t)?
            }
        }
        Ok(())
    }
}
pub fn parse(text: &str) -> Result<Definition, String> {
    if text.len() > military_templates::MAX_BYTES {
        return Err("MilitaryLimitExceeded".into());
    }
    let i: Input = toml::from_str(text).map_err(|e| format!("InvalidMilitary: {e}"))?;
    if i.version != 1
        || i.armies.len() > military_templates::MAX_ENTRIES
        || i.divisions.len() > military_templates::MAX_ENTRIES
        || i.training_days.len() > military_templates::MAX_ENTRIES
    {
        return Err("MilitaryLimitOrVersion".into());
    }
    let mut armies = BTreeMap::new();
    for a in i.armies {
        if armies.insert(a.id, a).is_some() {
            return Err("DuplicateArmy".into());
        }
    }
    let mut divisions = BTreeMap::new();
    for d in i.divisions {
        if divisions.insert(d.id, d).is_some() {
            return Err("DuplicateDivision".into());
        }
    }
    if i.background
        .values()
        .any(|b| b.committed < 0 || b.reserved < 0)
    {
        return Err("InvalidMilitaryBackground".into());
    }
    Ok(Definition {
        templates: military_templates::parse(&i.normal_templates)?,
        training_days: i.training_days,
        background: i.background,
        armies,
        divisions,
    })
}

pub fn schema() -> schemars::Schema {
    schemars::schema_for!(Input)
}
