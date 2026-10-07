//! Private economy authority. Immutable inputs stay in World::Defs.
use crate::{formula, world::World};
use oh_core::{Fx, NationId, Qty, StateId};
use oh_data::economy::{Definition, quantity, ratio, unit_ratio};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EconomyError {
    MissingContext,
    InvalidReference,
    InvalidValue,
    Overflow,
    Condition,
    InsufficientCapital,
    SameLaw,
    DuplicateProject,
    SlotCap,
    TargetConflict,
}
impl std::fmt::Display for EconomyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "economy:{self:?}")
    }
}
impl From<formula::EconomyArithmeticError> for EconomyError {
    fn from(e: formula::EconomyArithmeticError) -> Self {
        match e {
            formula::EconomyArithmeticError::InvalidValue => Self::InvalidValue,
            formula::EconomyArithmeticError::Overflow => Self::Overflow,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Allocate {
        ratios: [Fx; 4],
    },
    Construct {
        project: u64,
        state: u16,
        building: String,
    },
    Cancel {
        project: u64,
    },
    Reorder {
        projects: Vec<u64>,
    },
    ChangeLaw {
        law: String,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Dormancy {
    NoOwnership,
    TargetConflict,
    SlotCap,
    ZeroCap,
    ZeroFactor,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: u64,
    pub state: u16,
    pub building: String,
    pub target: i64,
    pub progress: Qty,
    pub dormancy: Option<Dormancy>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Contribution {
    pub state: u16,
    pub building: String,
    pub levels: i64,
    pub unit_ic: Qty,
    pub value: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Multiplier {
    pub source: String,
    pub factor: Fx,
    pub applied: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuantityLedger {
    pub tick: u64,
    pub contributions: Vec<Contribution>,
    pub population_by_state: BTreeMap<u16, i64>,
    pub resources_by_state: BTreeMap<u16, BTreeMap<String, i64>>,
    pub population: i64,
    pub resources: BTreeMap<String, i64>,
    pub multipliers: Vec<Multiplier>,
    pub total_ic: Qty,
    pub minimum: Fx,
    pub ratios: [Fx; 4],
    pub allocation: [Qty; 4],
    pub consumer_residual: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConstructionEntry {
    pub project: u64,
    pub factor: Fx,
    pub consumed: Qty,
    pub applied: Qty,
    pub discarded: Qty,
    pub completed: bool,
    pub dormancy: Option<Dormancy>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConstructionLedger {
    pub tick: u64,
    pub budget: Qty,
    pub entries: Vec<ConstructionEntry>,
    pub unused: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NationEconomy {
    laws: BTreeMap<String, String>,
    political_capital: Qty,
    stability: Fx,
    mobilization: Fx,
    allocation: [Fx; 4],
    committed: i64,
    reserved: i64,
    capacity: i64,
    projects: Vec<Project>,
    ledger: QuantityLedger,
    construction: ConstructionLedger,
}
impl NationEconomy {
    pub fn laws(&self) -> &BTreeMap<String, String> {
        &self.laws
    }
    pub fn political_capital(&self) -> Qty {
        self.political_capital
    }
    pub fn stability(&self) -> Fx {
        self.stability
    }
    pub fn mobilization(&self) -> Fx {
        self.mobilization
    }
    pub fn allocation(&self) -> [Fx; 4] {
        self.allocation
    }
    pub fn committed(&self) -> i64 {
        self.committed
    }
    pub fn reserved(&self) -> i64 {
        self.reserved
    }
    pub fn capacity(&self) -> i64 {
        self.capacity
    }
    pub fn available(&self) -> i64 {
        (self.capacity - self.committed - self.reserved).max(0)
    }
    pub fn overcommitted(&self) -> i64 {
        (self.committed + self.reserved - self.capacity).max(0)
    }
    pub fn projects(&self) -> &[Project] {
        &self.projects
    }
    pub fn ledger(&self) -> &QuantityLedger {
        &self.ledger
    }
    pub fn construction_ledger(&self) -> &ConstructionLedger {
        &self.construction
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Economy {
    definitions_hash: u64,
    nations: BTreeMap<u16, NationEconomy>,
}
fn fx(s: &str) -> Result<Fx, EconomyError> {
    ratio(s).map_err(|_| EconomyError::InvalidValue)
}
fn qty(s: &str) -> Result<Qty, EconomyError> {
    quantity(s).map_err(|_| EconomyError::InvalidValue)
}
fn defs(world: &World) -> Result<&Definition, EconomyError> {
    world.defs().economy().ok_or(EconomyError::MissingContext)
}
fn sum(a: Qty, b: Qty) -> Result<Qty, EconomyError> {
    a.checked_add(b).ok_or(EconomyError::Overflow)
}
fn units(q: Qty, count: i64) -> Result<Qty, EconomyError> {
    if count < 0 {
        return Err(EconomyError::InvalidValue);
    }
    let raw = i128::from(q.to_bits()) * i128::from(count);
    Ok(Qty::from_bits(
        i64::try_from(raw).map_err(|_| EconomyError::Overflow)?,
    ))
}
fn slots(world: &World, definition: &Definition, state: u16) -> Result<i64, EconomyError> {
    let s = world
        .state(StateId(state))
        .ok_or(EconomyError::InvalidReference)?;
    let mut occupied = 0i64;
    for (id, level) in s.buildings() {
        let b = definition
            .buildings
            .get(id)
            .ok_or(EconomyError::InvalidReference)?;
        let count = usize::try_from(*level).map_err(|_| EconomyError::InvalidValue)?;
        let costs = b.slots.get(..count).ok_or(EconomyError::TargetConflict)?;
        for amount in costs {
            occupied = occupied
                .checked_add(*amount)
                .ok_or(EconomyError::Overflow)?;
        }
    }
    Ok(occupied)
}
impl Economy {
    pub fn nation(&self, id: NationId) -> Option<&NationEconomy> {
        self.nations.get(&id.0)
    }
    pub fn definitions_hash(&self) -> u64 {
        self.definitions_hash
    }
    pub fn nations(&self) -> &BTreeMap<u16, NationEconomy> {
        &self.nations
    }
    pub(crate) fn initial(world: &World, tick: u64) -> Result<Self, EconomyError> {
        let d = defs(world)?;
        let mut nations = BTreeMap::new();
        for (id, input) in &d.nations {
            let allocation = input
                .allocation
                .each_ref()
                .map(|s| unit_ratio(s).map_err(|_| EconomyError::InvalidValue));
            let allocation = [
                allocation[0].clone()?,
                allocation[1].clone()?,
                allocation[2].clone()?,
                allocation[3].clone()?,
            ];
            let mut n = NationEconomy {
                laws: input.laws.clone(),
                political_capital: qty(&input.political_capital)?,
                stability: fx(&input.stability)?,
                mobilization: fx(&input.mobilization)?,
                allocation,
                committed: input.committed,
                reserved: input.reserved,
                capacity: 0,
                projects: vec![],
                ledger: QuantityLedger {
                    tick,
                    contributions: vec![],
                    population_by_state: BTreeMap::new(),
                    resources_by_state: BTreeMap::new(),
                    population: 0,
                    resources: BTreeMap::new(),
                    multipliers: vec![],
                    total_ic: Qty::ZERO,
                    minimum: Fx::ZERO,
                    ratios: allocation,
                    allocation: [Qty::ZERO; 4],
                    consumer_residual: Qty::ZERO,
                },
                construction: ConstructionLedger {
                    tick,
                    budget: Qty::ZERO,
                    entries: vec![],
                    unused: Qty::ZERO,
                },
            };
            let (ledger, capacity) = Self::derive(world, *id, &n, tick)?;
            n.capacity = capacity;
            n.ledger = ledger;
            n.construction.budget = n.ledger.allocation[1];
            n.construction.unused = n.construction.budget;
            nations.insert(*id, n);
        }
        Ok(Self {
            definitions_hash: oh_core::state_hash(d).map_err(|_| EconomyError::InvalidValue)?,
            nations,
        })
    }
    fn derive(
        world: &World,
        id: u16,
        n: &NationEconomy,
        tick: u64,
    ) -> Result<(QuantityLedger, i64), EconomyError> {
        let d = defs(world)?;
        let input = d.nations.get(&id).ok_or(EconomyError::InvalidReference)?;
        if n.stability < Fx::ZERO
            || n.stability > Fx::ONE
            || n.mobilization < Fx::ZERO
            || n.mobilization > Fx::ONE
            || n.political_capital < Qty::ZERO
            || n.political_capital > qty(&input.political_capital_cap)?
            || n.committed < 0
            || n.reserved < 0
            || n.committed.checked_add(n.reserved).is_none()
        {
            return Err(EconomyError::InvalidValue);
        }
        if n.laws.keys().ne(input.laws.keys()) {
            return Err(EconomyError::InvalidReference);
        }
        let mut ledger = QuantityLedger {
            tick,
            contributions: vec![],
            population_by_state: BTreeMap::new(),
            resources_by_state: BTreeMap::new(),
            population: 0,
            resources: BTreeMap::new(),
            multipliers: vec![],
            total_ic: Qty::ZERO,
            minimum: Fx::ZERO,
            ratios: n.allocation,
            allocation: [Qty::ZERO; 4],
            consumer_residual: Qty::ZERO,
        };
        for s in world.inputs().states().iter().filter(|s| s.owner().0 == id) {
            if s.population() < 0 || s.resources().values().any(|v| *v < 0) {
                return Err(EconomyError::InvalidValue);
            }
            ledger.population = ledger
                .population
                .checked_add(s.population())
                .ok_or(EconomyError::Overflow)?;
            ledger.population_by_state.insert(s.id().0, s.population());
            ledger
                .resources_by_state
                .insert(s.id().0, s.resources().clone());
            for (key, value) in s.resources() {
                let entry = ledger.resources.entry(key.clone()).or_insert(0);
                *entry = entry.checked_add(*value).ok_or(EconomyError::Overflow)?;
            }
            for (id, levels) in s.buildings() {
                let b = d.buildings.get(id).ok_or(EconomyError::InvalidReference)?;
                let unit = qty(&b.ic_per_level)?;
                let contribution = units(unit, *levels)?;
                ledger.total_ic = sum(ledger.total_ic, contribution)?;
                ledger.contributions.push(Contribution {
                    state: s.id().0,
                    building: id.clone(),
                    levels: *levels,
                    unit_ic: unit,
                    value: contribution,
                });
            }
        }
        let factor = fx(&input.ic_multiplier)?;
        ledger.total_ic = formula::quantity_times_ratio(ledger.total_ic, factor)?;
        ledger.multipliers.push(Multiplier {
            source: format!("nation:{id}"),
            factor,
            applied: ledger.total_ic,
        });
        for (category, id) in &n.laws {
            let law = d.laws.get(id).ok_or(EconomyError::InvalidReference)?;
            if &law.category != category {
                return Err(EconomyError::InvalidReference);
            }
            let factor = fx(&law.ic_multiplier)?;
            ledger.total_ic = formula::quantity_times_ratio(ledger.total_ic, factor)?;
            ledger.multipliers.push(Multiplier {
                source: format!("law:{id}"),
                factor,
                applied: ledger.total_ic,
            });
        }
        let law = d
            .laws
            .get(
                n.laws
                    .get(&d.economy_category)
                    .ok_or(EconomyError::InvalidReference)?,
            )
            .ok_or(EconomyError::InvalidReference)?;
        ledger.minimum = formula::economy_minimum(
            fx(&law.consumer_base)?,
            fx(&law.instability_slope)?,
            n.stability,
        )?;
        ledger.allocation =
            formula::economy_allocate(ledger.total_ic, n.allocation, ledger.minimum)?;
        ledger.consumer_residual =
            ledger.allocation[0] - formula::quantity_times_ratio(ledger.total_ic, n.allocation[0])?;
        let conscription = d
            .laws
            .get(
                n.laws
                    .get(&d.conscription_category)
                    .ok_or(EconomyError::InvalidReference)?,
            )
            .ok_or(EconomyError::InvalidReference)?;
        let capacity =
            formula::economy_capacity(ledger.population, fx(&conscription.conscription_ratio)?)?;
        Ok((ledger, capacity))
    }
    pub(crate) fn command(
        &mut self,
        world: &World,
        id: NationId,
        action: &Action,
        tick: u64,
        condition_met: bool,
    ) -> Result<(), EconomyError> {
        let d = defs(world)?;
        let mut next = self
            .nations
            .get(&id.0)
            .ok_or(EconomyError::InvalidReference)?
            .clone();
        match action {
            Action::Allocate { ratios } => next.allocation = *ratios,
            Action::ChangeLaw { law } => {
                let l = d.laws.get(law).ok_or(EconomyError::InvalidReference)?;
                if next.laws.get(&l.category) == Some(law) {
                    return Err(EconomyError::SameLaw);
                }
                if !condition_met {
                    return Err(EconomyError::Condition);
                }
                let cost = qty(&l.cost)?;
                if cost > next.political_capital {
                    return Err(EconomyError::InsufficientCapital);
                }
                next.political_capital -= cost;
                next.laws.insert(l.category.clone(), law.clone());
            }
            Action::Construct {
                project,
                state,
                building,
            } => {
                let s = world
                    .state(StateId(*state))
                    .ok_or(EconomyError::InvalidReference)?;
                if s.owner() != id {
                    return Err(EconomyError::InvalidReference);
                }
                if next
                    .projects
                    .iter()
                    .any(|p| p.id == *project || (p.state == *state && &p.building == building))
                {
                    return Err(EconomyError::DuplicateProject);
                }
                let b = d
                    .buildings
                    .get(building)
                    .ok_or(EconomyError::InvalidReference)?;
                let level = *s.buildings().get(building).unwrap_or(&0);
                let index = usize::try_from(level).map_err(|_| EconomyError::InvalidValue)?;
                let increase = *b.slots.get(index).ok_or(EconomyError::TargetConflict)?;
                let mut occupied = slots(world, d, *state)?;
                for p in next.projects.iter().filter(|p| p.state == *state) {
                    let b = d
                        .buildings
                        .get(&p.building)
                        .ok_or(EconomyError::InvalidReference)?;
                    let i =
                        usize::try_from(p.target - 1).map_err(|_| EconomyError::InvalidValue)?;
                    occupied = occupied
                        .checked_add(*b.slots.get(i).ok_or(EconomyError::InvalidReference)?)
                        .ok_or(EconomyError::Overflow)?;
                }
                if occupied
                    .checked_add(increase)
                    .ok_or(EconomyError::Overflow)?
                    > *d.state_slots
                        .get(state)
                        .ok_or(EconomyError::InvalidReference)?
                {
                    return Err(EconomyError::SlotCap);
                }
                next.projects.push(Project {
                    id: *project,
                    state: *state,
                    building: building.clone(),
                    target: level.checked_add(1).ok_or(EconomyError::Overflow)?,
                    progress: Qty::ZERO,
                    dormancy: None,
                });
            }
            Action::Cancel { project } => {
                let index = next
                    .projects
                    .iter()
                    .position(|p| p.id == *project)
                    .ok_or(EconomyError::InvalidReference)?;
                next.projects.remove(index);
            }
            Action::Reorder { projects } => {
                if projects.len() != next.projects.len()
                    || projects.iter().collect::<BTreeSet<_>>().len() != projects.len()
                {
                    return Err(EconomyError::InvalidValue);
                }
                let sorted = projects
                    .iter()
                    .map(|id| {
                        next.projects
                            .iter()
                            .find(|p| p.id == *id)
                            .cloned()
                            .ok_or(EconomyError::InvalidReference)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                next.projects = sorted;
            }
        }
        let (_, capacity) = Self::derive(world, id.0, &next, tick)?;
        next.capacity = capacity;
        self.nations.insert(id.0, next);
        Ok(())
    }
    pub(crate) fn political_effect(
        &mut self,
        world: &World,
        id: NationId,
        effect: &oh_data::trigger::Effect,
        tick: u64,
        condition_met: bool,
    ) -> Result<(), EconomyError> {
        if let oh_data::trigger::Effect::SetLaw(law) = effect {
            return self.command(
                world,
                id,
                &Action::ChangeLaw { law: law.clone() },
                tick,
                condition_met,
            );
        }
        let mut next = self
            .nations
            .get(&id.0)
            .ok_or(EconomyError::InvalidReference)?
            .clone();
        match effect {
            oh_data::trigger::Effect::AddStability(v) => {
                next.stability = next
                    .stability
                    .checked_add(v.parse::<Fx>().map_err(|_| EconomyError::InvalidValue)?)
                    .ok_or(EconomyError::Overflow)?
            }
            oh_data::trigger::Effect::AddMobilization(v) => {
                next.mobilization = next
                    .mobilization
                    .checked_add(v.parse::<Fx>().map_err(|_| EconomyError::InvalidValue)?)
                    .ok_or(EconomyError::Overflow)?
            }
            oh_data::trigger::Effect::AddPoliticalCapital(v) => {
                next.political_capital = next
                    .political_capital
                    .checked_add(v.parse::<Qty>().map_err(|_| EconomyError::InvalidValue)?)
                    .ok_or(EconomyError::Overflow)?
            }
            _ => return Err(EconomyError::InvalidValue),
        }
        let (_, capacity) = Self::derive(world, id.0, &next, tick)?;
        next.capacity = capacity;
        self.nations.insert(id.0, next);
        Ok(())
    }
    pub(crate) fn daily_economy(&mut self, world: &World, tick: u64) -> Result<(), EconomyError> {
        for (id, n) in &mut self.nations {
            let (ledger, capacity) = Self::derive(world, *id, n, tick)?;
            n.ledger = ledger;
            n.capacity = capacity;
        }
        Ok(())
    }
    pub(crate) fn daily_politics(&mut self, world: &World) -> Result<(), EconomyError> {
        let d = defs(world)?;
        for (id, n) in &mut self.nations {
            let input = d.nations.get(id).ok_or(EconomyError::InvalidReference)?;
            n.political_capital = formula::economy_political_income(
                n.political_capital,
                qty(&input.political_capital_daily)?,
                qty(&input.political_capital_cap)?,
            )?;
        }
        Ok(())
    }
    pub(crate) fn daily_construction(
        &mut self,
        world: &mut World,
        tick: u64,
    ) -> Result<(), EconomyError> {
        let d = defs(world)?.clone();
        let infrastructure: BTreeMap<_, _> = world
            .inputs()
            .states()
            .iter()
            .map(|s| (s.id().0, s.infrastructure()))
            .collect();
        for (id, n) in &mut self.nations {
            let budget = n.ledger.allocation[1];
            let mut ledger = ConstructionLedger {
                tick,
                budget,
                entries: vec![],
                unused: budget,
            };
            for index in 0..n.projects.len() {
                let p = n.projects[index].clone();
                let s = world
                    .state(StateId(p.state))
                    .ok_or(EconomyError::InvalidReference)?;
                let b = d
                    .buildings
                    .get(&p.building)
                    .ok_or(EconomyError::InvalidReference)?;
                let level = *s.buildings().get(&p.building).unwrap_or(&0);
                let stage =
                    usize::try_from(p.target - 1).map_err(|_| EconomyError::InvalidValue)?;
                let cost = qty(b.costs.get(stage).ok_or(EconomyError::InvalidReference)?)?;
                let mut reason = if s.owner().0 != *id {
                    Some(Dormancy::NoOwnership)
                } else if level.checked_add(1) != Some(p.target) {
                    Some(Dormancy::TargetConflict)
                } else {
                    None
                };
                if reason.is_none() {
                    let mut occupied = slots(world, &d, p.state)?;
                    for other in n.projects.iter().filter(|p| p.state == s.id().0) {
                        let b = d
                            .buildings
                            .get(&other.building)
                            .ok_or(EconomyError::InvalidReference)?;
                        let stage = usize::try_from(other.target - 1)
                            .map_err(|_| EconomyError::InvalidValue)?;
                        occupied = occupied
                            .checked_add(*b.slots.get(stage).ok_or(EconomyError::InvalidReference)?)
                            .ok_or(EconomyError::Overflow)?;
                    }
                    if occupied
                        > *d.state_slots
                            .get(&p.state)
                            .ok_or(EconomyError::InvalidReference)?
                    {
                        reason = Some(Dormancy::SlotCap);
                    }
                }
                let mut factor = Fx::ZERO;
                let cap = qty(&b.daily_cap)?;
                if reason.is_none() {
                    let infra = infrastructure[&p.state];
                    factor = b
                        .infrastructure_factors
                        .iter()
                        .find_map(|v| {
                            fx(&v.infrastructure)
                                .ok()
                                .filter(|v| *v == infra)
                                .map(|_| fx(&v.factor))
                        })
                        .ok_or(EconomyError::InvalidReference)??;
                    if cap == Qty::ZERO {
                        reason = Some(Dormancy::ZeroCap);
                    } else if factor == Fx::ZERO {
                        reason = Some(Dormancy::ZeroFactor);
                    }
                }
                let (consumed, applied, discarded) = if reason.is_none() {
                    formula::economy_construction(
                        ledger.unused.min(cap),
                        cost.checked_sub(p.progress)
                            .ok_or(EconomyError::InvalidValue)?,
                        factor,
                    )?
                } else {
                    (Qty::ZERO, Qty::ZERO, Qty::ZERO)
                };
                n.projects[index].progress = sum(p.progress, applied)?;
                n.projects[index].dormancy = reason;
                let completed = reason.is_none() && n.projects[index].progress == cost;
                if completed {
                    world
                        .complete_building(
                            StateId(p.state),
                            &p.building,
                            p.target,
                            b.infrastructure_levels
                                .as_ref()
                                .map(|levels| fx(&levels[stage]))
                                .transpose()?,
                        )
                        .map_err(|_| EconomyError::InvalidValue)?;
                }
                ledger.unused = ledger
                    .unused
                    .checked_sub(consumed)
                    .ok_or(EconomyError::Overflow)?;
                ledger.entries.push(ConstructionEntry {
                    project: p.id,
                    factor,
                    consumed,
                    applied,
                    discarded,
                    completed,
                    dormancy: reason,
                });
            }
            let completed: BTreeSet<_> = ledger
                .entries
                .iter()
                .filter(|e| e.completed)
                .map(|e| e.project)
                .collect();
            n.projects.retain(|p| !completed.contains(&p.id));
            n.construction = ledger;
        }
        Ok(())
    }
}
