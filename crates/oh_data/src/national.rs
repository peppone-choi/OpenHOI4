//! Validated M1 data boundary. Exact decimal strings never pass through floats.
use crate::{
    DataError, DataPack, ErrorKind, Fixed, load_pack,
    map::{MapData, load_map},
    read, valid_id,
};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NationDefinition {
    pub id: u16,
    pub tag: String,
    pub name_key: String,
    pub color: [u8; 3],
    pub capital: u16,
    pub government_key: String,
    pub ideology_support: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModifierInput {
    pub source: String,
    pub target_stat: String,
    pub operation: String,
    pub value: String,
    pub expires: Option<u64>,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub start_date: String,
    pub map: String,
    pub nations: Vec<String>,
    pub ownership: BTreeMap<u16, String>,
    #[serde(default)]
    pub control_overrides: BTreeMap<u16, String>,
    #[serde(default)]
    pub state_modifiers: BTreeMap<u16, Vec<ModifierInput>>,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VisualPalette {
    pub neutral: [u8; 3],
    pub terrain: BTreeMap<String, [u8; 3]>,
    pub state: BTreeMap<u16, [u8; 3]>,
}
#[derive(Debug, Clone)]
pub struct LoadedNational {
    pub pack: DataPack,
    pub scenario_id: String,
    pub scenario: Scenario,
    pub nations: Vec<NationDefinition>,
    pub map: MapData,
    pub visuals: VisualPalette,
}
pub fn nation_schema() -> Schema {
    schemars::schema_for!(NationDefinition)
}
pub fn scenario_schema() -> Schema {
    schemars::schema_for!(Scenario)
}
fn fail(path: &Path, message: impl Into<String>) -> DataError {
    DataError {
        path: path.into(),
        line: 1,
        column: 1,
        kind: ErrorKind::Schema,
        message: message.into(),
    }
}
fn document<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, DataError> {
    let source = read(path)?;
    source.document()?;
    toml::from_str(&source.text).map_err(|e| {
        source.error(
            e.span().map_or(0, |s| s.start),
            ErrorKind::Schema,
            e.message(),
        )
    })
}
pub fn load_scenario(root: &Path, id: &str) -> Result<LoadedNational, DataError> {
    if !valid_id(id) {
        return Err(fail(root, "invalid scenario ID"));
    }
    let path = root.join("scenarios").join(id).join("scenario.toml");
    let mut pack = load_pack(root)?;
    // M1 scenario overlay preserves the shipped M0 skeleton and golden contract.
    let defines = crate::parse_defines(&read(
        &root.join("scenarios").join(id).join("defines.toml"),
    )?)?;
    for (system, items) in defines.0 {
        pack.defines.0.entry(system).or_default().extend(items);
    }
    let scenario: Scenario = document(&path)?;
    let map = load_map(root, &scenario.map)?;
    let visual_path = root.join("maps").join(&scenario.map).join("visuals.toml");
    let visuals: VisualPalette = document(&visual_path)?;
    if visuals.state.len() != map.states.len()
        || map
            .states
            .iter()
            .any(|s| !visuals.state.contains_key(&s.id))
        || map
            .provinces
            .iter()
            .any(|p| !visuals.terrain.contains_key(&p.terrain))
    {
        return Err(fail(
            &visual_path,
            "palette must cover every state and terrain",
        ));
    }
    let mut nations = Vec::new();
    let mut tags = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for tag in &scenario.nations {
        if tag.is_empty()
            || !tag
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            || !tags.insert(tag.clone())
        {
            return Err(fail(&path, "invalid or duplicate tag"));
        }
        let np = root
            .join("scenarios")
            .join(id)
            .join("nations")
            .join(format!("{tag}.toml"));
        let nation: NationDefinition = document(&np)?;
        if &nation.tag != tag
            || !ids.insert(nation.id)
            || nation.name_key.is_empty()
            || nation.government_key.is_empty()
        {
            return Err(fail(&np, "invalid nation fields"));
        }
        if map.state_for_province(nation.capital).is_none() {
            return Err(fail(&np, "capital must reference land"));
        }
        let mut total = Fixed::ZERO;
        if nation.ideology_support.is_empty() {
            return Err(fail(&np, "empty ideology support"));
        }
        for (key, value) in &nation.ideology_support {
            let ratio = value
                .parse::<Fixed>()
                .map_err(|_| fail(&np, "invalid ideology ratio"))?;
            if key.is_empty() || !(Fixed::ZERO..=Fixed::ONE).contains(&ratio) {
                return Err(fail(&np, "ideology outside 0..1"));
            }
            total = total
                .checked_add(ratio)
                .ok_or_else(|| fail(&np, "ideology sum overflow"))?;
        }
        if total != Fixed::ONE {
            return Err(fail(&np, "ideology sum must be one in Fx"));
        }
        nations.push(nation);
    }
    if nations.is_empty() {
        return Err(fail(&path, "empty nations"));
    }
    nations.sort_by_key(|n| n.id);
    if scenario.ownership.len() != map.states.len() {
        return Err(fail(&path, "ownership must cover all states"));
    }
    for (state, tag) in &scenario.ownership {
        if !map.states.iter().any(|s| s.id == *state) || !tags.contains(tag) {
            return Err(fail(&path, "invalid ownership reference"));
        }
    }
    for (province, tag) in &scenario.control_overrides {
        if map.state_for_province(*province).is_none() || !tags.contains(tag) {
            return Err(fail(&path, "invalid control reference"));
        }
    }
    for (state, modifiers) in &scenario.state_modifiers {
        if !scenario.ownership.contains_key(state) {
            return Err(fail(&path, "invalid modifier state"));
        }
        let mut unique = BTreeSet::new();
        for m in modifiers {
            if m.source.is_empty()
                || m.target_stat != "infrastructure"
                || !["add", "mul"].contains(&m.operation.as_str())
                || !unique.insert((&m.source, &m.operation))
                || m.value.parse::<Fixed>().is_err()
            {
                return Err(fail(&path, "invalid infrastructure modifier"));
            }
        }
    }
    Ok(LoadedNational {
        pack,
        scenario_id: id.into(),
        scenario,
        nations,
        map,
        visuals,
    })
}

pub fn visuals_schema() -> Schema {
    schemars::schema_for!(VisualPalette)
}
