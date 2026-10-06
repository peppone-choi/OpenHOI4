//! Versioned, untrusted save DTOs. Live authority stays private and validated.
use crate::{Command, Date, Simulation, State, TimeConfig, world::World};
use oh_core::NationId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DateV1 {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}
impl From<Date> for DateV1 {
    fn from(d: Date) -> Self {
        Self {
            year: d.year(),
            month: d.month(),
            day: d.day(),
        }
    }
}
impl DateV1 {
    pub fn validate(&self) -> Result<Date, String> {
        Date::new(self.year, self.month, self.day).map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateV1 {
    pub scenario: String,
    pub seed: u64,
    pub tick: u64,
    pub date: DateV1,
    pub hour: u8,
    pub paused: bool,
    pub speed: u8,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimeV1 {
    pub speed_ms_per_tick: [u64; 5],
    pub initial_speed: u8,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommandV1 {
    Pause(bool),
    SetSpeed(u8),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingV1 {
    pub tick: u64,
    pub nation: u16,
    pub sequence: u64,
    pub command: CommandV1,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationSaveV1 {
    pub state: StateV1,
    pub config: TimeV1,
    pub queue: Vec<PendingV1>,
    pub world: Option<WorldV1>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldV1 {
    pub definitions_hash: u64,
    pub inputs: InputsV1,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InputsV1 {
    pub nations: Vec<NationV1>,
    pub states: Vec<StateWorldV1>,
    pub provinces: Vec<ProvinceV1>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NationV1 {
    pub id: u16,
    pub tag: String,
    pub government: String,
    pub support: Vec<(String, i64)>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateWorldV1 {
    pub id: u16,
    pub owner: u16,
    pub population: i64,
    pub resources: Vec<(String, i64)>,
    pub buildings: Vec<(String, i64)>,
    pub base: i64,
    pub modifiers: Vec<ModifierV1>,
    pub infrastructure: i64,
    pub ledger: LedgerV1,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ModifierOpV1 {
    Add,
    Mul,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModifierV1 {
    pub source: String,
    pub target_stat: String,
    pub op: ModifierOpV1,
    pub value: i64,
    pub expires: Option<u64>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LedgerOpV1 {
    Base,
    Add,
    Mul,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntryV1 {
    pub source: Option<String>,
    pub op: LedgerOpV1,
    pub value: i64,
    pub accumulated: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerV1 {
    pub target_stat: String,
    pub tick: u64,
    pub value: i64,
    pub entries: Vec<EntryV1>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvinceV1 {
    pub id: u16,
    pub state: Option<u16>,
    pub owner: Option<u16>,
    pub controller: Option<u16>,
}

/// Context comes from validated local data, never from the save's references.
#[derive(Clone)]
pub struct RestoreContext {
    pub scenario: String,
    pub start_date: Date,
    pub world: Option<World>,
}

pub(crate) fn strictly_sorted<T: Ord>(items: impl IntoIterator<Item = T>) -> bool {
    let mut previous = None;
    for item in items {
        if previous.as_ref().is_some_and(|p| p >= &item) {
            return false;
        }
        previous = Some(item);
    }
    true
}
fn ordinal(d: Date) -> u64 {
    let y = u64::from(d.year() - 1);
    let mut days = y * 365 + y / 4 - y / 100 + y / 400;
    for month in 1..d.month() {
        days += u64::from(crate::formula::days_in_month(d.year(), month));
    }
    days + u64::from(d.day() - 1)
}

impl Simulation {
    pub fn export_save(&self) -> Result<SimulationSaveV1, String> {
        let queue = self
            .queue
            .iter()
            .map(|(&(tick, nation, sequence), c)| {
                let command = match *c {
                    Command::Pause(v) => CommandV1::Pause(v),
                    Command::SetSpeed(v) if (1..=5).contains(&v) => CommandV1::SetSpeed(v),
                    _ => return Err("InvalidQueue: speed".into()),
                };
                Ok(PendingV1 {
                    tick,
                    nation: nation.0,
                    sequence,
                    command,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(SimulationSaveV1 {
            state: StateV1 {
                scenario: self.state.scenario.clone(),
                seed: self.state.seed,
                tick: self.state.tick,
                date: self.state.date.into(),
                hour: self.state.hour,
                paused: self.state.paused,
                speed: self.state.speed,
            },
            config: TimeV1 {
                speed_ms_per_tick: self.config.speed_ms_per_tick(),
                initial_speed: self.config.initial_speed(),
            },
            queue,
            world: self.world.as_ref().map(World::export_save),
        })
    }
    /// Returns a fully checked new value; cannot mutate an existing simulation.
    pub fn from_save(dto: SimulationSaveV1, context: &RestoreContext) -> Result<Self, String> {
        let state = &dto.state;
        let date = state.date.validate()?;
        if state.scenario != context.scenario
            || state.scenario.is_empty()
            || !state
                .scenario
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err("InvalidState: scenario".into());
        }
        if u64::from(state.hour) >= crate::formula::HOURS_PER_DAY || !(1..=5).contains(&state.speed)
        {
            return Err("InvalidState: hour/speed".into());
        }
        let elapsed = ordinal(date)
            .checked_sub(ordinal(context.start_date))
            .and_then(|days| days.checked_mul(crate::formula::HOURS_PER_DAY))
            .and_then(|hours| hours.checked_add(u64::from(state.hour)));
        if elapsed != Some(state.tick) {
            return Err("InvalidState: date/tick".into());
        }
        let config = TimeConfig::new(dto.config.speed_ms_per_tick, dto.config.initial_speed)
            .map_err(|e| e.to_string())?;
        if !strictly_sorted(dto.queue.iter().map(|q| (q.tick, q.nation, q.sequence))) {
            return Err("InvalidQueue: duplicate/unsorted key".into());
        }
        let mut queue = BTreeMap::new();
        for q in dto.queue {
            if q.tick < state.tick {
                return Err("InvalidQueue: past tick".into());
            }
            let command = match q.command {
                CommandV1::Pause(v) => Command::Pause(v),
                CommandV1::SetSpeed(v) if (1..=5).contains(&v) => Command::SetSpeed(v),
                _ => return Err("InvalidQueue: speed".into()),
            };
            queue.insert((q.tick, NationId(q.nation), q.sequence), command);
        }
        let world = match (dto.world, context.world.as_ref()) {
            (None, None) => None,
            (Some(w), Some(template)) => Some(World::from_save(w, template, state.tick)?),
            _ => return Err("InvalidWorld: M0/M1 mode".into()),
        };
        Ok(Self {
            state: State {
                scenario: state.scenario.clone(),
                seed: state.seed,
                tick: state.tick,
                date,
                hour: state.hour,
                paused: state.paused,
                speed: state.speed,
            },
            config,
            queue,
            world,
        })
    }
}
