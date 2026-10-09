//! M1 private, ID-sorted world. Definitions are immutable and excluded from saves.
//! WP-11 must restore all WorldInputs together with time/config/ordered commands,
//! reload definitions, verify pack identity, and rebuild ledgers at the saved tick.
use crate::ledger::{Modifier, ModifierOp, StatLedger};
use crate::save_state::*;
use oh_core::{Fx, NationId, ProvinceId, StateId};
use serde::Serialize;
use std::{collections::BTreeMap, sync::Arc};
#[derive(Clone, Debug)]
pub struct Defs {
    military: Option<oh_data::military::Definition>,
    production: Option<oh_data::production::Definition>,
    trigger: Option<oh_data::trigger::Definition>,
    economy: Option<oh_data::economy::Definition>,
    nations: Vec<oh_data::national::NationDefinition>,
    map: oh_data::map::MapData,
    visuals: oh_data::national::VisualPalette,
    map_id: String,
}
impl Defs {
    pub fn military(&self) -> Option<&oh_data::military::Definition> {
        self.military.as_ref()
    }
    pub fn production(&self) -> Option<&oh_data::production::Definition> {
        self.production.as_ref()
    }
    pub fn economy(&self) -> Option<&oh_data::economy::Definition> {
        self.economy.as_ref()
    }
    pub fn trigger(&self) -> Option<&oh_data::trigger::Definition> {
        self.trigger.as_ref()
    }
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
    pub(crate) fn export_save(&self) -> WorldV1 {
        WorldV1 {
            definitions_hash: self.definitions_hash,
            inputs: InputsV1 {
                nations: self
                    .inputs
                    .nations
                    .iter()
                    .map(|n| NationV1 {
                        id: n.id.0,
                        tag: n.tag.clone(),
                        government: n.government.clone(),
                        support: n
                            .support
                            .iter()
                            .map(|(k, v)| (k.clone(), v.to_bits()))
                            .collect(),
                    })
                    .collect(),
                states: self
                    .inputs
                    .states
                    .iter()
                    .map(|s| StateWorldV1 {
                        id: s.id.0,
                        owner: s.owner.0,
                        population: s.population,
                        resources: s.resources.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                        buildings: s.buildings.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                        base: s.base.to_bits(),
                        modifiers: s
                            .modifiers
                            .iter()
                            .map(|m| ModifierV1 {
                                source: m.source.clone(),
                                target_stat: m.target_stat.clone(),
                                op: match m.op {
                                    ModifierOp::Add => ModifierOpV1::Add,
                                    ModifierOp::Mul => ModifierOpV1::Mul,
                                },
                                value: m.value.to_bits(),
                                expires: m.expires,
                            })
                            .collect(),
                        infrastructure: s.infrastructure.to_bits(),
                        ledger: ledger_save(&s.ledger),
                    })
                    .collect(),
                provinces: self
                    .inputs
                    .provinces
                    .iter()
                    .map(|p| ProvinceV1 {
                        id: p.id.0,
                        state: p.state.map(|v| v.0),
                        owner: p.owner.map(|v| v.0),
                        controller: p.controller.map(|v| v.0),
                    })
                    .collect(),
            },
        }
    }
    pub(crate) fn from_save(dto: WorldV1, template: &Self, tick: u64) -> Result<Self, String> {
        if dto.definitions_hash != template.definitions_hash {
            return Err("DefinitionsMismatch".into());
        }
        let input = dto.inputs;
        if !strictly_sorted(input.nations.iter().map(|n| n.id))
            || !strictly_sorted(input.states.iter().map(|s| s.id))
            || !strictly_sorted(input.provinces.iter().map(|p| p.id))
            || input
                .nations
                .iter()
                .map(|n| n.id)
                .ne(template.inputs.nations.iter().map(|n| n.id.0))
            || input
                .states
                .iter()
                .map(|s| s.id)
                .ne(template.inputs.states.iter().map(|s| s.id.0))
            || input.provinces.iter().map(|p| p.id).ne(template
                .inputs
                .provinces
                .iter()
                .map(|p| p.id.0))
        {
            return Err("InvalidReference: entity ID sets/order".into());
        }
        let mut nations = Vec::new();
        for (n, definition) in input.nations.into_iter().zip(&template.defs.nations) {
            if n.tag != definition.tag
                || n.government.is_empty()
                || n.support.is_empty()
                || !strictly_sorted(n.support.iter().map(|(k, _)| k))
            {
                return Err(format!("InvalidNation: {}", n.id));
            }
            let mut total = 0i64;
            let mut support = BTreeMap::new();
            for (k, v) in n.support {
                if k.is_empty() || !(0..=Fx::ONE.to_bits()).contains(&v) {
                    return Err(format!("InvalidSupport: {}", n.id));
                }
                total = total.checked_add(v).ok_or("InvalidSupport: sum overflow")?;
                support.insert(k, Fx::from_bits(v));
            }
            if total != Fx::ONE.to_bits() {
                return Err(format!("InvalidSupport: {} sum", n.id));
            }
            nations.push(NationState {
                id: NationId(n.id),
                tag: n.tag,
                government: n.government,
                support,
            });
        }
        let has_nation = |id: u16| {
            nations
                .binary_search_by_key(&NationId(id), |n| n.id)
                .is_ok()
        };
        let mut states = Vec::new();
        for s in input.states {
            if !has_nation(s.owner) || s.population < 0 || s.base < 0 {
                return Err(format!("InvalidState: {} owner/population/base", s.id));
            }
            let resources = counts(
                s.resources,
                template.defs.map.resource_ids(),
                s.id,
                "resources",
            )?;
            let buildings = counts(
                s.buildings,
                template.defs.map.building_ids(),
                s.id,
                "buildings",
            )?;
            if !s
                .modifiers
                .windows(2)
                .all(|m| modifier_key(&m[0]) <= modifier_key(&m[1]))
            {
                return Err(format!("InvalidModifier: {} order", s.id));
            }
            let mut modifiers = Vec::new();
            for m in s.modifiers {
                if m.source.is_empty() || m.target_stat != "infrastructure" {
                    return Err(format!("InvalidModifier: {} source/target", s.id));
                }
                modifiers.push(Modifier {
                    source: m.source,
                    target_stat: m.target_stat,
                    op: match m.op {
                        ModifierOpV1::Add => ModifierOp::Add,
                        ModifierOpV1::Mul => ModifierOp::Mul,
                    },
                    value: Fx::from_bits(m.value),
                    expires: m.expires,
                });
            }
            let base = Fx::from_bits(s.base);
            let ledger = StatLedger::evaluate("infrastructure", base, &modifiers, tick)
                .map_err(|e| format!("InvalidLedger: {} {e}", s.id))?;
            if ledger_save(&ledger) != s.ledger || ledger.value().to_bits() != s.infrastructure {
                return Err(format!("LedgerMismatch: {}", s.id));
            }
            states.push(StateState {
                id: StateId(s.id),
                owner: NationId(s.owner),
                population: s.population,
                resources,
                buildings,
                base,
                modifiers,
                infrastructure: ledger.value(),
                ledger,
            });
        }
        let mut provinces = Vec::new();
        for p in input.provinces {
            let expected = template.defs.map.state_for_province(p.id);
            if p.state != expected {
                return Err(format!("InvalidReference: province {} state", p.id));
            }
            if let Some(state) = expected {
                let owner = states
                    .binary_search_by_key(&StateId(state), |s| s.id)
                    .ok()
                    .map(|i| states[i].owner.0);
                if p.owner != owner || !p.controller.is_some_and(has_nation) {
                    return Err(format!(
                        "InvalidReference: province {} owner/controller",
                        p.id
                    ));
                }
            } else if p.owner.is_some() || p.controller.is_some() {
                return Err(format!("InvalidReference: water province {}", p.id));
            }
            provinces.push(ProvinceState {
                id: ProvinceId(p.id),
                state: p.state.map(StateId),
                owner: p.owner.map(NationId),
                controller: p.controller.map(NationId),
            });
        }
        Ok(Self {
            defs: template.defs.clone(),
            definitions_hash: dto.definitions_hash,
            inputs: WorldInputs {
                nations,
                states,
                provinces,
            },
        })
    }
    pub fn from_loaded(loaded: &oh_data::national::LoadedNational) -> Result<Self, String> {
        if let Some(m) = &loaded.military {
            m.validate(loaded)?;
        }
        let mut canonical = loaded.clone();
        canonical.nations.sort_by_key(|n| n.id);
        canonical.map.states.sort_by_key(|s| s.id);
        canonical.map.provinces.sort_by_key(|p| p.id);
        let loaded = &canonical;
        if let Some(e) = &loaded.economy {
            e.validate(loaded)?;
        }
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
                military: loaded.military.clone(),
                trigger: oh_data::trigger::definition(loaded)?,
                economy: loaded.economy.clone(),
                production: loaded.production.clone(),
                nations: loaded.nations.clone(),
                map: loaded.map.clone(),
                visuals: loaded.visuals.clone(),
                map_id: loaded.scenario.map.clone(),
            }),
            definitions_hash: if let Some(m) = &loaded.military {
                oh_core::state_hash(&(definitions_hash, m.identity()?))
                    .map_err(|e| e.to_string())?
            } else {
                definitions_hash
            },
            inputs: WorldInputs {
                nations,
                states,
                provinces,
            },
        })
    }
    pub(crate) fn complete_building(
        &mut self,
        id: StateId,
        building: &str,
        level: i64,
        infrastructure: Option<Fx>,
    ) -> Result<(), String> {
        let state = self
            .inputs
            .states
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or("missing state")?;
        state.buildings.insert(building.into(), level);
        if let Some(value) = infrastructure {
            state.base = value;
        }
        Ok(())
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

fn modifier_key(m: &ModifierV1) -> (&str, ModifierOpV1, &str, Option<u64>, i64) {
    (&m.target_stat, m.op, &m.source, m.expires, m.value)
}
fn counts(
    values: Vec<(String, i64)>,
    registry: &std::collections::BTreeSet<String>,
    id: u16,
    field: &str,
) -> Result<BTreeMap<String, i64>, String> {
    if !strictly_sorted(values.iter().map(|(k, _)| k))
        || values.iter().any(|(k, v)| !registry.contains(k) || *v < 0)
    {
        return Err(format!("InvalidReference: state {id} {field}"));
    }
    Ok(values.into_iter().collect())
}
fn ledger_save(l: &StatLedger) -> LedgerV1 {
    LedgerV1 {
        target_stat: l.target_stat().into(),
        tick: l.tick(),
        value: l.value().to_bits(),
        entries: l
            .entries()
            .iter()
            .map(|e| EntryV1 {
                source: e.source.clone(),
                op: match e.op {
                    crate::ledger::LedgerOp::Base => LedgerOpV1::Base,
                    crate::ledger::LedgerOp::Add => LedgerOpV1::Add,
                    crate::ledger::LedgerOp::Mul => LedgerOpV1::Mul,
                },
                value: e.value.to_bits(),
                accumulated: e.accumulated.to_bits(),
            })
            .collect(),
    }
}
