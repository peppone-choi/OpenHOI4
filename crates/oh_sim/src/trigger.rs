//! Pure condition evaluation and transactional effects, shared by real hosts.
use crate::{
    Date, State,
    save_state::{DateV1, strictly_sorted},
    world::World,
};
use oh_core::{NationId, ProvinceId, StateId};
use oh_data::trigger::{Condition, Definition, Effect, MAX_DEPTH, MAX_EFFECTS, Target};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TriggerError {
    MissingContext,
    Unsupported,
    InvalidReference,
    Depth,
    Budget,
    InvalidArgument,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope {
    Global,
    Nation(NationId),
    State(StateId),
}
/// Conditions receive an immutable host. Event chance needs a separate M3 API.
pub trait Host: Clone {
    fn condition(&self, condition: &Condition, scope: Scope) -> Result<bool, TriggerError>;
    fn scope(&self, target: &Target, current: Scope) -> Result<Scope, TriggerError>;
    fn effect(&mut self, effect: &Effect, scope: Scope) -> Result<(), TriggerError>;
}
pub fn evaluate<H: Host>(host: &H, c: &Condition, scope: Scope) -> Result<bool, TriggerError> {
    eval(host, c, scope, 1)
}
fn eval<H: Host>(
    host: &H,
    c: &Condition,
    scope: Scope,
    depth: usize,
) -> Result<bool, TriggerError> {
    if depth > MAX_DEPTH {
        return Err(TriggerError::Depth);
    }
    match c {
        Condition::All(cs) => {
            if cs.is_empty() {
                return Err(TriggerError::InvalidArgument);
            }
            for c in cs {
                if !eval(host, c, scope, depth + 1)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Condition::Any(cs) => {
            if cs.is_empty() {
                return Err(TriggerError::InvalidArgument);
            }
            for c in cs {
                if eval(host, c, scope, depth + 1)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Condition::Not(c) => Ok(!eval(host, c, scope, depth + 1)?),
        _ => host.condition(c, scope),
    }
}
pub fn execute<H: Host>(
    host: &mut H,
    effects: &[Effect],
    scope: Scope,
) -> Result<usize, TriggerError> {
    let mut candidate = host.clone();
    let mut count = 0;
    exec(&mut candidate, effects, scope, 1, &mut count)?;
    *host = candidate;
    Ok(count)
}
fn exec<H: Host>(
    host: &mut H,
    es: &[Effect],
    scope: Scope,
    depth: usize,
    count: &mut usize,
) -> Result<(), TriggerError> {
    for e in es {
        if depth > MAX_DEPTH {
            return Err(TriggerError::Depth);
        }
        match e {
            Effect::Scope(a) => {
                let target = oh_data::trigger::target(&a.target)
                    .map_err(|_| TriggerError::InvalidArgument)?;
                let next = host.scope(&target, scope)?;
                exec(host, &a.effects, next, depth + 1, count)?;
            }
            Effect::If(a) => {
                let selected = if eval(host, &a.condition, scope, depth + 1)? {
                    &a.then
                } else {
                    &a.r#else
                };
                exec(host, selected, scope, depth + 1, count)?;
            }
            _ => {
                if *count == MAX_EFFECTS {
                    return Err(TriggerError::Budget);
                }
                *count += 1;
                host.effect(e, scope)?;
            }
        }
    }
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum EndCause {
    Date,
    Condition,
    Explicit(String),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EndCheckpoint {
    pub tick: u64,
    pub date: DateV1,
    pub hour: u8,
    pub causes: Vec<EndCause>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TriggerState {
    pub definitions_hash: u64,
    pub flags: Vec<(u16, Vec<String>)>,
    pub ended: Option<EndCheckpoint>,
}
impl TriggerState {
    pub(crate) fn empty(world: &World) -> Self {
        Self {
            definitions_hash: 0,
            flags: world
                .inputs()
                .nations()
                .iter()
                .map(|n| (n.id().0, Vec::new()))
                .collect(),
            ended: None,
        }
    }
    pub fn initial(defs: &Definition, world: &World) -> Result<Self, TriggerError> {
        let mut flags = Vec::new();
        for n in world.inputs().nations() {
            let mut fs = defs
                .initial_flags
                .as_ref()
                .and_then(|m| m.get(n.tag()))
                .cloned()
                .unwrap_or_default();
            fs.sort();
            flags.push((n.id().0, fs));
        }
        Ok(Self {
            definitions_hash: oh_core::state_hash(defs)
                .map_err(|_| TriggerError::InvalidArgument)?,
            flags,
            ended: None,
        })
    }
    pub fn validate(&self, defs: &Definition, world: &World, state: &State) -> Result<(), String> {
        if self.definitions_hash != oh_core::state_hash(defs).map_err(|e| e.to_string())? {
            return Err("TriggerDefinitionsMismatch".into());
        }
        if self.flags.iter().map(|(id, _)| *id).ne(world
            .inputs()
            .nations()
            .iter()
            .map(|n| n.id().0))
        {
            return Err("InvalidFlags: nation set/order".into());
        }
        for (_, fs) in &self.flags {
            if !strictly_sorted(fs.iter())
                || fs
                    .iter()
                    .any(|k| !defs.flag_keys.as_ref().is_some_and(|keys| keys.contains(k)))
            {
                return Err("InvalidFlags: key/order".into());
            }
        }
        if let Some(end) = &self.ended {
            if end.tick != state.tick()
                || end.date != DateV1::from(state.date())
                || end.hour != state.hour()
                || end.causes.is_empty()
                || !strictly_sorted(end.causes.iter())
            {
                return Err("InvalidEnd: checkpoint/order".into());
            }
            for cause in &end.causes {
                match cause {
                    EndCause::Date => {
                        let end = defs.end_date.as_ref().ok_or("InvalidEnd: no end_date")?;
                        let (y, m, d) = oh_data::trigger::date(end)?;
                        let expected = crate::formula::next_hour(
                            Date::new(y, m, d).map_err(|e| e.to_string())?,
                            23,
                        )
                        .map_err(|e| e.to_string())?;
                        if (state.date(), state.hour()) != expected {
                            return Err("InvalidEnd: date boundary".into());
                        }
                    }
                    EndCause::Condition => {
                        defs.end_conditions
                            .as_ref()
                            .ok_or("InvalidEnd: no condition")?;
                    }
                    EndCause::Explicit(id) => {
                        if !oh_data::valid_id(id)
                            || !defs
                                .effect_programs
                                .as_ref()
                                .is_some_and(|ps| ps.values().any(|p| has_source(&p.effects, id)))
                        {
                            return Err("InvalidEnd: source ID".into());
                        }
                    }
                }
            }
        } else {
            if defs.end_date.as_ref().is_some_and(|end| {
                oh_data::trigger::date(end).is_ok_and(|d| date(state.date()) > d)
            }) {
                return Err("InvalidEnd: missing date checkpoint".into());
            }
        }
        Ok(())
    }
}
fn has_source(es: &[Effect], id: &str) -> bool {
    es.iter().any(|e| match e {
        Effect::EndScenario(s) => s == id,
        Effect::Scope(a) => has_source(&a.effects, id),
        Effect::If(a) => has_source(&a.then, id) || has_source(&a.r#else, id),
        _ => false,
    })
}
fn date(d: Date) -> (u32, u8, u8) {
    (d.year(), d.month(), d.day())
}
pub fn root(world: &World, tag: Option<&str>) -> Result<Scope, TriggerError> {
    match tag {
        None => Ok(Scope::Global),
        Some(tag) => world
            .inputs()
            .nations()
            .iter()
            .find(|n| n.tag() == tag)
            .map(|n| Scope::Nation(n.id()))
            .ok_or(TriggerError::InvalidReference),
    }
}
#[derive(Clone)]
pub struct SimHost<'a> {
    world: &'a World,
    state: &'a State,
    flags: BTreeMap<u16, BTreeSet<String>>,
    economy: Option<crate::economy::Economy>,
    pub sources: BTreeSet<String>,
}
impl<'a> SimHost<'a> {
    pub fn new(world: &'a World, state: &'a State, trigger: &TriggerState) -> Self {
        Self {
            world,
            state,
            flags: trigger
                .flags
                .iter()
                .map(|(id, fs)| (*id, fs.iter().cloned().collect()))
                .collect(),
            sources: BTreeSet::new(),
            economy: None,
        }
    }
    pub(crate) fn with_economy(mut self, economy: Option<&crate::economy::Economy>) -> Self {
        self.economy = economy.cloned();
        self
    }
    pub(crate) fn economy(&self) -> Option<&crate::economy::Economy> {
        self.economy.as_ref()
    }
    pub fn commit(self, trigger: &mut TriggerState) {
        trigger.flags = self
            .flags
            .into_iter()
            .map(|(id, fs)| (id, fs.into_iter().collect()))
            .collect();
    }
}
impl Host for SimHost<'_> {
    fn condition(&self, c: &Condition, scope: Scope) -> Result<bool, TriggerError> {
        if let Condition::DateGte(s) = c {
            return Ok(date(self.state.date())
                >= oh_data::trigger::date(s).map_err(|_| TriggerError::InvalidArgument)?);
        }
        let Scope::Nation(id) = scope else {
            return Err(TriggerError::MissingContext);
        };
        let n = self
            .world
            .nation(id)
            .ok_or(TriggerError::InvalidReference)?;
        match c {
            Condition::Stability(value)
            | Condition::Mobilization(value)
            | Condition::PoliticalCapital(value) => {
                let economy = self.economy.as_ref().ok_or(TriggerError::MissingContext)?;
                let e = economy.nation(id).ok_or(TriggerError::InvalidReference)?;
                if matches!(c, Condition::PoliticalCapital(_)) {
                    // Compare Qty directly with the exact decimal. Never narrow PC into Fx.
                    let (oh_data::trigger::Compare::Gte(s)
                    | oh_data::trigger::Compare::Lte(s)
                    | oh_data::trigger::Compare::Eq(s)) = value;
                    let v = s
                        .parse::<oh_core::Qty>()
                        .map_err(|_| TriggerError::InvalidArgument)?;
                    return Ok(match value {
                        oh_data::trigger::Compare::Gte(_) => e.political_capital() >= v,
                        oh_data::trigger::Compare::Lte(_) => e.political_capital() <= v,
                        oh_data::trigger::Compare::Eq(_) => e.political_capital() == v,
                    });
                }
                value
                    .test(if matches!(c, Condition::Stability(_)) {
                        e.stability()
                    } else {
                        e.mobilization()
                    })
                    .map_err(|_| TriggerError::InvalidArgument)
            }
            Condition::HasLaw(law) => Ok(self
                .economy
                .as_ref()
                .ok_or(TriggerError::MissingContext)?
                .nation(id)
                .ok_or(TriggerError::InvalidReference)?
                .laws()
                .values()
                .any(|id| id == law)),
            Condition::NationIs(tag) => Ok(n.tag() == tag),
            Condition::ControlsProvince(p) => Ok(self
                .world
                .province(ProvinceId(*p))
                .ok_or(TriggerError::InvalidReference)?
                .controller()
                == Some(id)),
            Condition::OwnsState(s) => Ok(self
                .world
                .state(StateId(*s))
                .ok_or(TriggerError::InvalidReference)?
                .owner()
                == id),
            Condition::HasFlag(k) => Ok(self
                .flags
                .get(&id.0)
                .ok_or(TriggerError::InvalidReference)?
                .contains(k)),
            Condition::IdeologySupport(a) => a
                .value
                .test(
                    *n.support()
                        .get(&a.ideology)
                        .ok_or(TriggerError::InvalidReference)?,
                )
                .map_err(|_| TriggerError::InvalidArgument),
            _ => Err(TriggerError::Unsupported),
        }
    }
    fn scope(&self, target: &Target, current: Scope) -> Result<Scope, TriggerError> {
        match target {
            Target::SelfScope => Ok(current),
            Target::Nation(tag) => root(self.world, Some(tag)),
            Target::State(id) => {
                self.world
                    .state(StateId(*id))
                    .ok_or(TriggerError::InvalidReference)?;
                Ok(Scope::State(StateId(*id)))
            }
            Target::OwnerOfState(id) => Ok(Scope::Nation(
                self.world
                    .state(StateId(*id))
                    .ok_or(TriggerError::InvalidReference)?
                    .owner(),
            )),
            Target::CapitalState => {
                let Scope::Nation(id) = current else {
                    return Err(TriggerError::MissingContext);
                };
                let n = self
                    .world
                    .defs()
                    .nations()
                    .iter()
                    .find(|n| n.id == id.0)
                    .ok_or(TriggerError::InvalidReference)?;
                Ok(Scope::State(
                    self.world
                        .province(ProvinceId(n.capital))
                        .and_then(|p| p.state())
                        .ok_or(TriggerError::InvalidReference)?,
                ))
            }
            _ => Err(TriggerError::Unsupported),
        }
    }
    fn effect(&mut self, e: &Effect, scope: Scope) -> Result<(), TriggerError> {
        if let Effect::EndScenario(id) = e {
            if !oh_data::valid_id(id) {
                return Err(TriggerError::InvalidArgument);
            }
            self.sources.insert(id.clone());
            return Ok(());
        }
        let Scope::Nation(id) = scope else {
            return Err(TriggerError::MissingContext);
        };
        match e {
            Effect::SetFlag(k) | Effect::ClearFlag(k) => {
                let defs = self
                    .world
                    .defs()
                    .trigger()
                    .ok_or(TriggerError::MissingContext)?;
                if !defs.flag_keys.as_ref().is_some_and(|ks| ks.contains(k)) {
                    return Err(TriggerError::InvalidReference);
                }
                let flags = self
                    .flags
                    .get_mut(&id.0)
                    .ok_or(TriggerError::InvalidReference)?;
                if matches!(e, Effect::SetFlag(_)) {
                    flags.insert(k.clone());
                } else {
                    flags.remove(k);
                }
                Ok(())
            }
            Effect::AddStability(v)
            | Effect::AddMobilization(v)
            | Effect::AddPoliticalCapital(v)
            | Effect::SetLaw(v) => {
                let empty = TriggerState {
                    definitions_hash: 0,
                    flags: self
                        .flags
                        .iter()
                        .map(|(id, fs)| (*id, fs.iter().cloned().collect()))
                        .collect(),
                    ended: None,
                };
                let allowed = if matches!(e, Effect::SetLaw(_)) {
                    let d = self
                        .world
                        .defs()
                        .economy()
                        .ok_or(TriggerError::MissingContext)?;
                    let l = d.laws.get(v).ok_or(TriggerError::InvalidReference)?;
                    evaluate(self, &l.condition, scope)?
                } else {
                    true
                };
                let _ = empty;
                self.economy
                    .as_mut()
                    .ok_or(TriggerError::MissingContext)?
                    .political_effect(self.world, id, e, self.state.tick(), allowed)
                    .map_err(|_| TriggerError::InvalidArgument)
            }
            _ => Err(TriggerError::Unsupported),
        }
    }
}
pub fn checkpoint(
    world: &World,
    state: &State,
    trigger: &mut TriggerState,
    sources: BTreeSet<String>,
    evaluate_condition: bool,
) -> Result<(), TriggerError> {
    checkpoint_economy(world, state, trigger, sources, evaluate_condition, None)
}
pub(crate) fn checkpoint_economy(
    world: &World,
    state: &State,
    trigger: &mut TriggerState,
    sources: BTreeSet<String>,
    evaluate_condition: bool,
    economy: Option<&crate::economy::Economy>,
) -> Result<(), TriggerError> {
    let defs = world.defs().trigger().ok_or(TriggerError::MissingContext)?;
    let mut causes = Vec::new();
    if let Some(end) = &defs.end_date
        && date(state.date())
            > oh_data::trigger::date(end).map_err(|_| TriggerError::InvalidArgument)?
    {
        causes.push(EndCause::Date);
    }
    if evaluate_condition && let Some(c) = &defs.end_conditions {
        let host = SimHost::new(world, state, trigger).with_economy(economy);
        if evaluate(&host, c, root(world, defs.end_root.as_deref())?)? {
            causes.push(EndCause::Condition);
        }
    }
    causes.extend(sources.into_iter().map(EndCause::Explicit));
    if !causes.is_empty() {
        trigger.ended = Some(EndCheckpoint {
            tick: state.tick(),
            date: state.date().into(),
            hour: state.hour(),
            causes,
        });
    }
    Ok(())
}
