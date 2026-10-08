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
#[serde(deny_unknown_fields)]
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

/// Trusted future consumer API. No player command maps to these operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManpowerOperation {
    Reserve,
    CommitReservation,
    Consume,
    CancelReservation,
    ReturnCommitted,
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
    pub laws: BTreeMap<String, String>,
    pub stability: Fx,
    pub capacity: i64,
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
    pub state: u16,
    pub building: String,
    pub target: i64,
    pub starting_progress: Qty,
    pub cost: Qty,
    pub daily_cap: Qty,
    pub infrastructure: Fx,
    pub owner: u16,
    pub factor_evaluated: bool,
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
    scores: Option<BTreeMap<u16, IndustrialScore>>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndustrialScore {
    pub tick: u64,
    pub input_tick: u64,
    pub weight: Fx,
    pub input: Qty,
    pub term: Qty,
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
fn target_stage(target: i64, building: &oh_data::economy::Building) -> Result<usize, EconomyError> {
    let stage = target
        .checked_sub(1)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(EconomyError::InvalidValue)?;
    if stage >= building.costs.len() || stage >= building.slots.len() {
        return Err(EconomyError::InvalidValue);
    }
    Ok(stage)
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
        if count > b.costs.len() {
            return Err(EconomyError::TargetConflict);
        }
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
    pub fn industrial_scores(&self) -> Option<&BTreeMap<u16, IndustrialScore>> {
        self.scores.as_ref()
    }
    pub(crate) fn checkpoint_scores(
        &mut self,
        world: &World,
        tick: u64,
    ) -> Result<(), EconomyError> {
        let Some(weights) = world
            .defs()
            .trigger()
            .and_then(|d| d.score_weights.as_ref())
        else {
            self.scores = None;
            return Ok(());
        };
        let values = weights.values().map_err(|_| EconomyError::InvalidValue)?;
        if [0, 2, 3].iter().any(|i| values[*i] != Fx::ZERO) {
            return Err(EconomyError::MissingContext);
        }
        let mut scores = BTreeMap::new();
        for (id, n) in &self.nations {
            scores.insert(
                *id,
                IndustrialScore {
                    tick,
                    input_tick: n.ledger.tick,
                    weight: values[1],
                    input: n.ledger.total_ic,
                    term: formula::economy_weighted_quantity(n.ledger.total_ic, values[1])?,
                },
            );
        }
        self.scores = Some(scores);
        Ok(())
    }
    pub(crate) fn validate_scores(
        &self,
        world: &World,
        tick: u64,
        ended: bool,
    ) -> Result<(), EconomyError> {
        let mut expected = self.clone();
        if ended {
            expected.checkpoint_scores(world, tick)?;
        } else {
            expected.scores = None;
        }
        if self.scores != expected.scores {
            return Err(EconomyError::InvalidValue);
        }
        Ok(())
    }
    pub fn nation(&self, id: NationId) -> Option<&NationEconomy> {
        self.nations.get(&id.0)
    }
    pub fn definitions_hash(&self) -> u64 {
        self.definitions_hash
    }
    pub fn nations(&self) -> &BTreeMap<u16, NationEconomy> {
        &self.nations
    }
    /// A caller needs exclusive private authority; Simulation exposes only &Economy.
    /// WP16 must bind actual reservation/project IDs and exactly-once ownership before using this API.
    pub fn manpower(
        &mut self,
        nation: NationId,
        operation: ManpowerOperation,
        amount: i64,
    ) -> Result<(), EconomyError> {
        if amount < 0 {
            return Err(EconomyError::InvalidValue);
        }
        let mut next = self
            .nations
            .get(&nation.0)
            .ok_or(EconomyError::InvalidReference)?
            .clone();
        match operation {
            ManpowerOperation::Reserve | ManpowerOperation::Consume => {
                if amount > next.available() {
                    return Err(EconomyError::InvalidValue);
                }
                if operation == ManpowerOperation::Reserve {
                    next.reserved = next
                        .reserved
                        .checked_add(amount)
                        .ok_or(EconomyError::Overflow)?;
                } else {
                    next.committed = next
                        .committed
                        .checked_add(amount)
                        .ok_or(EconomyError::Overflow)?;
                }
            }
            ManpowerOperation::CommitReservation => {
                if amount > next.reserved {
                    return Err(EconomyError::InvalidValue);
                }
                next.reserved -= amount;
                next.committed = next
                    .committed
                    .checked_add(amount)
                    .ok_or(EconomyError::Overflow)?;
            }
            ManpowerOperation::CancelReservation => {
                if amount > next.reserved {
                    return Err(EconomyError::InvalidValue);
                }
                next.reserved -= amount;
            }
            ManpowerOperation::ReturnCommitted => {
                if amount > next.committed {
                    return Err(EconomyError::InvalidValue);
                }
                next.committed -= amount;
            }
        }
        next.committed
            .checked_add(next.reserved)
            .ok_or(EconomyError::Overflow)?;
        self.nations.insert(nation.0, next);
        Ok(())
    }
    pub(crate) fn validate_action(
        &self,
        world: &World,
        nation: NationId,
        action: &Action,
    ) -> Result<(), EconomyError> {
        let d = defs(world)?;
        if self.nation(nation).is_none() {
            return Err(EconomyError::InvalidReference);
        }
        match action {
            Action::Allocate { ratios } => {
                formula::economy_allocate(Qty::ZERO, *ratios, Fx::ZERO)?;
            }
            Action::Construct {
                state, building, ..
            } => {
                if world.state(StateId(*state)).is_none() || !d.buildings.contains_key(building) {
                    return Err(EconomyError::InvalidReference);
                }
            }
            Action::ChangeLaw { law } => {
                if !d.laws.contains_key(law) {
                    return Err(EconomyError::InvalidReference);
                }
            }
            Action::Reorder { projects } => {
                if projects.iter().collect::<BTreeSet<_>>().len() != projects.len() {
                    return Err(EconomyError::InvalidValue);
                }
            }
            Action::Cancel { .. } => {}
        }
        Ok(())
    }
    pub(crate) fn validate(&self, world: &World, tick: u64) -> Result<(), EconomyError> {
        let d = defs(world)?;
        if self.definitions_hash
            != oh_core::state_hash(d).map_err(|_| EconomyError::InvalidValue)?
            || self.nations.keys().ne(d.nations.keys())
        {
            return Err(EconomyError::InvalidReference);
        }
        // Current occupied buildings and historical daily contributions have
        // separate lifetimes. Do not count retained/dormant project reservations here.
        if !d.is_empty() {
            for state in world.inputs().states() {
                let limit = d
                    .state_slots
                    .get(&state.id().0)
                    .ok_or(EconomyError::InvalidReference)?;
                if slots(world, d, state.id().0)? > *limit {
                    return Err(EconomyError::SlotCap);
                }
            }
        }
        for (id, n) in &self.nations {
            let (_, capacity) = Self::derive(world, *id, n, tick)?;
            if n.capacity != capacity {
                return Err(EconomyError::InvalidValue);
            }
            let mut ids = BTreeSet::new();
            let mut targets = BTreeSet::new();
            for p in &n.projects {
                let b = d
                    .buildings
                    .get(&p.building)
                    .ok_or(EconomyError::InvalidReference)?;
                let stage = target_stage(p.target, b)?;
                let cost = qty(b.costs.get(stage).ok_or(EconomyError::InvalidReference)?)?;
                if world.state(StateId(p.state)).is_none()
                    || p.progress < Qty::ZERO
                    || p.progress >= cost
                    || !ids.insert(p.id)
                    || !targets.insert((p.state, &p.building))
                {
                    return Err(EconomyError::InvalidValue);
                }
            }
            let l = &n.ledger;
            if l.laws.keys().ne(n.laws.keys()) || l.stability < Fx::ZERO || l.stability > Fx::ONE {
                return Err(EconomyError::InvalidValue);
            }
            for (category, id) in &l.laws {
                if d.laws.get(id).is_none_or(|law| &law.category != category) {
                    return Err(EconomyError::InvalidReference);
                }
            }
            let law = &d.laws[&l.laws[&d.economy_category]];
            if l.minimum
                != formula::economy_minimum(
                    fx(&law.consumer_base)?,
                    fx(&law.instability_slope)?,
                    l.stability,
                )?
            {
                return Err(EconomyError::InvalidValue);
            }
            let conscription = &d.laws[&l.laws[&d.conscription_category]];
            if l.capacity
                != formula::economy_capacity(l.population, fx(&conscription.conscription_ratio)?)?
            {
                return Err(EconomyError::InvalidValue);
            }
            if l.multipliers
                .iter()
                .map(|v| v.source.as_str())
                .ne(std::iter::once(format!("nation:{id}"))
                    .chain(l.laws.values().map(|id| format!("law:{id}")))
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|v| v.as_str()))
            {
                return Err(EconomyError::InvalidReference);
            }

            if l.tick > tick || (l.tick != 0 && !l.tick.is_multiple_of(formula::HOURS_PER_DAY)) {
                return Err(EconomyError::InvalidValue);
            }
            let mut total = Qty::ZERO;
            let mut contributions = BTreeSet::new();
            for c in &l.contributions {
                let b = d
                    .buildings
                    .get(&c.building)
                    .ok_or(EconomyError::InvalidReference)?;
                if !l.population_by_state.contains_key(&c.state)
                    || world.state(StateId(c.state)).is_none()
                    || c.unit_ic != qty(&b.ic_per_level)?
                    || c.levels < 0
                    || usize::try_from(c.levels).map_err(|_| EconomyError::InvalidValue)?
                        > b.costs.len()
                    || !contributions.insert((c.state, &c.building))
                    || c.value != units(c.unit_ic, c.levels)?
                {
                    return Err(EconomyError::InvalidValue);
                }
                total = sum(total, c.value)?;
            }
            if !crate::save_state::strictly_sorted(
                l.contributions.iter().map(|c| (c.state, &c.building)),
            ) {
                return Err(EconomyError::InvalidValue);
            }
            if l.population_by_state.keys().ne(l.resources_by_state.keys()) {
                return Err(EconomyError::InvalidValue);
            }
            let mut population = 0i64;
            let mut resources = BTreeMap::new();
            for (state, value) in &l.population_by_state {
                if *value < 0 || world.state(StateId(*state)).is_none() {
                    return Err(EconomyError::InvalidValue);
                }
                population = population
                    .checked_add(*value)
                    .ok_or(EconomyError::Overflow)?;
            }
            for values in l.resources_by_state.values() {
                for (key, value) in values {
                    if *value < 0 || !world.defs().map().resource_ids().contains(key) {
                        return Err(EconomyError::InvalidReference);
                    }
                    let entry = resources.entry(key.clone()).or_insert(0i64);
                    *entry = entry.checked_add(*value).ok_or(EconomyError::Overflow)?;
                }
            }
            if population != l.population || resources != l.resources {
                return Err(EconomyError::InvalidValue);
            }
            for (index, m) in l.multipliers.iter().enumerate() {
                let expected = if index == 0 {
                    if m.source != format!("nation:{id}") {
                        return Err(EconomyError::InvalidReference);
                    }
                    fx(&d.nations[id].ic_multiplier)?
                } else {
                    let law = m
                        .source
                        .strip_prefix("law:")
                        .and_then(|id| d.laws.get(id))
                        .ok_or(EconomyError::InvalidReference)?;
                    fx(&law.ic_multiplier)?
                };
                if m.factor != expected {
                    return Err(EconomyError::InvalidValue);
                }
                total = formula::quantity_times_ratio(total, m.factor)?;
                if m.applied != total {
                    return Err(EconomyError::InvalidValue);
                }
            }
            if l.multipliers.len() != n.laws.len() + 1
                || total != l.total_ic
                || formula::economy_allocate(total, l.ratios, l.minimum)? != l.allocation
                || l.consumer_residual
                    != l.allocation[0] - formula::quantity_times_ratio(total, l.ratios[0])?
            {
                return Err(EconomyError::InvalidValue);
            }
            let c = &n.construction;
            if c.tick != l.tick || c.budget != l.allocation[1] || c.unused < Qty::ZERO {
                return Err(EconomyError::InvalidValue);
            }
            let mut consumed = Qty::ZERO;
            let mut project_ids = BTreeSet::new();
            for e in &c.entries {
                let b = d
                    .buildings
                    .get(&e.building)
                    .ok_or(EconomyError::InvalidReference)?;
                let stage = target_stage(e.target, b)?;
                if world.state(StateId(e.state)).is_none()
                    || world.nation(NationId(e.owner)).is_none()
                    || e.cost != qty(b.costs.get(stage).ok_or(EconomyError::InvalidReference)?)?
                    || e.daily_cap != qty(&b.daily_cap)?
                    || e.starting_progress < Qty::ZERO
                    || e.starting_progress >= e.cost
                    || e.consumed > e.daily_cap
                {
                    return Err(EconomyError::InvalidValue);
                }
                if e.factor_evaluated {
                    let factor = b
                        .infrastructure_factors
                        .iter()
                        .find(|v| fx(&v.infrastructure).ok() == Some(e.infrastructure))
                        .ok_or(EconomyError::InvalidReference)?;
                    if e.factor != fx(&factor.factor)? {
                        return Err(EconomyError::InvalidValue);
                    }
                } else if e.factor != Fx::ZERO || e.dormancy.is_none() {
                    return Err(EconomyError::InvalidValue);
                }
                if e.completed
                    != (e
                        .starting_progress
                        .checked_add(e.applied)
                        .ok_or(EconomyError::Overflow)?
                        == e.cost)
                    || e.starting_progress
                        .checked_add(e.applied)
                        .ok_or(EconomyError::Overflow)?
                        > e.cost
                {
                    return Err(EconomyError::InvalidValue);
                }
                if e.factor < Fx::ZERO
                    || e.consumed < Qty::ZERO
                    || e.applied < Qty::ZERO
                    || e.discarded < Qty::ZERO
                    || !project_ids.insert(e.project)
                    || sum(e.applied, e.discarded)?
                        != formula::quantity_times_ratio(e.consumed, e.factor)?
                    || e.dormancy.is_some() && (e.consumed != Qty::ZERO || e.completed)
                {
                    return Err(EconomyError::InvalidValue);
                }
                consumed = sum(consumed, e.consumed)?;
            }
            if sum(consumed, c.unused)? != c.budget {
                return Err(EconomyError::InvalidValue);
            }
        }
        Ok(())
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
                allocation[0]?,
                allocation[1]?,
                allocation[2]?,
                allocation[3]?,
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
                    laws: input.laws.clone(),
                    stability: fx(&input.stability)?,
                    capacity: 0,
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
            scores: None,
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
            laws: n.laws.clone(),
            stability: n.stability,
            capacity: 0,
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
        ledger.capacity = capacity;
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
                for p in next.projects.iter().filter(|p| {
                    p.state == *state
                        && s.buildings()
                            .get(&p.building)
                            .copied()
                            .unwrap_or(0)
                            .checked_add(1)
                            == Some(p.target)
                }) {
                    let b = d
                        .buildings
                        .get(&p.building)
                        .ok_or(EconomyError::InvalidReference)?;
                    let i = target_stage(p.target, b)?;
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
            .map(|s| {
                Ok((
                    s.id().0,
                    formula::stat_value(s.base(), "infrastructure", s.modifiers(), tick)
                        .map_err(|_| EconomyError::InvalidValue)?,
                ))
            })
            .collect::<Result<_, EconomyError>>()?;
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
                let stage = target_stage(p.target, b)?;
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
                    for other in n.projects.iter().filter(|p| {
                        p.state == s.id().0
                            && s.buildings()
                                .get(&p.building)
                                .copied()
                                .unwrap_or(0)
                                .checked_add(1)
                                == Some(p.target)
                            && !ledger
                                .entries
                                .iter()
                                .any(|e| e.project == p.id && e.completed)
                    }) {
                        let b = d
                            .buildings
                            .get(&other.building)
                            .ok_or(EconomyError::InvalidReference)?;
                        let stage = target_stage(other.target, b)?;
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
                let owner = s.owner().0;
                let mut factor = Fx::ZERO;
                let factor_evaluated = reason.is_none();
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
                    state: p.state,
                    building: p.building.clone(),
                    target: p.target,
                    starting_progress: p.progress,
                    cost,
                    daily_cap: cap,
                    infrastructure: infrastructure[&p.state],
                    owner,
                    factor_evaluated,
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

#[cfg(test)]
#[path = "../tests/support/economy_fixture.rs"]
mod test_fixture;
#[cfg(test)]
mod tests {
    use super::*;
    fn target_candidate(
        target: i64,
        historical: bool,
    ) -> (
        crate::economy_save::SimulationSaveV5,
        crate::save_state::RestoreContext,
    ) {
        let l = test_fixture::loaded();
        let world = World::from_loaded(&l).unwrap();
        let date = crate::Date::new(2000, 1, 1).unwrap();
        let mut sim = crate::Simulation::with_world(
            "m1".into(),
            date,
            1000,
            crate::TimeConfig::from_defines(&l.pack.defines).unwrap(),
            world.clone(),
        )
        .unwrap();
        sim.enqueue(
            0,
            NationId(1),
            1,
            crate::Command::Economy(Action::Construct {
                project: 91,
                state: 1,
                building: "industry".into(),
            }),
        )
        .unwrap();
        for _ in 0..24 {
            sim.step().unwrap();
        }
        let mut dto = sim.export_save_v5().unwrap();
        let nation = dto.economy.nations.get_mut(&1).unwrap();
        if historical {
            nation.construction.entries[0].target = target;
        } else {
            nation.projects[0].target = target;
        }
        (
            dto,
            crate::save_state::RestoreContext {
                scenario: "m1".into(),
                start_date: date,
                world: Some(world),
            },
        )
    }
    macro_rules! invalid_target {
        ($name:ident,$value:expr,$historical:expr) => {
            #[test]
            fn $name() {
                let (dto, context) = target_candidate($value, $historical);
                let before = oh_core::canonical_bytes(context.world.as_ref().unwrap()).unwrap();
                let error = crate::Simulation::from_save_v5(dto, &context)
                    .err()
                    .expect("accepted invalid target");
                assert_eq!(error, "economy:InvalidValue");
                assert_eq!(
                    oh_core::canonical_bytes(context.world.as_ref().unwrap()).unwrap(),
                    before
                );
            }
        };
    }
    invalid_target!(project_target_zero, 0, false);
    invalid_target!(project_target_negative, -1, false);
    invalid_target!(project_target_minimum, i64::MIN, false);
    invalid_target!(project_target_maximum, i64::MAX, false);
    invalid_target!(ledger_target_zero, 0, true);
    invalid_target!(ledger_target_negative, -1, true);
    invalid_target!(ledger_target_minimum, i64::MIN, true);
    invalid_target!(ledger_target_maximum, i64::MAX, true);
    #[test]
    fn valid_target_one_and_maximum_are_not_normalized_to_current_plus_one() {
        for target in [1, 3] {
            let (dto, context) = target_candidate(target, false);
            let restored = crate::Simulation::from_save_v5(dto.clone(), &context).unwrap();
            assert_eq!(restored.export_save_v5().unwrap(), dto);
            assert_eq!(
                restored
                    .economy()
                    .unwrap()
                    .nation(NationId(1))
                    .unwrap()
                    .projects()[0]
                    .target,
                target
            );
        }
    }
    #[test]
    fn historical_valid_first_and_last_stage_keep_their_own_cost_snapshot() {
        for (target, index) in [(1, 0), (3, 2)] {
            let (mut dto, context) = target_candidate(target, true);
            let definition = &context
                .world
                .as_ref()
                .unwrap()
                .defs()
                .economy()
                .unwrap()
                .buildings["industry"];
            dto.economy
                .nations
                .get_mut(&1)
                .unwrap()
                .construction
                .entries[0]
                .cost = qty(&definition.costs[index]).unwrap();
            let restored = crate::Simulation::from_save_v5(dto.clone(), &context).unwrap();
            assert_eq!(restored.export_save_v5().unwrap(), dto);
        }
    }
    fn assert_authoritative_restore(world: &World, economy: &Economy, tick: u64) {
        let loaded = test_fixture::loaded();
        let start = crate::Date::new(2000, 1, 1).unwrap();
        let template = World::from_loaded(&loaded).unwrap();
        let context = crate::save_state::RestoreContext {
            scenario: "m1".into(),
            start_date: start,
            world: Some(template.clone()),
        };
        let mut sim = crate::Simulation::with_world(
            "m1".into(),
            start,
            1,
            crate::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
            template,
        )
        .unwrap();
        for _ in 0..tick {
            sim.step().unwrap();
        }
        let mut current = world.clone();
        current.evaluate(tick).unwrap();
        sim.world = Some(current);
        sim.economy = Some(economy.clone());
        let dto = sim.export_save_v5().unwrap();
        let restored = crate::Simulation::from_save_v5(dto.clone(), &context).unwrap();
        assert_eq!(restored.export_save_v5().unwrap(), dto);
        assert_eq!(
            oh_core::canonical_bytes(&sim).unwrap(),
            oh_core::canonical_bytes(&restored).unwrap()
        );
        assert_eq!(sim.state_hash().unwrap(), restored.state_hash().unwrap());
    }
    fn transfer(world: &World, state: u16, owner: u16) -> World {
        let mut dto = world.export_save();
        dto.inputs
            .states
            .iter_mut()
            .find(|s| s.id == state)
            .unwrap()
            .owner = owner;
        for p in &mut dto.inputs.provinces {
            if p.state == Some(state) {
                p.owner = Some(owner);
            }
        }
        World::from_save(dto, world, 0).unwrap()
    }
    #[test]
    fn req_eco_06_ownership_loss_recovery_and_new_owner_target_conflict() {
        let l = test_fixture::loaded();
        let mut world = World::from_loaded(&l).unwrap();
        let mut e = Economy::initial(&world, 0).unwrap();
        e.command(
            &world,
            NationId(1),
            &Action::Construct {
                project: 7,
                state: 1,
                building: "industry".into(),
            },
            0,
            true,
        )
        .unwrap();
        e.nations.get_mut(&1).unwrap().projects[0].progress = Qty::from_num(2);
        world = transfer(&world, 1, 2);
        e.daily_economy(&world, 24).unwrap();
        e.daily_construction(&mut world, 24).unwrap();
        assert_authoritative_restore(&world, &e, 24);
        assert_eq!(
            e.nation(NationId(1)).unwrap().projects[0].dormancy,
            Some(Dormancy::NoOwnership)
        );
        assert_eq!(
            e.nation(NationId(1)).unwrap().projects[0].progress,
            Qty::from_num(2)
        );
        let mut valid_world = transfer(&world, 1, 1);
        let mut valid = e.clone();
        valid.daily_economy(&valid_world, 48).unwrap();
        valid.daily_construction(&mut valid_world, 48).unwrap();
        assert_eq!(
            valid.nation(NationId(1)).unwrap().projects[0].progress,
            Qty::from_num(4.5)
        );
        assert_eq!(
            valid.nation(NationId(1)).unwrap().projects[0].dormancy,
            None
        );
        e.command(
            &world,
            NationId(2),
            &Action::Construct {
                project: 8,
                state: 1,
                building: "industry".into(),
            },
            24,
            true,
        )
        .unwrap();
        e.nations.get_mut(&2).unwrap().projects[0].progress = Qty::from_num(19);
        e.daily_economy(&world, 48).unwrap();
        e.daily_construction(&mut world, 48).unwrap();
        assert_authoritative_restore(&world, &e, 48);
        assert_eq!(world.state(StateId(1)).unwrap().buildings()["industry"], 2);
        world = transfer(&world, 1, 1);
        world = transfer(&world, 2, 1);
        e.command(
            &world,
            NationId(1),
            &Action::Construct {
                project: 9,
                state: 2,
                building: "industry".into(),
            },
            48,
            true,
        )
        .unwrap();
        e.daily_economy(&world, 72).unwrap();
        e.daily_construction(&mut world, 72).unwrap();
        assert_authoritative_restore(&world, &e, 72);
        let n = e.nation(NationId(1)).unwrap();
        assert_eq!(n.projects.len(), 1);
        assert_eq!(n.projects[0].id, 7);
        assert_eq!(n.projects[0].progress, Qty::from_num(2));
        assert_eq!(n.projects[0].dormancy, Some(Dormancy::TargetConflict));
        assert_eq!(world.state(StateId(1)).unwrap().buildings()["industry"], 2);
        assert_eq!(world.state(StateId(2)).unwrap().buildings()["industry"], 1);
        assert!(
            n.construction
                .entries
                .iter()
                .find(|v| v.project == 9)
                .unwrap()
                .completed
        );
    }
    #[test]
    fn req_eco_06_zero_factor_skip_and_slot_limit_do_not_consume() {
        let mut l = test_fixture::loaded();
        l.scenario.ownership.insert(2, "NTH".into());
        l.economy
            .as_mut()
            .unwrap()
            .buildings
            .get_mut("industry")
            .unwrap()
            .infrastructure_factors
            .iter_mut()
            .for_each(|f| {
                if f.infrastructure != "0" {
                    f.factor = "0".into();
                }
            });
        let mut world = World::from_loaded(&l).unwrap();
        let mut e = Economy::initial(&world, 0).unwrap();
        for (id, state) in [(1, 1), (2, 2)] {
            e.command(
                &world,
                NationId(1),
                &Action::Construct {
                    project: id,
                    state,
                    building: "industry".into(),
                },
                0,
                true,
            )
            .unwrap();
        }
        e.daily_economy(&world, 24).unwrap();
        e.daily_construction(&mut world, 24).unwrap();
        let n = e.nation(NationId(1)).unwrap();
        assert_eq!(n.projects[0].dormancy, Some(Dormancy::ZeroFactor));
        assert_eq!(n.projects[0].progress, Qty::ZERO);
        assert_eq!(n.projects[1].progress, Qty::from_num(2.5));
        assert_eq!(n.construction.entries[0].consumed, Qty::ZERO);
        let mut l = test_fixture::loaded();
        l.economy.as_mut().unwrap().state_slots.insert(1, 1);
        let world = World::from_loaded(&l).unwrap();
        let mut e = Economy::initial(&world, 0).unwrap();
        let before = e.clone();
        assert_eq!(
            e.command(
                &world,
                NationId(1),
                &Action::Construct {
                    project: 3,
                    state: 1,
                    building: "industry".into()
                },
                0,
                true
            ),
            Err(EconomyError::SlotCap)
        );
        assert_eq!(e, before);
    }
}
