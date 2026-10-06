//! M1 private, ID-sorted world. Definitions are immutable and excluded from saves.
//! WP-11 must restore all WorldInputs together with time/config/ordered commands,
//! reload definitions, verify pack identity, and rebuild ledgers at the saved tick.
use crate::ledger::{Modifier, ModifierOp, StatLedger};
use oh_core::{Fx, NationId, ProvinceId, StateId};
use serde::Serialize;
use std::{collections::BTreeMap, sync::Arc};
#[derive(Clone, Debug)]
pub struct Defs {
    nations: Vec<oh_data::national::NationDefinition>,
    map: oh_data::map::MapData,
    visuals: oh_data::national::VisualPalette,
    map_id: String,
}
impl Defs {
    pub fn visuals(&self) -> &oh_data::national::VisualPalette {
        &self.visuals
    }
    pub fn map_id(&self) -> &str {
        &self.map_id
    }
    pub fn nations(&self) -> &[oh_data::national::NationDefinition] {
        &self.nations
    }
    pub fn map(&self) -> &oh_data::map::MapData {
        &self.map
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NationState {
    id: NationId,
    tag: String,
    government: String,
    support: BTreeMap<String, Fx>,
}
impl NationState {
    pub fn id(&self) -> NationId {
        self.id
    }
    pub fn tag(&self) -> &str {
        &self.tag
    }
    pub fn government(&self) -> &str {
        &self.government
    }
    pub fn support(&self) -> &BTreeMap<String, Fx> {
        &self.support
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StateState {
    id: StateId,
    owner: NationId,
    population: i64,
    resources: BTreeMap<String, i64>,
    buildings: BTreeMap<String, i64>,
    base: Fx,
    modifiers: Vec<Modifier>,
    infrastructure: Fx,
    ledger: StatLedger,
}
impl StateState {
    pub fn id(&self) -> StateId {
        self.id
    }
    pub fn owner(&self) -> NationId {
        self.owner
    }
    pub fn population(&self) -> i64 {
        self.population
    }
    pub fn resources(&self) -> &BTreeMap<String, i64> {
        &self.resources
    }
    pub fn buildings(&self) -> &BTreeMap<String, i64> {
        &self.buildings
    }
    pub fn base(&self) -> Fx {
        self.base
    }
    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }
    pub fn infrastructure(&self) -> Fx {
        self.infrastructure
    }
    pub fn ledger(&self) -> &StatLedger {
        &self.ledger
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProvinceState {
    id: ProvinceId,
    state: Option<StateId>,
    owner: Option<NationId>,
    controller: Option<NationId>,
}
impl ProvinceState {
    pub fn id(&self) -> ProvinceId {
        self.id
    }
    pub fn state(&self) -> Option<StateId> {
        self.state
    }
    pub fn owner(&self) -> Option<NationId> {
        self.owner
    }
    pub fn controller(&self) -> Option<NationId> {
        self.controller
    }
}
/// Complete authoritative world snapshot, including bases and inactive/expiring
/// modifiers. Derived values/ledgers are retained for atomic query publication.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WorldInputs {
    nations: Vec<NationState>,
    states: Vec<StateState>,
    provinces: Vec<ProvinceState>,
}
impl WorldInputs {
    pub fn nations(&self) -> &[NationState] {
        &self.nations
    }
    pub fn states(&self) -> &[StateState] {
        &self.states
    }
    pub fn provinces(&self) -> &[ProvinceState] {
        &self.provinces
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct World {
    #[serde(skip)]
    defs: Arc<Defs>,
    /// Immutable definition identity belongs in hashes; full Defs are save-excluded.
    definitions_hash: u64,
    inputs: WorldInputs,
}
impl World {
    pub fn from_loaded(loaded: &oh_data::national::LoadedNational) -> Result<Self, String> {
        let mut canonical = loaded.clone();
        canonical.nations.sort_by_key(|n| n.id);
        canonical.map.states.sort_by_key(|s| s.id);
        canonical.map.provinces.sort_by_key(|p| p.id);
        let loaded = &canonical;
        let nation_id = |tag: &str| {
            loaded
                .nations
                .iter()
                .find(|n| n.tag == tag)
                .map(|n| NationId(n.id))
                .ok_or_else(|| "unknown nation".to_string())
        };
        let mut nations = Vec::new();
        for n in &loaded.nations {
            let support = n
                .ideology_support
                .iter()
                .map(|(k, v)| {
                    v.parse::<Fx>()
                        .map(|v| (k.clone(), v))
                        .map_err(|e| e.to_string())
                })
                .collect::<Result<_, _>>()?;
            nations.push(NationState {
                id: NationId(n.id),
                tag: n.tag.clone(),
                government: n.government_key.clone(),
                support,
            });
        }
        nations.sort_by_key(|n| n.id);
        let mut states = Vec::new();
        for s in &loaded.map.states {
            let base = Fx::checked_from_num(s.infrastructure).ok_or("infrastructure overflow")?;
            let mut modifiers = loaded
                .scenario
                .state_modifiers
                .get(&s.id)
                .into_iter()
                .flatten()
                .map(|m| {
                    Ok(Modifier {
                        source: m.source.clone(),
                        target_stat: m.target_stat.clone(),
                        op: match m.operation.as_str() {
                            "add" => ModifierOp::Add,
                            "mul" => ModifierOp::Mul,
                            _ => return Err("invalid modifier operation".to_string()),
                        },
                        value: m.value.parse::<Fx>().map_err(|e| e.to_string())?,
                        expires: m.expires,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            modifiers.sort_by(|a, b| {
                (&a.target_stat, a.op, &a.source, a.expires, a.value).cmp(&(
                    &b.target_stat,
                    b.op,
                    &b.source,
                    b.expires,
                    b.value,
                ))
            });
            let ledger = StatLedger::evaluate("infrastructure", base, &modifiers, 0)
                .map_err(|e| e.to_string())?;
            states.push(StateState {
                id: StateId(s.id),
                owner: nation_id(
                    loaded
                        .scenario
                        .ownership
                        .get(&s.id)
                        .ok_or("missing owner")?,
                )?,
                population: s.population,
                resources: s.resources.clone(),
                buildings: s.buildings.clone(),
                base,
                modifiers,
                infrastructure: ledger.value(),
                ledger,
            });
        }
        states.sort_by_key(|s| s.id);
        let mut provinces = Vec::new();
        for p in &loaded.map.provinces {
            let state = loaded.map.state_for_province(p.id).map(StateId);
            let owner = state.map(|id| {
                states
                    .iter()
                    .find(|s| s.id == id)
                    .expect("validated state")
                    .owner
            });
            let controller = loaded
                .scenario
                .control_overrides
                .get(&p.id)
                .map(|t| nation_id(t))
                .transpose()?
                .or(owner);
            provinces.push(ProvinceState {
                id: ProvinceId(p.id),
                state,
                owner,
                controller,
            });
        }
        provinces.sort_by_key(|p| p.id);
        let map_identity: Vec<_> = loaded
            .map
            .provinces
            .iter()
            .map(|p| {
                (
                    p.id,
                    p.rgb,
                    match p.kind {
                        oh_data::map::ProvinceKind::Land => 0u8,
                        oh_data::map::ProvinceKind::Sea => 1,
                        oh_data::map::ProvinceKind::Lake => 2,
                    },
                    &p.terrain,
                    p.coastal,
                    p.island,
                )
            })
            .collect();
        let state_identity: Vec<_> = loaded
            .map
            .states
            .iter()
            .map(|s| (s.id, &s.name_key, &s.provinces))
            .collect();
        let edges: Vec<_> = loaded
            .map
            .edges
            .iter()
            .map(|e| {
                (
                    e.a,
                    e.b,
                    match e.kind {
                        oh_data::map::EdgeKind::Normal => 0u8,
                        oh_data::map::EdgeKind::RiverSmall => 1,
                        oh_data::map::EdgeKind::RiverLarge => 2,
                        oh_data::map::EdgeKind::Strait => 3,
                        oh_data::map::EdgeKind::Impassable => 4,
                    },
                    e.distance_km,
                )
            })
            .collect();
        let victory_points: Vec<_> = loaded
            .map
            .victory_points
            .iter()
            .map(|v| (v.province, v.points, &v.name_key))
            .collect();
        let definitions_hash = oh_core::state_hash(&(
            &loaded.nations,
            map_identity,
            state_identity,
            loaded.map.width,
            loaded.map.height,
            &loaded.map.index,
            &loaded.visuals,
            &loaded.scenario.map,
            &loaded.map.centers,
            edges,
            victory_points,
            loaded.map.km_per_pixel,
        ))
        .map_err(|e| e.to_string())?;
        Ok(Self {
            defs: Arc::new(Defs {
                nations: loaded.nations.clone(),
                map: loaded.map.clone(),
                visuals: loaded.visuals.clone(),
                map_id: loaded.scenario.map.clone(),
            }),
            definitions_hash,
            inputs: WorldInputs {
                nations,
                states,
                provinces,
            },
        })
    }
    pub fn definitions_hash(&self) -> u64 {
        self.definitions_hash
    }
    pub fn defs(&self) -> &Defs {
        &self.defs
    }
    pub fn inputs(&self) -> &WorldInputs {
        &self.inputs
    }
    pub fn nation(&self, id: NationId) -> Option<&NationState> {
        self.inputs
            .nations
            .binary_search_by_key(&id, |n| n.id)
            .ok()
            .map(|i| &self.inputs.nations[i])
    }
    pub fn state(&self, id: StateId) -> Option<&StateState> {
        self.inputs
            .states
            .binary_search_by_key(&id, |s| s.id)
            .ok()
            .map(|i| &self.inputs.states[i])
    }
    pub fn province(&self, id: ProvinceId) -> Option<&ProvinceState> {
        self.inputs
            .provinces
            .binary_search_by_key(&id, |p| p.id)
            .ok()
            .map(|i| &self.inputs.provinces[i])
    }
    /// Scheduler-only mutation. Caller works on a clone and commits atomically.
    pub(crate) fn evaluate(&mut self, tick: u64) -> Result<(), String> {
        for s in &mut self.inputs.states {
            let ledger = StatLedger::evaluate("infrastructure", s.base, &s.modifiers, tick)
                .map_err(|e| e.to_string())?;
            s.infrastructure = ledger.value();
            ledger
                .verify_applied_value(s.infrastructure)
                .map_err(|e| e.to_string())?;
            s.ledger = ledger;
        }
        Ok(())
    }
}
