//! Strict standalone resolved normal-component definitions, not pack/live authority.
use crate::national::LoadedNational;
use oh_core::{Fx, Qty};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::Path};
// Resource limits are format limits; 12/4 composition limits come from 01 §4.7.
pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_ENTRIES: usize = 4096;
pub const POLICY: &str = "wp16-resolved-normal-sum-personnel-weighted-min-speed-v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Combat,
    Support,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatsInput {
    strength: String,
    soft_fire: String,
    hard_fire: String,
    defense: String,
    breakthrough: String,
    frontage: String,
    supply_use: String,
    organization: String,
    armor: String,
    piercing: String,
    speed_kmh: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub strength: Qty,
    pub soft_fire: Qty,
    pub hard_fire: Qty,
    pub defense: Qty,
    pub breakthrough: Qty,
    pub frontage: Qty,
    pub supply_use: Qty,
    pub organization: Qty,
    pub armor: Qty,
    pub piercing: Qty,
    pub speed_kmh: Fx,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Equipment {
    pub family: String,
    pub model: String,
    pub items: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentInput {
    id: String,
    role: Role,
    manpower: i64,
    equipment: Vec<Equipment>,
    stats: StatsInput,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Component {
    pub role: Role,
    pub manpower: i64,
    pub equipment: BTreeMap<String, Equipment>,
    pub stats: Stats,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub family: String,
    pub model: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateInput {
    id: String,
    combat: Vec<String>,
    support: Vec<String>,
    bindings: Vec<Binding>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Template {
    pub combat: Vec<String>,
    pub support: Vec<String>,
    pub bindings: BTreeMap<String, String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u16,
    components: Vec<ComponentInput>,
    templates: Vec<TemplateInput>,
}
/// Construction only through the parser; no mutable access or deserialize backdoor.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Definitions {
    components: BTreeMap<String, Component>,
    templates: BTreeMap<String, Template>,
}
fn id(s: &str) -> Result<(), String> {
    if s.len() > 64 || !crate::valid_id(s) {
        Err("InvalidValue: bounded ID required".into())
    } else {
        Ok(())
    }
}
fn bounded(n: usize, max: usize) -> Result<(), String> {
    if n > max {
        Err("LimitExceeded: military definitions".into())
    } else {
        Ok(())
    }
}
fn decimal(s: &str) -> Result<(), String> {
    let mut pieces = s.split('.');
    let whole = pieces.next().unwrap_or("");
    let fraction = pieces.next();
    if s.len() > 64
        || whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.is_some_and(|f| f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()))
        || pieces.next().is_some()
    {
        return Err("InvalidValue: bounded unsigned decimal string required".into());
    }
    Ok(())
}
fn qty(s: &str) -> Result<Qty, String> {
    decimal(s)?;
    let value = s.parse::<Qty>().map_err(|_| "InvalidValue: Qty overflow")?;
    if value < Qty::ZERO {
        return Err("InvalidValue: negative Qty".into());
    }
    Ok(value)
}
fn stats(s: StatsInput) -> Result<Stats, String> {
    decimal(&s.speed_kmh)?;
    let speed_kmh = s
        .speed_kmh
        .parse::<Fx>()
        .map_err(|_| "InvalidValue: Fx overflow")?;
    if speed_kmh <= Fx::ZERO {
        return Err("InvalidValue: positive speed_kmh required".into());
    }
    Ok(Stats {
        strength: qty(&s.strength)?,
        soft_fire: qty(&s.soft_fire)?,
        hard_fire: qty(&s.hard_fire)?,
        defense: qty(&s.defense)?,
        breakthrough: qty(&s.breakthrough)?,
        frontage: qty(&s.frontage)?,
        supply_use: qty(&s.supply_use)?,
        organization: qty(&s.organization)?,
        armor: qty(&s.armor)?,
        piercing: qty(&s.piercing)?,
        speed_kmh,
    })
}
pub fn parse(text: &str) -> Result<Definitions, String> {
    bounded(text.len(), MAX_BYTES)?;
    let raw: Document = toml::from_str(text).map_err(|e| format!("InvalidDefinitions: {e}"))?;
    if raw.version != 1 {
        return Err("InvalidDefinitions: unsupported version".into());
    }
    bounded(raw.components.len(), MAX_ENTRIES)?;
    bounded(raw.templates.len(), MAX_ENTRIES)?;
    let mut components = BTreeMap::new();
    for c in raw.components {
        id(&c.id)?;
        if c.manpower < 0 {
            return Err("InvalidValue: negative manpower".into());
        }
        bounded(c.equipment.len(), MAX_ENTRIES)?;
        let mut equipment = BTreeMap::new();
        for e in c.equipment {
            id(&e.family)?;
            id(&e.model)?;
            if e.items < 0 {
                return Err("InvalidValue: negative equipment count".into());
            }
            if equipment.insert(e.family.clone(), e).is_some() {
                return Err("Duplicate: component equipment family".into());
            }
        }
        if components
            .insert(
                c.id,
                Component {
                    role: c.role,
                    manpower: c.manpower,
                    equipment,
                    stats: stats(c.stats)?,
                },
            )
            .is_some()
        {
            return Err("Duplicate: component ID".into());
        }
    }
    let mut templates = BTreeMap::new();
    for t in raw.templates {
        id(&t.id)?;
        bounded(t.combat.len(), 12)?;
        bounded(t.support.len(), 4)?;
        bounded(t.bindings.len(), MAX_ENTRIES)?;
        let mut bindings = BTreeMap::new();
        for b in t.bindings {
            id(&b.family)?;
            id(&b.model)?;
            if bindings.insert(b.family, b.model).is_some() {
                return Err("Duplicate: template binding family".into());
            }
        }
        for (entries, role) in [(&t.combat, Role::Combat), (&t.support, Role::Support)] {
            for name in entries {
                id(name)?;
                let c = components
                    .get(name)
                    .ok_or("InvalidReference: component ID")?;
                if c.role != role {
                    return Err("InvalidReference: component role".into());
                }
                for e in c.equipment.values() {
                    if bindings.get(&e.family) != Some(&e.model) {
                        return Err("BindingContextMismatch: component resolved stats require exact authored model".into());
                    }
                }
            }
        }
        // Occurrence order has no effect on these operators; sorting retains duplicates.
        let mut combat = t.combat;
        combat.sort();
        let mut support = t.support;
        support.sort();
        if templates
            .insert(
                t.id,
                Template {
                    combat,
                    support,
                    bindings,
                },
            )
            .is_some()
        {
            return Err("Duplicate: template ID".into());
        }
    }
    Ok(Definitions {
        components,
        templates,
    })
}
pub fn read_file(path: &Path) -> Result<Definitions, String> {
    if !std::fs::metadata(path)
        .map_err(|e| e.to_string())?
        .is_file()
    {
        return Err("InvalidDefinitions: regular file required".into());
    }
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("InvalidDefinitions: regular file required".into());
    }
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    bounded(bytes.len(), MAX_BYTES)?;
    parse(std::str::from_utf8(&bytes).map_err(|_| "InvalidDefinitions: UTF-8 required")?)
}
impl Definitions {
    pub fn components(&self) -> &BTreeMap<String, Component> {
        &self.components
    }
    pub fn templates(&self) -> &BTreeMap<String, Template> {
        &self.templates
    }
    pub fn identity(&self) -> Result<u64, String> {
        oh_core::state_hash(&(POLICY, &self.components, &self.templates)).map_err(|e| e.to_string())
    }
    /// Validate selected normal-template bindings against the actual loaded production registry.
    pub fn validate_bindings(
        &self,
        loaded: &LoadedNational,
        nation: u16,
        template: &str,
    ) -> Result<(), String> {
        let t = self
            .templates
            .get(template)
            .ok_or("InvalidReference: template ID")?;
        if !loaded.nations.iter().any(|n| n.id == nation) {
            return Err("InvalidReference: nation ID".into());
        }
        let production = loaded
            .production
            .as_ref()
            .ok_or("MissingContext: actual production required")?;
        let input = production
            .nations
            .get(&nation)
            .ok_or("MissingContext: production nation required")?;
        for (family, model) in &t.bindings {
            let m = production
                .models
                .get(model)
                .ok_or("InvalidReference: production model")?;
            if &m.family != family {
                return Err("InvalidReference: production model family".into());
            }
            if !input.allowed_models.contains(model) {
                return Err("DisallowedModel: selected nation".into());
            }
        }
        Ok(())
    }
}
