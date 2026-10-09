//! Authoritative training ownership and roster. No combat or movement effects.
use crate::{
    economy::{Economy, ManpowerOperation},
    military_templates::{Normal, aggregate},
    production::Production,
    world::World,
};
use oh_core::{DivisionId, NationId, ProvinceId};
use oh_data::military::Background;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MilitaryError {
    MissingContext,
    InvalidReference,
    InvalidValue,
    Overflow,
    NotOwner,
    NotReady,
    Terminal,
    ArmyFull,
    Understrength,
    AccountingMismatch,
}
impl std::fmt::Display for MilitaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "military:{self:?}")
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Action {
    Train {
        template: String,
    },
    Cancel {
        job: u64,
    },
    Deploy {
        job: u64,
        army: u64,
        province: ProvinceId,
        allow_understrength: bool,
    },
    SetPriority {
        army: u64,
        priority: u16,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Army {
    id: u64,
    nation: u16,
    general: String,
    division_limit: u32,
    priority: u16,
}
impl Army {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn nation(&self) -> NationId {
        NationId(self.nation)
    }
    pub fn general(&self) -> &str {
        &self.general
    }
    pub fn division_limit(&self) -> u32 {
        self.division_limit
    }
    pub fn priority(&self) -> u16 {
        self.priority
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Division {
    id: u32,
    nation: u16,
    army: u64,
    province: u16,
    template: String,
    normal: Normal,
    manpower: i64,
    equipment: BTreeMap<String, i64>,
}
impl Division {
    pub fn id(&self) -> DivisionId {
        DivisionId(self.id)
    }
    pub fn nation(&self) -> NationId {
        NationId(self.nation)
    }
    pub fn army(&self) -> u64 {
        self.army
    }
    pub fn province(&self) -> ProvinceId {
        ProvinceId(self.province)
    }
    pub fn template(&self) -> &str {
        &self.template
    }
    pub fn normal(&self) -> &Normal {
        &self.normal
    }
    pub fn manpower(&self) -> i64 {
        self.manpower
    }
    pub fn equipment(&self) -> &BTreeMap<String, i64> {
        &self.equipment
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Training,
    Ready,
    Cancelled,
    Deployed,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainingJob {
    id: u64,
    nation: u16,
    template: String,
    normal: Normal,
    training_days: u32,
    status: JobStatus,
    progress_days: u32,
    start_tick: Option<u64>,
    reserved_manpower: i64,
    equipment: BTreeMap<String, i64>,
    division: Option<u32>,
}
impl TrainingJob {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn nation(&self) -> NationId {
        NationId(self.nation)
    }
    pub fn template(&self) -> &str {
        &self.template
    }
    pub fn normal(&self) -> &Normal {
        &self.normal
    }
    pub fn training_days(&self) -> u32 {
        self.training_days
    }
    pub fn status(&self) -> JobStatus {
        self.status
    }
    pub fn progress_days(&self) -> u32 {
        self.progress_days
    }
    pub fn start_tick(&self) -> Option<u64> {
        self.start_tick
    }
    pub fn reserved_manpower(&self) -> i64 {
        self.reserved_manpower
    }
    pub fn equipment(&self) -> &BTreeMap<String, i64> {
        &self.equipment
    }
    pub fn division(&self) -> Option<DivisionId> {
        self.division.map(DivisionId)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Military {
    definitions_hash: u64,
    next_job_id: u64,
    next_division_id: u32,
    armies: BTreeMap<u64, Army>,
    divisions: BTreeMap<u32, Division>,
    jobs: BTreeMap<u64, TrainingJob>,
    background: BTreeMap<u16, Background>,
}
fn err<T>(v: Result<T, impl std::fmt::Display>) -> Result<T, MilitaryError> {
    v.map_err(|_| MilitaryError::InvalidValue)
}
fn defs(w: &World) -> Result<&oh_data::military::Definition, MilitaryError> {
    w.defs().military().ok_or(MilitaryError::MissingContext)
}
/// Whole people/items; zero deficits receive zero. ID is the stable tie break.
pub fn proportional(
    available: i64,
    deficits: &BTreeMap<u64, i64>,
) -> Result<BTreeMap<u64, i64>, MilitaryError> {
    if available < 0 || deficits.values().any(|v| *v < 0) {
        return Err(MilitaryError::InvalidValue);
    }
    let total = deficits.values().try_fold(0i128, |s, v| {
        s.checked_add(i128::from(*v)).ok_or(MilitaryError::Overflow)
    })?;
    let budget = i128::from(available).min(total);
    let mut result = BTreeMap::new();
    let mut remainders = Vec::new();
    let mut spent = 0;
    for (&id, &v) in deficits {
        let numerator = budget
            .checked_mul(i128::from(v))
            .ok_or(MilitaryError::Overflow)?;
        let q = if total == 0 { 0 } else { numerator / total };
        let r = if total == 0 { 0 } else { numerator % total };
        result.insert(id, q as i64);
        spent += q;
        remainders.push((r, id));
    }
    remainders.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (_, id) in remainders.into_iter().take((budget - spent) as usize) {
        *result.get_mut(&id).unwrap() += 1;
    }
    Ok(result)
}
impl Military {
    pub fn definitions_hash(&self) -> u64 {
        self.definitions_hash
    }
    pub fn next_job_id(&self) -> u64 {
        self.next_job_id
    }
    pub fn next_division_id(&self) -> u32 {
        self.next_division_id
    }
    pub fn armies(&self) -> &BTreeMap<u64, Army> {
        &self.armies
    }
    pub fn divisions(&self) -> &BTreeMap<u32, Division> {
        &self.divisions
    }
    pub fn jobs(&self) -> &BTreeMap<u64, TrainingJob> {
        &self.jobs
    }
    pub fn background(&self) -> &BTreeMap<u16, Background> {
        &self.background
    }
    pub fn initial(w: &World, e: &Economy, p: &Production) -> Result<Self, MilitaryError> {
        let d = defs(w)?;
        let armies = d
            .armies
            .iter()
            .map(|(&id, a)| {
                (
                    id,
                    Army {
                        id,
                        nation: a.nation,
                        general: a.general.clone(),
                        division_limit: a.capacity,
                        priority: a.priority,
                    },
                )
            })
            .collect();
        let mut divisions = BTreeMap::new();
        for (&id, v) in &d.divisions {
            let a = &d.armies[&v.army];
            divisions.insert(
                id,
                Division {
                    id,
                    nation: a.nation,
                    army: v.army,
                    province: v.province,
                    template: v.template.clone(),
                    normal: err(aggregate(d.templates(), &v.template))?,
                    manpower: v.manpower,
                    equipment: v.equipment.clone(),
                },
            );
        }
        let next_division_id = divisions.last_key_value().map_or(Ok(0), |(&id, _)| {
            id.checked_add(1).ok_or(MilitaryError::Overflow)
        })?;
        let state = Self {
            definitions_hash: err(d.identity())?,
            next_job_id: 0,
            next_division_id,
            armies,
            divisions,
            jobs: BTreeMap::new(),
            background: d.background.clone(),
        };
        state.validate(w, e, p, 0)?;
        Ok(state)
    }
    pub fn returnable_holdings(
        &self,
    ) -> Result<BTreeMap<u16, BTreeMap<String, i64>>, MilitaryError> {
        let mut held: BTreeMap<u16, BTreeMap<String, i64>> = BTreeMap::new();
        for j in self.jobs.values() {
            for (m, v) in &j.equipment {
                let n = held
                    .entry(j.nation)
                    .or_default()
                    .entry(m.clone())
                    .or_default();
                *n = n.checked_add(*v).ok_or(MilitaryError::Overflow)?;
            }
        }
        Ok(held)
    }
    pub fn validate_pending(
        &self,
        w: &World,
        n: NationId,
        a: &Action,
    ) -> Result<(), MilitaryError> {
        if w.nation(n).is_none() {
            return Err(MilitaryError::InvalidReference);
        }
        match a {
            Action::Train { template } => {
                if !defs(w)?.training_days.contains_key(template) {
                    return Err(MilitaryError::InvalidReference);
                }
            }
            Action::Cancel { job } | Action::Deploy { job, .. } => {
                let j = self.jobs.get(job).ok_or(MilitaryError::InvalidReference)?;
                if j.nation != n.0 {
                    return Err(MilitaryError::NotOwner);
                }
            }
            Action::SetPriority { army, .. } => {
                self.owned_army(n, *army)?;
            }
        }
        if let Action::Deploy { army, province, .. } = a {
            self.owned_army(n, *army)?;
            if w.province(*province).is_none() {
                return Err(MilitaryError::InvalidReference);
            }
        }
        Ok(())
    }
    fn owned_army(&self, n: NationId, id: u64) -> Result<&Army, MilitaryError> {
        let a = self
            .armies
            .get(&id)
            .ok_or(MilitaryError::InvalidReference)?;
        if a.nation != n.0 {
            return Err(MilitaryError::NotOwner);
        }
        Ok(a)
    }
    pub(crate) fn command(
        &mut self,
        w: &World,
        e: &mut Economy,
        p: &mut Production,
        n: NationId,
        a: &Action,
        tick: u64,
    ) -> Result<(), MilitaryError> {
        let mut next = self.clone();
        let mut ne = e.clone();
        let mut np = p.clone();
        next.apply(w, &mut ne, &mut np, n, a)?;
        next.validate(w, &ne, &np, tick)?;
        *self = next;
        *e = ne;
        *p = np;
        Ok(())
    }
    fn apply(
        &mut self,
        w: &World,
        e: &mut Economy,
        p: &mut Production,
        n: NationId,
        a: &Action,
    ) -> Result<(), MilitaryError> {
        self.validate_pending(w, n, a)?;
        match a {
            Action::Train { template } => {
                if self.jobs.len() >= oh_data::military_templates::MAX_ENTRIES {
                    return Err(MilitaryError::InvalidValue);
                }
                let normal = err(aggregate(defs(w)?.templates(), template))?;
                let id = self.next_job_id;
                self.next_job_id = id.checked_add(1).ok_or(MilitaryError::Overflow)?;
                self.jobs.insert(
                    id,
                    TrainingJob {
                        id,
                        nation: n.0,
                        template: template.clone(),
                        normal,
                        training_days: defs(w)?.training_days[template],
                        status: JobStatus::Pending,
                        progress_days: 0,
                        start_tick: None,
                        reserved_manpower: 0,
                        equipment: BTreeMap::new(),
                        division: None,
                    },
                );
            }
            Action::SetPriority { army, priority } => {
                self.armies.get_mut(army).unwrap().priority = *priority;
            }
            Action::Cancel { job } => {
                let j = self.jobs.get_mut(job).unwrap();
                if matches!(j.status, JobStatus::Cancelled | JobStatus::Deployed) {
                    return Err(MilitaryError::Terminal);
                }
                err(e.manpower(n, ManpowerOperation::CancelReservation, j.reserved_manpower))?;
                for (m, v) in &j.equipment {
                    err(p.adjust_stock(w, n, m, *v))?
                }
                j.reserved_manpower = 0;
                j.equipment.clear();
                j.progress_days = 0;
                j.start_tick = None;
                j.status = JobStatus::Cancelled;
            }
            Action::Deploy {
                job,
                army,
                province,
                allow_understrength,
            } => {
                let j = &self.jobs[job];
                if matches!(j.status, JobStatus::Cancelled | JobStatus::Deployed) {
                    return Err(MilitaryError::Terminal);
                }
                if j.status != JobStatus::Ready {
                    return Err(MilitaryError::NotReady);
                }
                let a = self.owned_army(n, *army)?;
                if self.divisions.values().filter(|d| d.army == *army).count()
                    >= a.division_limit as usize
                {
                    return Err(MilitaryError::ArmyFull);
                }
                let province_state = w
                    .province(*province)
                    .ok_or(MilitaryError::InvalidReference)?;
                if province_state.controller() != Some(n) || province_state.state().is_none() {
                    return Err(MilitaryError::NotOwner);
                }
                if !allow_understrength
                    && j.normal
                        .equipment
                        .iter()
                        .any(|(m, v)| j.equipment.get(m).copied().unwrap_or(0) < *v)
                {
                    return Err(MilitaryError::Understrength);
                }
                let id = self.next_division_id;
                self.next_division_id = id.checked_add(1).ok_or(MilitaryError::Overflow)?;
                err(e.manpower(n, ManpowerOperation::CommitReservation, j.reserved_manpower))?;
                self.divisions.insert(
                    id,
                    Division {
                        id,
                        nation: n.0,
                        army: *army,
                        province: province.0,
                        template: j.template.clone(),
                        normal: j.normal.clone(),
                        manpower: j.reserved_manpower,
                        equipment: j.equipment.clone(),
                    },
                );
                let j = self.jobs.get_mut(job).unwrap();
                j.reserved_manpower = 0;
                j.equipment.clear();
                j.status = JobStatus::Deployed;
                j.division = Some(id);
            }
        }
        Ok(())
    }
    pub(crate) fn daily(
        &mut self,
        w: &World,
        e: &mut Economy,
        p: &mut Production,
        tick: u64,
    ) -> Result<(), MilitaryError> {
        let mut next = self.clone();
        let mut ne = e.clone();
        let mut np = p.clone();
        next.run_daily(w, &mut ne, &mut np, tick)?;
        next.validate(w, &ne, &np, tick)?;
        *self = next;
        *e = ne;
        *p = np;
        Ok(())
    }
    fn run_daily(
        &mut self,
        w: &World,
        e: &mut Economy,
        p: &mut Production,
        tick: u64,
    ) -> Result<(), MilitaryError> {
        for n in self.background.keys().copied().collect::<Vec<_>>() {
            let priorities = self
                .armies
                .values()
                .filter(|a| a.nation == n)
                .map(|a| a.priority)
                .collect::<BTreeSet<_>>();
            for priority in priorities {
                let ids = self
                    .divisions
                    .iter()
                    .filter(|(_, d)| d.nation == n && self.armies[&d.army].priority == priority)
                    .map(|(&id, _)| id)
                    .collect::<Vec<_>>();
                let deficits = ids
                    .iter()
                    .map(|id| {
                        (
                            u64::from(*id),
                            self.divisions[id].normal.manpower - self.divisions[id].manpower,
                        )
                    })
                    .collect();
                let alloc = proportional(
                    e.nation(NationId(n))
                        .ok_or(MilitaryError::MissingContext)?
                        .available(),
                    &deficits,
                )?;
                for (id, v) in alloc {
                    err(e.manpower(NationId(n), ManpowerOperation::Consume, v))?;
                    self.divisions.get_mut(&(id as u32)).unwrap().manpower += v;
                }
                self.allocate_division_equipment(w, p, n, &ids)?;
            }
            let ids = self
                .jobs
                .iter()
                .filter(|(_, j)| j.nation == n)
                .map(|(&id, _)| id)
                .collect::<Vec<_>>();
            for id in &ids {
                let j = self.jobs.get_mut(id).unwrap();
                if j.status == JobStatus::Pending
                    && e.nation(NationId(n)).unwrap().available() >= j.normal.manpower
                {
                    err(e.manpower(NationId(n), ManpowerOperation::Reserve, j.normal.manpower))?;
                    j.reserved_manpower = j.normal.manpower;
                    j.start_tick = Some(tick);
                    j.status = JobStatus::Training;
                } else if j.status == JobStatus::Training {
                    j.progress_days = j
                        .progress_days
                        .checked_add(1)
                        .ok_or(MilitaryError::Overflow)?;
                    if j.progress_days == j.training_days {
                        j.status = JobStatus::Ready;
                    }
                }
            }
            let models = ids
                .iter()
                .filter_map(|id| {
                    let j = &self.jobs[id];
                    matches!(j.status, JobStatus::Training | JobStatus::Ready)
                        .then_some(j.normal.equipment.keys())
                })
                .flatten()
                .cloned()
                .collect::<BTreeSet<_>>();
            for model in models {
                let deficits = ids
                    .iter()
                    .filter_map(|id| {
                        let j = &self.jobs[id];
                        matches!(j.status, JobStatus::Training | JobStatus::Ready).then_some((
                            *id,
                            j.normal.equipment.get(&model).copied().unwrap_or(0)
                                - j.equipment.get(&model).copied().unwrap_or(0),
                        ))
                    })
                    .collect();
                let alloc = proportional(
                    p.stock(NationId(n), &model)
                        .ok_or(MilitaryError::InvalidReference)?,
                    &deficits,
                )?;
                for (id, v) in alloc {
                    if v == 0 && !self.jobs[&id].normal.equipment.contains_key(&model) {
                        continue;
                    }
                    err(p.adjust_stock(w, NationId(n), &model, -v))?;
                    *self
                        .jobs
                        .get_mut(&id)
                        .unwrap()
                        .equipment
                        .entry(model.clone())
                        .or_default() += v;
                }
            }
        }
        Ok(())
    }
    fn allocate_division_equipment(
        &mut self,
        w: &World,
        p: &mut Production,
        n: u16,
        ids: &[u32],
    ) -> Result<(), MilitaryError> {
        let models = ids
            .iter()
            .flat_map(|id| self.divisions[id].normal.equipment.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        for model in models {
            let deficits = ids
                .iter()
                .map(|id| {
                    let d = &self.divisions[id];
                    (
                        u64::from(*id),
                        d.normal.equipment.get(&model).copied().unwrap_or(0)
                            - d.equipment.get(&model).copied().unwrap_or(0),
                    )
                })
                .collect();
            let alloc = proportional(
                p.stock(NationId(n), &model)
                    .ok_or(MilitaryError::InvalidReference)?,
                &deficits,
            )?;
            for (id, v) in alloc {
                if v == 0
                    && !self.divisions[&(id as u32)]
                        .normal
                        .equipment
                        .contains_key(&model)
                {
                    continue;
                }
                err(p.adjust_stock(w, NationId(n), &model, -v))?;
                *self
                    .divisions
                    .get_mut(&(id as u32))
                    .unwrap()
                    .equipment
                    .entry(model.clone())
                    .or_default() += v;
            }
        }
        Ok(())
    }
    pub fn validate(
        &self,
        w: &World,
        e: &Economy,
        p: &Production,
        tick: u64,
    ) -> Result<(), MilitaryError> {
        let d = defs(w)?;
        if self.definitions_hash != err(d.identity())?
            || self.background != d.background
            || self.armies.len() != d.armies.len()
            || self.jobs.len() > oh_data::military_templates::MAX_ENTRIES
            || self.next_job_id != self.jobs.len() as u64
            || self.jobs.keys().copied().ne(0..self.next_job_id)
        {
            return Err(MilitaryError::InvalidValue);
        }
        for (&id, a) in &self.armies {
            let input = d.armies.get(&id).ok_or(MilitaryError::InvalidReference)?;
            if a.id != id
                || a.nation != input.nation
                || a.general != input.general
                || a.division_limit != input.capacity
                || self.divisions.values().filter(|v| v.army == id).count()
                    > a.division_limit as usize
            {
                return Err(MilitaryError::InvalidValue);
            }
        }
        let initial_next = d.divisions.last_key_value().map_or(Ok(0), |(&id, _)| {
            id.checked_add(1).ok_or(MilitaryError::Overflow)
        })?;
        if self.next_division_id < initial_next {
            return Err(MilitaryError::InvalidValue);
        }
        for (&id, input) in &d.divisions {
            let current = self
                .divisions
                .get(&id)
                .ok_or(MilitaryError::InvalidReference)?;
            if current.army != input.army
                || current.template != input.template
                || current.manpower < input.manpower
                || input
                    .equipment
                    .iter()
                    .any(|(m, v)| current.equipment.get(m).copied().unwrap_or(0) < *v)
            {
                return Err(MilitaryError::InvalidValue);
            }
        }
        let mut deployed = BTreeSet::new();
        let mut committed = self
            .background
            .iter()
            .map(|(n, b)| (*n, i128::from(b.committed)))
            .collect::<BTreeMap<_, _>>();
        let mut reserved = self
            .background
            .iter()
            .map(|(n, b)| (*n, i128::from(b.reserved)))
            .collect::<BTreeMap<_, _>>();
        let counts = |normal: &Normal,
                      manpower: i64,
                      gear: &BTreeMap<String, i64>|
         -> Result<(), MilitaryError> {
            if manpower < 0
                || manpower > normal.manpower
                || gear.iter().any(|(m, v)| {
                    *v < 0 || !normal.equipment.contains_key(m) || *v > normal.equipment[m]
                })
            {
                return Err(MilitaryError::InvalidValue);
            }
            Ok(())
        };
        for (&id, v) in &self.divisions {
            if id != v.id
                || id >= self.next_division_id
                || self.owned_army(NationId(v.nation), v.army).is_err()
                || w.province(ProvinceId(v.province))
                    .is_none_or(|v| v.state().is_none())
                || v.normal != err(aggregate(d.templates(), &v.template))?
            {
                return Err(MilitaryError::InvalidValue);
            }
            counts(&v.normal, v.manpower, &v.equipment)?;
            *committed
                .get_mut(&v.nation)
                .ok_or(MilitaryError::InvalidReference)? += i128::from(v.manpower);
        }
        for (&id, j) in &self.jobs {
            if id != j.id
                || id >= self.next_job_id
                || j.normal != err(aggregate(d.templates(), &j.template))?
                || d.training_days.get(&j.template) != Some(&j.training_days)
            {
                return Err(MilitaryError::InvalidValue);
            }
            counts(&j.normal, j.reserved_manpower, &j.equipment)?;
            match j.status {
                JobStatus::Pending | JobStatus::Cancelled => {
                    if j.reserved_manpower != 0
                        || !j.equipment.is_empty()
                        || j.progress_days != 0
                        || j.start_tick.is_some()
                        || j.division.is_some()
                    {
                        return Err(MilitaryError::InvalidValue);
                    }
                }
                JobStatus::Training | JobStatus::Ready => {
                    let start = j.start_tick.ok_or(MilitaryError::InvalidValue)?;
                    if start > tick
                        || start % 24 != 0
                        || j.reserved_manpower != j.normal.manpower
                        || j.division.is_some()
                        || u64::from(j.progress_days)
                            != ((tick - start) / 24).min(u64::from(j.training_days))
                    {
                        return Err(MilitaryError::InvalidValue);
                    }
                    if (j.status == JobStatus::Ready) != (j.progress_days == j.training_days) {
                        return Err(MilitaryError::InvalidValue);
                    }
                }
                JobStatus::Deployed => {
                    let v = self
                        .divisions
                        .get(&j.division.ok_or(MilitaryError::InvalidValue)?)
                        .ok_or(MilitaryError::InvalidReference)?;
                    if !deployed.insert(v.id)
                        || v.id < initial_next
                        || j.start_tick.is_none_or(|start| {
                            start > tick
                                || start % 24 != 0
                                || (tick - start) / 24 < u64::from(j.training_days)
                        })
                        || j.reserved_manpower != 0
                        || !j.equipment.is_empty()
                        || j.progress_days != j.training_days
                        || v.nation != j.nation
                        || v.template != j.template
                    {
                        return Err(MilitaryError::InvalidValue);
                    }
                }
            }
            *reserved
                .get_mut(&j.nation)
                .ok_or(MilitaryError::InvalidReference)? += i128::from(j.reserved_manpower);
        }
        if deployed
            .iter()
            .copied()
            .ne(initial_next..self.next_division_id)
            || self
                .divisions
                .keys()
                .filter(|id| **id >= initial_next)
                .copied()
                .ne(initial_next..self.next_division_id)
        {
            return Err(MilitaryError::InvalidValue);
        }
        for (&n, b) in &self.background {
            let a = e.nation(NationId(n)).ok_or(MilitaryError::MissingContext)?;
            if b.committed < 0
                || b.reserved < 0
                || committed[&n] != i128::from(a.committed())
                || reserved[&n] != i128::from(a.reserved())
            {
                return Err(MilitaryError::AccountingMismatch);
            }
        }
        let held = self.returnable_holdings()?;
        let limit = w
            .defs()
            .production()
            .and_then(|d| d.tuning.as_ref())
            .ok_or(MilitaryError::MissingContext)?
            .inventory_count_limit;
        for (n, models) in held {
            for (m, v) in models {
                if p.stock(NationId(n), &m)
                    .ok_or(MilitaryError::InvalidReference)?
                    .checked_add(v)
                    .ok_or(MilitaryError::Overflow)?
                    > limit
                {
                    return Err(MilitaryError::InvalidValue);
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approved_proportion_and_id_tie() {
        assert_eq!(
            proportional(4, &[(0, 6), (1, 2)].into()).unwrap(),
            [(0, 3), (1, 1)].into()
        );
        assert_eq!(
            proportional(1, &[(9, 1), (2, 1)].into()).unwrap(),
            [(2, 1), (9, 0)].into()
        );
        assert_eq!(
            proportional(i64::MAX, &[(0, i64::MAX), (1, i64::MAX)].into())
                .unwrap()
                .values()
                .map(|v| i128::from(*v))
                .sum::<i128>(),
            i128::from(i64::MAX)
        );
    }
}
