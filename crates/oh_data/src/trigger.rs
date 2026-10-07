//! Declarative syntax and references. Capabilities are distinct from grammar.
use crate::{Fixed, national::LoadedNational, valid_id};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Technical AST limits from 02 §5.6, not gameplay coefficients.
pub const MAX_DEPTH: usize = 16;
pub const MAX_EFFECTS: usize = 1000;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Compare {
    Gte(String),
    Lte(String),
    Eq(String),
}
impl Compare {
    pub fn value(&self) -> Result<Fixed, String> {
        let (Self::Gte(v) | Self::Lte(v) | Self::Eq(v)) = self;
        v.parse().map_err(|_| "invalid exact Fx decimal".into())
    }
    pub fn test(&self, input: Fixed) -> Result<bool, String> {
        let value = self.value()?;
        Ok(match self {
            Self::Gte(_) => input >= value,
            Self::Lte(_) => input <= value,
            Self::Eq(_) => input == value,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdeologyArg {
    pub ideology: String,
    pub value: Compare,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    DateGte(String),
    AtWar(bool),
    AtWarWith(String),
    NationIs(String),
    Stability(Compare),
    Mobilization(Compare),
    PoliticalCapital(Compare),
    HasLaw(String),
    ControlsProvince(u16),
    OwnsState(u16),
    HasFlag(String),
    IdeologySupport(IdeologyArg),
    Chance(String),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeArg {
    pub target: String,
    pub effects: Vec<Effect>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IfArg {
    pub condition: Condition,
    pub then: Vec<Effect>,
    pub r#else: Vec<Effect>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildingArg {
    pub state: u16,
    pub building: String,
    pub levels: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransferArg {
    pub state: u16,
    pub nation: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EquipmentArg {
    pub equipment: String,
    pub amount: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Scope(ScopeArg),
    If(IfArg),
    AddStability(String),
    AddMobilization(String),
    AddPoliticalCapital(String),
    SetLaw(String),
    AddBuilding(BuildingArg),
    TransferState(TransferArg),
    DeclareWar(String),
    AddManpower(i64),
    AddEquipment(EquipmentArg),
    SetFlag(String),
    ClearFlag(String),
    EndScenario(String),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectProgram {
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "String")]
    pub root: Option<String>,
    pub effects: Vec<Effect>,
}
/// Missing is None; explicit null is rejected at every data boundary.
pub fn present<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScoreWeights {
    pub victory_points: String,
    pub industrial_capacity: String,
    pub survival: String,
    pub faction_victory: String,
}
impl ScoreWeights {
    pub fn values(&self) -> Result<[Fixed; 4], String> {
        let mut result = [Fixed::ZERO; 4];
        for (i, v) in [
            &self.victory_points,
            &self.industrial_capacity,
            &self.survival,
            &self.faction_victory,
        ]
        .into_iter()
        .enumerate()
        {
            result[i] = v
                .parse()
                .map_err(|_| "score_weights invalid Fx decimal".to_owned())?;
        }
        Ok(result)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Definition {
    pub end_date: Option<String>,
    pub end_conditions: Option<Condition>,
    pub end_root: Option<String>,
    pub flag_keys: Option<Vec<String>>,
    pub initial_flags: Option<BTreeMap<String, Vec<String>>>,
    pub effect_programs: Option<BTreeMap<String, EffectProgram>>,
    pub score_weights: Option<ScoreWeights>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Context {
    Global,
    Nation,
    State,
    Event,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    SelfScope,
    Nation(String),
    State(u16),
    CapitalState,
    FactionLeader,
    OwnerOfState(u16),
    EnemyOfWar,
}
pub fn target(s: &str) -> Result<Target, String> {
    Ok(match s {
        "self" => Target::SelfScope,
        "capital_state" => Target::CapitalState,
        "faction_leader" => Target::FactionLeader,
        "enemy_of_war" => Target::EnemyOfWar,
        _ => {
            let (kind, id) = s.split_once(':').ok_or("invalid scope target")?;
            match kind {
                "nation" if tag(id) => Target::Nation(id.into()),
                "state" | "owner_of_state"
                    if !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) =>
                {
                    let id = id.parse().map_err(|_| "scope ID outside u16")?;
                    if kind == "state" {
                        Target::State(id)
                    } else {
                        Target::OwnerOfState(id)
                    }
                }
                _ => return Err("invalid scope target".into()),
            }
        }
    })
}
fn tag(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}
pub fn date(s: &str) -> Result<(u32, u8, u8), String> {
    let parts: Vec<_> = s.split('-').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("invalid Gregorian date".into());
    }
    let y: u32 = parts[0].parse().map_err(|_| "date year")?;
    let m: u8 = parts[1].parse().map_err(|_| "date month")?;
    let d: u8 = parts[2].parse().map_err(|_| "date day")?;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400)) => 29,
        2 => 28,
        _ => 0,
    };
    if y == 0 || d == 0 || d > days {
        return Err("invalid Gregorian date".into());
    }
    Ok((y, m, d))
}
pub(crate) fn condition_shape(c: &Condition, depth: usize) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err("trigger depth exceeds 16".into());
    }
    match c {
        Condition::All(cs) | Condition::Any(cs) => {
            if cs.is_empty() {
                return Err("empty condition group".into());
            }
            for c in cs {
                condition_shape(c, depth + 1)?;
            }
        }
        Condition::Not(c) => condition_shape(c, depth + 1)?,
        Condition::DateGte(s) => {
            date(s)?;
        }
        Condition::NationIs(s) | Condition::AtWarWith(s) => {
            if !tag(s) {
                return Err("invalid nation tag".into());
            }
        }
        Condition::Stability(c) | Condition::Mobilization(c) => {
            if !(Fixed::ZERO..=Fixed::ONE).contains(&c.value()?) {
                return Err("ratio outside 0..1".into());
            }
        }
        Condition::PoliticalCapital(c) => {
            let (Compare::Gte(s) | Compare::Lte(s) | Compare::Eq(s)) = c;
            crate::economy::quantity(s)?;
        }
        Condition::IdeologySupport(a) => {
            if a.ideology.is_empty() || !(Fixed::ZERO..=Fixed::ONE).contains(&a.value.value()?) {
                return Err("invalid ideology comparison".into());
            }
        }
        Condition::Chance(s) => {
            let n: Fixed = s.parse().map_err(|_| "invalid chance")?;
            if !(Fixed::ZERO..=Fixed::ONE).contains(&n) {
                return Err("chance outside0..1".into());
            }
        }
        Condition::HasLaw(s) | Condition::HasFlag(s) if !valid_id(s) => {
            return Err("invalid registry ID".into());
        }
        _ => {}
    }
    Ok(())
}
fn effects_shape(es: &[Effect], depth: usize) -> Result<(), String> {
    for e in es {
        if depth > MAX_DEPTH {
            return Err("effect depth exceeds16".into());
        }
        match e {
            Effect::Scope(a) => {
                target(&a.target)?;
                effects_shape(&a.effects, depth + 1)?;
            }
            Effect::If(a) => {
                condition_shape(&a.condition, depth + 1)?;
                effects_shape(&a.then, depth + 1)?;
                effects_shape(&a.r#else, depth + 1)?;
            }
            Effect::AddPoliticalCapital(s) => {
                s.parse::<oh_core::Qty>()
                    .map_err(|_| "invalid exact effect Qty")?;
            }
            Effect::AddStability(s) | Effect::AddMobilization(s) => {
                s.parse::<Fixed>().map_err(|_| "invalid exact effect Fx")?;
            }
            Effect::DeclareWar(s) => {
                if !tag(s) {
                    return Err("invalid nation tag".into());
                }
            }
            Effect::AddBuilding(a) => {
                if !valid_id(&a.building) || a.levels <= 0 {
                    return Err("invalid building amount/ID".into());
                }
            }
            Effect::TransferState(a) => {
                if !tag(&a.nation) {
                    return Err("invalid transfer nation".into());
                }
            }
            Effect::AddEquipment(a) => {
                if !valid_id(&a.equipment) || a.amount <= 0 {
                    return Err("invalid equipment amount/ID".into());
                }
            }
            Effect::SetLaw(s)
            | Effect::SetFlag(s)
            | Effect::ClearFlag(s)
            | Effect::EndScenario(s)
                if !valid_id(s) =>
            {
                return Err("invalid effect ID".into());
            }
            _ => {}
        }
    }
    Ok(())
}
fn raw_depth(v: &serde_json::Value, depth: usize) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err("trigger depth exceeds16".into());
    }
    if let Some(o) = v.as_object() {
        if o.len() != 1 {
            return Err("AST node must have one key".into());
        }
        for (k, v) in o {
            match k.as_str() {
                "all" | "any" => {
                    for c in v.as_array().ok_or("condition group type")? {
                        raw_depth(c, depth + 1)?;
                    }
                }
                "not" => raw_depth(v, depth + 1)?,
                "scope" => {
                    if let Some(es) = v.get("effects").and_then(|v| v.as_array()) {
                        for e in es {
                            raw_depth(e, depth + 1)?;
                        }
                    }
                }
                "if" => {
                    if let Some(c) = v.get("condition") {
                        raw_depth(c, depth + 1)?;
                    }
                    for k in ["then", "else"] {
                        if let Some(es) = v.get(k).and_then(|v| v.as_array()) {
                            for e in es {
                                raw_depth(e, depth + 1)?;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
pub fn preflight_value(v: &serde_json::Value) -> Result<(), String> {
    raw_depth(v, 1)
}
pub fn parse_condition(s: &str) -> Result<Condition, String> {
    let value: serde_json::Value = serde_json::from_str(s).map_err(|e| e.to_string())?;
    raw_depth(&value, 1)?;
    let c = serde_json::from_str(s).map_err(|e| e.to_string())?;
    condition_shape(&c, 1)?;
    Ok(c)
}
pub fn parse_effects(s: &str) -> Result<Vec<Effect>, String> {
    let value: serde_json::Value = serde_json::from_str(s).map_err(|e| e.to_string())?;
    for e in value.as_array().ok_or("effects must be array")? {
        raw_depth(e, 1)?;
    }
    let es: Vec<Effect> = serde_json::from_str(s).map_err(|e| e.to_string())?;
    effects_shape(&es, 1)?;
    Ok(es)
}
fn nation_ref(l: &LoadedNational, s: &str) -> Result<(), String> {
    if l.nations.iter().any(|n| n.tag == s) {
        Ok(())
    } else {
        Err(format!("missing nation:{s}"))
    }
}
fn state_ref(l: &LoadedNational, s: u16) -> Result<(), String> {
    if l.map.states.iter().any(|n| n.id == s) {
        Ok(())
    } else {
        Err(format!("missing state:{s}"))
    }
}
fn flag_ref(d: &Definition, s: &str) -> Result<(), String> {
    if d.flag_keys
        .as_ref()
        .is_some_and(|ks| ks.iter().any(|k| k == s))
    {
        Ok(())
    } else {
        Err(format!("missing flag:{s}"))
    }
}
fn condition_refs(
    l: &LoadedNational,
    d: &Definition,
    c: &Condition,
    context: Context,
    scope_tag: Option<&str>,
) -> Result<(), String> {
    use Condition::*;
    match c {
        All(cs) | Any(cs) => {
            for c in cs {
                condition_refs(l, d, c, context, scope_tag)?;
            }
        }
        Not(c) => condition_refs(l, d, c, context, scope_tag)?,
        DateGte(_) => {}
        Chance(_) => {
            if context != Context::Event {
                return Err("chance requires event activation context".into());
            }
        }
        _ => {
            if context != Context::Nation {
                return Err("condition requires explicit nation root/context".into());
            }
            match c {
                NationIs(s) | AtWarWith(s) => nation_ref(l, s)?,
                OwnsState(s) => state_ref(l, *s)?,
                ControlsProvince(p) => {
                    if !l.map.provinces.iter().any(|v| v.id == *p) {
                        return Err(format!("missing province:{p}"));
                    }
                }
                HasFlag(s) => flag_ref(d, s)?,
                IdeologySupport(a) => {
                    if !l.nations.iter().any(|n| {
                        Some(n.tag.as_str()) == scope_tag
                            && n.ideology_support.contains_key(&a.ideology)
                    }) {
                        return Err(format!("missing ideology:{}", a.ideology));
                    }
                }
                HasLaw(s) => {
                    if !l.economy.as_ref().is_some_and(|d| d.laws.contains_key(s)) {
                        return Err(format!("missing law registry:{s}"));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
fn condition_capability(c: &Condition, economy: bool) -> Result<(), String> {
    match c {
        Condition::All(cs) | Condition::Any(cs) => {
            for c in cs {
                condition_capability(c, economy)?;
            }
        }
        Condition::Not(c) => condition_capability(c, economy)?,
        Condition::DateGte(_)
        | Condition::NationIs(_)
        | Condition::OwnsState(_)
        | Condition::ControlsProvince(_)
        | Condition::HasFlag(_)
        | Condition::IdeologySupport(_) => {}
        Condition::Stability(_)
        | Condition::Mobilization(_)
        | Condition::PoliticalCapital(_)
        | Condition::HasLaw(_)
            if economy => {}
        _ => return Err("host capability: unsupported condition producer".into()),
    }
    Ok(())
}
fn effects_refs(
    l: &LoadedNational,
    d: &Definition,
    es: &[Effect],
    context: Context,
    scope_tag: Option<&str>,
) -> Result<(), String> {
    for e in es {
        match e {
            Effect::Scope(a) => {
                let (next, tag) = match target(&a.target)? {
                    Target::SelfScope => (context, scope_tag.map(str::to_owned)),
                    Target::Nation(s) => {
                        nation_ref(l, &s)?;
                        (Context::Nation, Some(s))
                    }
                    Target::State(id) => {
                        state_ref(l, id)?;
                        (Context::State, None)
                    }
                    Target::OwnerOfState(id) => {
                        state_ref(l, id)?;
                        (Context::Nation, l.scenario.ownership.get(&id).cloned())
                    }
                    Target::CapitalState if context == Context::Nation => (Context::State, None),
                    _ => return Err("host capability: unsupported scope/context".into()),
                };
                effects_refs(l, d, &a.effects, next, tag.as_deref())?;
            }
            Effect::If(a) => {
                condition_refs(l, d, &a.condition, context, scope_tag)?;
                effects_refs(l, d, &a.then, context, scope_tag)?;
                effects_refs(l, d, &a.r#else, context, scope_tag)?;
            }
            Effect::SetFlag(s) | Effect::ClearFlag(s) => {
                if context != Context::Nation {
                    return Err("host capability: flags require nation scope".into());
                }
                flag_ref(d, s)?;
            }
            Effect::EndScenario(_) => {}
            Effect::AddBuilding(a) => {
                state_ref(l, a.state)?;
                if !l.map.building_ids().contains(&a.building) {
                    return Err("missing building ID".into());
                }
            }
            Effect::TransferState(a) => {
                state_ref(l, a.state)?;
                nation_ref(l, &a.nation)?;
            }
            Effect::SetLaw(s) => {
                if context != Context::Nation
                    || !l.economy.as_ref().is_some_and(|d| d.laws.contains_key(s))
                {
                    return Err("missing law registry/context".into());
                }
            }
            Effect::DeclareWar(s) => nation_ref(l, s)?,
            _ => {
                if context != Context::Nation {
                    return Err("effect requires nation context".into());
                }
            }
        }
    }
    Ok(())
}
fn effects_capability(es: &[Effect], economy: bool) -> Result<(), String> {
    for e in es {
        match e {
            Effect::Scope(a) => effects_capability(&a.effects, economy)?,
            Effect::If(a) => {
                condition_capability(&a.condition, economy)?;
                effects_capability(&a.then, economy)?;
                effects_capability(&a.r#else, economy)?;
            }
            Effect::SetFlag(_) | Effect::ClearFlag(_) | Effect::EndScenario(_) => {}
            Effect::AddStability(_)
            | Effect::AddMobilization(_)
            | Effect::AddPoliticalCapital(_)
            | Effect::SetLaw(_)
                if economy => {}
            _ => return Err("host capability: unsupported effect producer".into()),
        }
    }
    Ok(())
}
pub fn definition(l: &LoadedNational) -> Result<Option<Definition>, String> {
    let s = &l.scenario;
    if s.end_date.is_none()
        && s.end_conditions.is_none()
        && s.end_root.is_none()
        && s.flag_keys.is_none()
        && s.initial_flags.is_none()
        && s.effect_programs.is_none()
        && s.score_weights.is_none()
    {
        return Ok(None);
    }
    let d = Definition {
        end_date: s.end_date.clone(),
        end_conditions: s.end_conditions.clone(),
        end_root: s.end_root.clone(),
        flag_keys: s.flag_keys.clone(),
        initial_flags: s.initial_flags.clone(),
        effect_programs: s.effect_programs.clone(),
        score_weights: s.score_weights.clone(),
    };
    if let Some(end) = &d.end_date
        && date(end).map_err(|e| format!("end_date: {e}"))?
            < date(&s.start_date).map_err(|e| format!("start_date: {e}"))?
    {
        return Err("end_date precedes start_date".into());
    }
    if let Some(root) = &d.end_root {
        nation_ref(l, root).map_err(|e| format!("end_root: {e}"))?;
    }
    let mut keys = BTreeSet::new();
    if let Some(ks) = &d.flag_keys {
        for k in ks {
            if !valid_id(k) || !keys.insert(k) {
                return Err("flag_keys invalid/duplicate ID".into());
            }
        }
    }
    if let Some(fs) = &d.initial_flags {
        for (n, ks) in fs {
            nation_ref(l, n).map_err(|e| format!("initial_flags.{n}: {e}"))?;
            let mut seen = BTreeSet::new();
            for k in ks {
                flag_ref(&d, k).map_err(|e| format!("initial_flags.{n}: {e}"))?;
                if !seen.insert(k) {
                    return Err("initial_flags duplicate key".into());
                }
            }
        }
    }
    if let Some(c) = &d.end_conditions {
        condition_shape(c, 1).map_err(|e| format!("end_conditions: {e}"))?;
        condition_refs(
            l,
            &d,
            c,
            if d.end_root.is_some() {
                Context::Nation
            } else {
                Context::Global
            },
            d.end_root.as_deref(),
        )
        .map_err(|e| format!("end_conditions: {e}"))?;
    }
    if let Some(ps) = &d.effect_programs {
        for (id, p) in ps {
            if !valid_id(id) {
                return Err("effect_programs: invalid program ID".into());
            }
            if let Some(r) = &p.root {
                nation_ref(l, r).map_err(|e| format!("effect_programs.{id}.root: {e}"))?;
            }
            effects_shape(&p.effects, 1).map_err(|e| format!("effect_programs.{id}: {e}"))?;
            effects_refs(
                l,
                &d,
                &p.effects,
                if p.root.is_some() {
                    Context::Nation
                } else {
                    Context::Global
                },
                p.root.as_deref(),
            )
            .map_err(|e| format!("effect_programs.{id}: {e}"))?;
        }
    }
    if let Some(c) = &d.end_conditions {
        condition_capability(c, l.economy.is_some()).map_err(|e| format!("end_conditions: {e}"))?;
    }
    if let Some(ps) = &d.effect_programs {
        for (id, p) in ps {
            effects_capability(&p.effects, l.economy.is_some())
                .map_err(|e| format!("effect_programs.{id}: {e}"))?;
        }
    }
    if let Some(w) = &d.score_weights
        && w.values()?.iter().any(|v| *v != Fixed::ZERO)
    {
        return Err("score_weights: host capability: score input producer unavailable".into());
    }
    Ok(Some(d))
}

#[derive(Serialize)]
pub struct RegistryItem {
    pub key: &'static str,
    pub kind: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub argument_schema: serde_json::Value,
    pub simulation_supported: bool,
}
pub fn registry() -> Vec<RegistryItem> {
    let conditions = [
        ("date_gte", "\"2000-01-01\""),
        ("at_war", "true"),
        ("at_war_with", "\"STH\""),
        ("nation_is", "\"NTH\""),
        ("stability", "{\"gte\":\"0.5\"}"),
        ("mobilization", "{\"lte\":\"1\"}"),
        ("political_capital", "{\"eq\":\"10\"}"),
        ("has_law", "\"law_a\""),
        ("controls_province", "0"),
        ("owns_state", "10"),
        ("has_flag", "\"x\""),
        (
            "ideology_support",
            "{\"ideology\":\"democratic\",\"value\":{\"gte\":\"0.5\"}}",
        ),
        ("chance", "\"0.5\""),
    ];
    let effects = [
        ("add_stability", "\"0.1\""),
        ("add_mobilization", "\"-0.1\""),
        ("add_political_capital", "\"3\""),
        ("set_law", "\"law_a\""),
        (
            "add_building",
            "{\"state\":10,\"building\":\"industry\",\"levels\":1}",
        ),
        ("transfer_state", "{\"state\":10,\"nation\":\"STH\"}"),
        ("declare_war", "\"STH\""),
        ("add_manpower", "1"),
        (
            "add_equipment",
            "{\"equipment\":\"test_equipment\",\"amount\":1}",
        ),
        ("set_flag", "\"x\""),
        ("clear_flag", "\"x\""),
        ("end_scenario", "\"test_end\""),
    ];
    let cs = serde_json::to_value(schemars::schema_for!(Condition)).expect("schema");
    let es = serde_json::to_value(schemars::schema_for!(Effect)).expect("schema");
    conditions
        .into_iter()
        .map(|(key, example)| RegistryItem {
            key,
            kind: "condition",
            description: match key {
                "date_gte" => "Gregorian date at least argument",
                "has_flag" => "Read nation flag set",
                "nation_is" => "Compare scoped nation tag",
                "controls_province" => "Compare actual province controller",
                "owns_state" => "Compare actual state owner",
                "ideology_support" => "Compare actual nation ideology ratio",
                "at_war" => "Compare whether scoped nation is at war (future war producer)",
                "at_war_with" => {
                    "Compare war relation with referenced nation (future war producer)"
                }
                "stability" => "Compare nation stability ratio (future politics producer)",
                "mobilization" => "Compare nation mobilization ratio (future politics producer)",
                "political_capital" => "Compare political capital (future politics producer)",
                "has_law" => "Read active law ID (future politics producer)",
                "chance" => {
                    "Event activation probability; requires separate simulation RNG adapter"
                }
                _ => unreachable!("registry condition"),
            },
            example,
            argument_schema: argument_schema(&cs, key),
            simulation_supported: matches!(
                key,
                "date_gte"
                    | "has_flag"
                    | "nation_is"
                    | "controls_province"
                    | "owns_state"
                    | "ideology_support"
            ),
        })
        .chain(effects.into_iter().map(|(key, example)| RegistryItem {
            key,
            kind: "effect",
            description: match key {
                "set_flag" => "Idempotently insert nation flag",
                "clear_flag" => "Idempotently remove nation flag",
                "end_scenario" => "Stage explicit end source until transaction succeeds",
                "add_stability" => "Apply signed stability delta through future politics adapter",
                "add_mobilization" => {
                    "Apply signed mobilization delta through future politics adapter"
                }
                "add_political_capital" => {
                    "Apply signed political capital delta through future politics adapter"
                }
                "set_law" => "Set referenced active law through future politics adapter",
                "add_building" => {
                    "Add levels to referenced state building through future construction adapter"
                }
                "transfer_state" => "Change state ownership through future authority adapter",
                "declare_war" => {
                    "Declare war with referenced nation through future diplomacy adapter"
                }
                "add_manpower" => "Apply signed manpower delta through future manpower adapter",
                "add_equipment" => {
                    "Add referenced equipment quantity through future inventory adapter"
                }
                _ => unreachable!("registry effect"),
            },
            example,
            argument_schema: argument_schema(&es, key),
            simulation_supported: matches!(key, "set_flag" | "clear_flag" | "end_scenario"),
        }))
        .collect()
}

fn argument_schema(root: &serde_json::Value, key: &str) -> serde_json::Value {
    let mut argument = root["oneOf"]
        .as_array()
        .expect("enum schema")
        .iter()
        .find_map(|v| v["properties"].get(key).cloned())
        .expect("registered argument schema");
    if let Some(defs) = root.get("$defs") {
        argument["$defs"] = defs.clone();
    }
    let mut node = root.clone();
    node.as_object_mut().expect("schema object").remove("$defs");
    node.as_object_mut()
        .expect("schema object")
        .remove("$schema");
    argument["$defs"]["RegistryNode"] = node;
    fn references(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if key == "$ref" && value == "#" {
                        *value = serde_json::json!("#/$defs/RegistryNode");
                    } else {
                        references(value);
                    }
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    references(value);
                }
            }
            _ => {}
        }
    }
    references(&mut argument);
    argument
}

pub(crate) fn economy_conditions(l: &LoadedNational) -> Result<(), String> {
    let Some(e) = &l.economy else {
        return Ok(());
    };
    let d = Definition {
        end_date: None,
        end_conditions: None,
        end_root: None,
        flag_keys: l.scenario.flag_keys.clone(),
        initial_flags: l.scenario.initial_flags.clone(),
        effect_programs: None,
        score_weights: None,
    };
    for (id, law) in &e.laws {
        for nation in &l.nations {
            condition_refs(l, &d, &law.condition, Context::Nation, Some(&nation.tag))
                .map_err(|error| format!("laws.{id}.condition: {error}"))?;
        }
        condition_capability(&law.condition, true)?;
    }
    Ok(())
}
