//! Strict, bounded EXTERNAL diagnostic inputs. Never registered pack/save authority.
use crate::{
    map::{EdgeKind, MapData, ProvinceKind},
    national::{LoadedNational, NationDefinition},
};
use oh_core::{Fx, Qty};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};

// Format/resource bounds, not game coefficients. The entire document is bounded before parsing.
pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_NATIONS: usize = 1024;
pub const MAX_ENTRIES: usize = 4096;
pub const MAX_METADATA: usize = 16384;
const MAX_DECIMAL_BYTES: usize = 64;
pub const POLICY: &str = "wp19-fixed-feeders-shared-undirected-capacity-distance-decay-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Capital,
    Hub,
    Port,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInput {
    pub id: u64,
    pub kind: SourceKind,
    pub province: u16,
    pub capacity: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RailInput {
    pub a: u16,
    pub b: u16,
    pub level: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InfrastructureInput {
    level: String,
    factor: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapacityInput {
    level: u32,
    capacity: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TuningInput {
    reference_speed_kmh: String,
    decay_per_reference_hour: String,
    infrastructure: Vec<InfrastructureInput>,
    rail_capacity: Vec<CapacityInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NationInput {
    id: u16,
    sources: Vec<SourceInput>,
    rails: Vec<RailInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u16,
    tuning: TuningInput,
    nations: Vec<NationInput>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Tuning {
    pub reference_speed_kmh: Fx,
    pub decay_per_reference_hour: Fx,
    pub infrastructure: BTreeMap<Fx, Fx>,
    pub rail_capacity: BTreeMap<u32, Qty>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Source {
    pub id: u64,
    pub kind: SourceKind,
    pub province: u16,
    pub capacity: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Nation {
    pub sources: Vec<Source>,
    pub rails: Vec<RailInput>,
}
/// Only the validated parser constructs this immutable normalized document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Config {
    tuning: Tuning,
    nations: BTreeMap<u16, Nation>,
}
fn decimal(s: &str) -> Result<(), String> {
    let mut pieces = s.split('.');
    let whole = pieces.next().unwrap_or("");
    let fraction = pieces.next();
    if s.len() > MAX_DECIMAL_BYTES
        || whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.is_some_and(|f| f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()))
        || pieces.next().is_some()
    {
        return Err("InvalidValue: expected bounded unsigned decimal string".into());
    }
    Ok(())
}
fn fx(s: &str, positive: bool) -> Result<Fx, String> {
    decimal(s)?;
    let v = s.parse::<Fx>().map_err(|_| "InvalidValue: Fx overflow")?;
    if v < Fx::ZERO || (positive && v == Fx::ZERO) {
        return Err("InvalidValue: Fx must be positive (or nonnegative where specified)".into());
    }
    Ok(v)
}
fn qty(s: &str) -> Result<Qty, String> {
    decimal(s)?;
    let v = s.parse::<Qty>().map_err(|_| "InvalidValue: Qty overflow")?;
    if v <= Qty::ZERO {
        return Err("InvalidValue: capacity must be positive Qty".into());
    }
    Ok(v)
}
fn bounded(n: usize, max: usize, field: &str) -> Result<(), String> {
    if n > max {
        Err(format!("LimitExceeded: {field}"))
    } else {
        Ok(())
    }
}
/// Exact decimal strings parse straight into checked fixed point, never floats.
pub fn parse(text: &str) -> Result<Config, String> {
    bounded(text.len(), MAX_BYTES, "config bytes")?;
    let raw: Document = toml::from_str(text).map_err(|e| format!("InvalidConfig: {e}"))?;
    normalize(raw)
}
/// Bounded read even when metadata size is stale or the file grows during reading.
pub fn read_file(path: &Path) -> Result<Config, String> {
    if !std::fs::metadata(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .is_file()
    {
        return Err("InvalidConfig: expected regular file".into());
    }
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("InvalidConfig: expected regular file".into());
    }
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    bounded(bytes.len(), MAX_BYTES, "config bytes")?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "InvalidConfig: expected UTF-8")?;
    parse(text).map_err(|e| format!("{}: {e}", path.display()))
}
fn normalize(raw: Document) -> Result<Config, String> {
    if raw.version != 1 {
        return Err("InvalidConfig: unsupported version".into());
    }
    bounded(raw.nations.len(), MAX_NATIONS, "nations")?;
    bounded(
        raw.tuning.infrastructure.len(),
        MAX_ENTRIES,
        "infrastructure",
    )?;
    bounded(raw.tuning.rail_capacity.len(), MAX_ENTRIES, "rail_capacity")?;
    if raw.nations.is_empty() || raw.tuning.infrastructure.is_empty() {
        return Err("MissingContext: explicit nations and infrastructure required".into());
    }
    let mut tuning = Tuning {
        reference_speed_kmh: fx(&raw.tuning.reference_speed_kmh, true)?,
        decay_per_reference_hour: fx(&raw.tuning.decay_per_reference_hour, false)?,
        infrastructure: BTreeMap::new(),
        rail_capacity: BTreeMap::new(),
    };
    for i in raw.tuning.infrastructure {
        let level = fx(&i.level, false)?;
        let factor = fx(&i.factor, true)?;
        if tuning.infrastructure.contains_key(&level) {
            return Err("Duplicate: normalized infrastructure level".into());
        }
        tuning.infrastructure.insert(level, factor);
    }
    for c in raw.tuning.rail_capacity {
        if c.level == 0 {
            return Err("InvalidValue: rail capacity level must be positive".into());
        }
        let capacity = qty(&c.capacity)?;
        if tuning.rail_capacity.contains_key(&c.level) {
            return Err("Duplicate: rail capacity level".into());
        }
        tuning.rail_capacity.insert(c.level, capacity);
    }
    let mut nations = BTreeMap::new();
    let mut total = 0usize;
    for n in raw.nations {
        if nations.contains_key(&n.id) {
            return Err("Duplicate: nation ID".into());
        }
        bounded(n.sources.len(), MAX_ENTRIES, "sources")?;
        bounded(n.rails.len(), MAX_ENTRIES, "rails")?;
        total = total
            .checked_add(n.sources.len())
            .and_then(|v| v.checked_add(n.rails.len()))
            .ok_or("LimitExceeded: metadata")?;
        bounded(total, MAX_METADATA, "total metadata")?;
        let mut ids = BTreeSet::new();
        let mut placements = BTreeSet::new();
        let mut edges = BTreeSet::new();
        let mut sources = Vec::new();
        for s in n.sources {
            if !ids.insert(s.id) || !placements.insert(s.province) {
                return Err("Duplicate: source ID or placement".into());
            }
            sources.push(Source {
                id: s.id,
                kind: s.kind,
                province: s.province,
                capacity: qty(&s.capacity)?,
            });
        }
        for r in &n.rails {
            if r.a >= r.b || r.level == 0 {
                return Err(
                    "InvalidReference: rail requires canonical a < b and positive level".into(),
                );
            }
            if !edges.insert((r.a, r.b)) {
                return Err("Duplicate: rail edge".into());
            }
            if !tuning.rail_capacity.contains_key(&r.level) {
                return Err("MissingContext: rail level capacity".into());
            }
        }
        sources.sort_by_key(|s| s.id);
        let mut rails = n.rails;
        rails.sort_by_key(|r| (r.a, r.b));
        nations.insert(n.id, Nation { sources, rails });
    }
    Ok(Config { tuning, nations })
}
impl Config {
    pub fn tuning(&self) -> &Tuning {
        &self.tuning
    }
    pub fn nations(&self) -> &BTreeMap<u16, Nation> {
        &self.nations
    }
    /// Diagnostic normalized identity only; never World state_hash or pack identity.
    pub fn identity(&self) -> Result<u64, String> {
        oh_core::state_hash(&("external-supply-network-v1", POLICY, self))
            .map_err(|e| e.to_string())
    }
    pub fn validate_references(&self, loaded: &LoadedNational) -> Result<(), String> {
        self.validate_against(&loaded.nations, &loaded.map)
    }
    /// Validate static references only, not current modifier factors or building instances.
    pub fn validate_against(
        &self,
        nations: &[NationDefinition],
        map: &MapData,
    ) -> Result<(), String> {
        let land = |id: u16| -> Result<&crate::map::ProvinceDefinition, String> {
            let p = map
                .provinces
                .iter()
                .find(|p| p.id == id)
                .ok_or("InvalidReference: unknown province")?;
            if p.kind != ProvinceKind::Land || !map.states.iter().any(|s| s.provinces.contains(&id))
            {
                return Err("InvalidReference: province requires land and actual state".into());
            }
            Ok(p)
        };
        for (&id, n) in &self.nations {
            let definition = nations
                .iter()
                .find(|n| n.id == id)
                .ok_or("InvalidReference: unknown nation")?;
            land(definition.capital)?;
            for s in &n.sources {
                let p = land(s.province)?;
                if (s.kind == SourceKind::Capital) != (s.province == definition.capital) {
                    return Err("InvalidReference: source kind/capital mismatch".into());
                }
                if s.kind == SourceKind::Port && !p.coastal {
                    return Err("InvalidReference: port requires actual coastal land".into());
                }
            }
            for r in &n.rails {
                land(r.a)?;
                land(r.b)?;
                if !map.edges.iter().any(|e| {
                    e.a == r.a
                        && e.b == r.b
                        && matches!(
                            e.kind,
                            EdgeKind::Normal | EdgeKind::RiverSmall | EdgeKind::RiverLarge
                        )
                        && e.distance_km > Fx::ZERO
                }) {
                    return Err(
                        "InvalidReference: rail requires actual positive eligible land edge".into(),
                    );
                }
            }
        }
        Ok(())
    }
}
