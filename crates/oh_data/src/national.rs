//! Validated M1 data boundary. Exact decimal strings never pass through floats.
use crate::{
    DataError, DataPack, ErrorKind, Fixed,
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
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String")]
    pub military: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String")]
    pub production: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String")]
    pub economy: Option<String>,
    pub start_date: String,
    pub map: String,
    pub nations: Vec<String>,
    pub ownership: BTreeMap<u16, String>,
    #[serde(default)]
    pub control_overrides: BTreeMap<u16, String>,
    #[serde(default)]
    pub state_modifiers: BTreeMap<u16, Vec<ModifierInput>>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String")]
    pub end_date: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "crate::trigger::Condition")]
    pub end_conditions: Option<crate::trigger::Condition>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String")]
    pub end_root: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "Vec<String>")]
    pub flag_keys: Option<Vec<String>>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "BTreeMap<String, Vec<String>>")]
    pub initial_flags: Option<BTreeMap<String, Vec<String>>>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "BTreeMap<String, crate::trigger::EffectProgram>")]
    pub effect_programs: Option<BTreeMap<String, crate::trigger::EffectProgram>>,
    #[serde(
        default,
        deserialize_with = "crate::trigger::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "crate::trigger::ScoreWeights")]
    pub score_weights: Option<crate::trigger::ScoreWeights>,
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
    pub military: Option<crate::military::Definition>,
    pub production: Option<crate::production::Definition>,
    pub economy: Option<crate::economy::Definition>,
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
    let message = message.into();
    let field = if message.contains("capital") {
        Some("capital")
    } else if message.contains("ideology") {
        Some("ideology_support")
    } else if message.contains("ownership") {
        Some("ownership")
    } else if message.contains("control") {
        Some("control_overrides")
    } else if message.contains("modifier") {
        Some("state_modifiers")
    } else if message.contains("tag") || message == "empty nations" {
        Some("nations")
    } else {
        None
    };
    if let Some(field) = field
        && let Ok(source) = read(path)
        && let Ok(doc) = source.document()
        && let Some(v) = doc.get(field)
    {
        return source.error(v.span().start, ErrorKind::Schema, message);
    }
    DataError {
        path: path.into(),
        line: 1,
        column: 1,
        kind: ErrorKind::Schema,
        message,
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
fn field_error(path: &Path, field: &str, message: &str) -> DataError {
    if let Ok(source) = read(path)
        && let Ok(doc) = source.document()
        && let Some(v) = doc.get(field)
    {
        return source.error(v.span().start, ErrorKind::Schema, message);
    }
    fail(path, message)
}
pub(crate) fn read_nation(path: &Path, tag: &str) -> Result<NationDefinition, DataError> {
    let nation: NationDefinition = document(path)?;
    if nation.tag != tag
        || tag.is_empty()
        || !tag
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    {
        return Err(field_error(
            path,
            "tag",
            "nation tag does not match scenario reference",
        ));
    }
    Ok(nation)
}
pub(crate) fn validate_nation_names(
    path: &Path,
    nation: &NationDefinition,
) -> Result<(), DataError> {
    if nation.name_key.is_empty() {
        return Err(field_error(path, "name_key", "empty nation name_key"));
    }
    if nation.government_key.is_empty() {
        return Err(field_error(
            path,
            "government_key",
            "empty nation government_key",
        ));
    }
    Ok(())
}
pub(crate) fn validate_nation_ideology(
    path: &Path,
    nation: &NationDefinition,
) -> Result<(), DataError> {
    let mut total = Fixed::ZERO;
    if nation.ideology_support.is_empty() {
        return Err(fail(path, "empty ideology support"));
    }
    for (key, value) in &nation.ideology_support {
        let ratio = value
            .parse::<Fixed>()
            .map_err(|_| fail(path, "invalid ideology ratio"))?;
        if key.is_empty() || !(Fixed::ZERO..=Fixed::ONE).contains(&ratio) {
            return Err(fail(path, "ideology outside 0..1"));
        }
        total = total
            .checked_add(ratio)
            .ok_or_else(|| fail(path, "ideology sum overflow"))?;
    }
    if total != Fixed::ONE {
        return Err(fail(path, "ideology sum must be one in Fx"));
    }
    Ok(())
}
pub fn load_scenario(root: &Path, id: &str) -> Result<LoadedNational, DataError> {
    let loaded = read_scenario(root, id)?;
    crate::check_present_catalogs(root)?;
    Ok(loaded)
}
pub(crate) fn read_scenario(root: &Path, id: &str) -> Result<LoadedNational, DataError> {
    if !valid_id(id) {
        return Err(fail(root, "invalid scenario ID"));
    }
    let path = root.join("scenarios").join(id).join("scenario.toml");
    let mut pack = crate::pack_validation::resolve_packs(&[root.to_owned()])?
        .remove(0)
        .data;
    // M1 scenario overlay preserves the shipped M0 skeleton and golden contract.
    pack.defines = crate::scenario_defines::load(root, id, &pack.defines, true)?;
    // Bound AST recursion before typed recursive deserialization.
    let raw: toml::Value = document(&path)?;
    let raw = serde_json::to_value(raw).map_err(|e| fail(&path, e.to_string()))?;
    if let Some(c) = raw.get("end_conditions") {
        crate::trigger::preflight_value(c).map_err(|e| field_error(&path, "end_conditions", &e))?;
    }
    if let Some(programs) = raw.get("effect_programs").and_then(|v| v.as_object()) {
        for (id, p) in programs {
            if let Some(es) = p.get("effects").and_then(|v| v.as_array()) {
                for e in es {
                    crate::trigger::preflight_value(e).map_err(|e| {
                        field_error(&path, "effect_programs", &format!("program {id}: {e}"))
                    })?;
                }
            }
        }
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
        let nation = read_nation(&np, tag)?;
        if !ids.insert(nation.id) {
            return Err(field_error(&np, "id", "duplicate nation ID"));
        }
        validate_nation_names(&np, &nation)?;
        if map.state_for_province(nation.capital).is_none() {
            return Err(fail(&np, "capital must reference land"));
        }
        validate_nation_ideology(&np, &nation)?;
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
    let economy = if let Some(id) = &scenario.economy {
        if !valid_id(id) {
            return Err(field_error(&path, "economy", "invalid economy ID"));
        }
        let economy_path = root.join("common/economy").join(format!("{id}.toml"));
        let raw: toml::Value = document(&economy_path)?;
        let raw = serde_json::to_value(raw).map_err(|e| fail(&economy_path, e.to_string()))?;
        if let Some(laws) = raw.get("laws").and_then(|v| v.as_object()) {
            for law in laws.values() {
                if let Some(c) = law.get("condition") {
                    crate::trigger::preflight_value(c)
                        .map_err(|e| field_error(&economy_path, "condition", &e))?;
                }
            }
        }
        Some(document::<crate::economy::Definition>(&economy_path)?)
    } else {
        None
    };
    let production = if let Some(id) = &scenario.production {
        if !valid_id(id) {
            return Err(field_error(&path, "production", "invalid production ID"));
        }
        let definition_path = root.join("common/production").join(format!("{id}.toml"));
        let mut d = document::<crate::production::Definition>(&definition_path)?;
        d.tuning = Some(
            crate::production::Tuning::from_defines(&pack.defines)
                .map_err(|e| field_error(&path, "production", &e))?,
        );
        Some(d)
    } else {
        None
    };
    let military = if let Some(id) = &scenario.military {
        if !valid_id(id) {
            return Err(field_error(&path, "military", "invalid military ID"));
        }
        let file = root.join("common/military").join(format!("{id}.toml"));
        Some(
            crate::military::parse(&read(&file)?.text)
                .map_err(|e| field_error(&file, "military", &e))?,
        )
    } else {
        None
    };
    let loaded = LoadedNational {
        military,
        production,
        economy,
        pack,
        scenario_id: id.into(),
        scenario,
        nations,
        map,
        visuals,
    };
    if let Some(e) = &loaded.economy {
        e.validate(&loaded).map_err(|message| {
            field_error(
                &root.join("common/economy").join(format!(
                    "{}.toml",
                    loaded.scenario.economy.as_ref().unwrap()
                )),
                "economy",
                &message,
            )
        })?;
    }
    if let Some(p) = &loaded.production {
        p.validate(&loaded)
            .map_err(|e| field_error(&path, "production", &e))?;
    }
    if let Some(m) = &loaded.military {
        m.validate(&loaded)
            .map_err(|e| field_error(&path, "military", &e))?;
    }
    crate::trigger::definition(&loaded).map_err(|message| {
        let candidate = message
            .split(|c: char| c == '.' || c == ':' || c.is_ascii_whitespace())
            .next()
            .unwrap_or("");
        let field = [
            "start_date",
            "end_date",
            "end_conditions",
            "end_root",
            "flag_keys",
            "initial_flags",
            "effect_programs",
            "score_weights",
        ]
        .into_iter()
        .find(|f| *f == candidate)
        .unwrap_or("end_conditions");
        field_error(&path, field, &format!("trigger {field}: {message}"))
    })?;
    Ok(loaded)
}

pub fn visuals_schema() -> Schema {
    schemars::schema_for!(VisualPalette)
}

/// Shared palette validation for standalone maps and scenario loading.
pub(crate) fn validate_visuals(path: &Path, map: &MapData) -> Result<(), DataError> {
    let visuals: VisualPalette = document(path)?;
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
        return Err(fail(path, "palette must cover every state and terrain"));
    }
    Ok(())
}
