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
        if self.trigger.is_some() {
            return Err("TriggerRequiresV4".into());
        }
        if self.movement.is_some() {
            return Err("MovementRequiresV2".into());
        }
        self.export_legacy_base()
    }
    fn export_legacy_base(&self) -> Result<SimulationSaveV1, String> {
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
        legacy_trigger_mode(context)?;
        Self::from_save_base(dto, context)
    }
    pub(crate) fn from_save_base(
        dto: SimulationSaveV1,
        context: &RestoreContext,
    ) -> Result<Self, String> {
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
            movement: None,
            trigger: None,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommandV2 {
    Pause(bool),
    SetSpeed(u8),
    Move { unit: u32, destination: u16 },
    Stop { unit: u32 },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingV2 {
    pub tick: u64,
    pub nation: u16,
    pub sequence: u64,
    pub command: CommandV2,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CorrectionV2 {
    pub from: u16,
    pub to: u16,
    pub factors: [i64; 4],
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LegV2 {
    pub from: u16,
    pub to: u16,
    pub hours: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnitV2 {
    pub id: u32,
    pub nation: u16,
    pub province: u16,
    pub speed: i64,
    pub allowed: Vec<u16>,
    pub corrections: Vec<CorrectionV2>,
    pub route: Vec<LegV2>,
    pub elapsed: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationSaveV2 {
    /// Frozen legacy structure; its queue is empty (complete queue below).
    pub base: SimulationSaveV1,
    pub queue: Vec<PendingV2>,
    pub units: Vec<UnitV2>,
}
impl Simulation {
    pub fn export_save_v2(&self) -> Result<SimulationSaveV2, String> {
        if self.trigger.is_some() {
            return Err("TriggerRequiresV4".into());
        }
        let movement = self.movement.as_ref().ok_or("V2RequiresMovement")?;
        if movement.has_strait_context() {
            return Err("StraitRequiresV3".into());
        }
        let mut legacy = self.clone();
        legacy.movement = None;
        legacy.queue.clear();
        let base = legacy.export_save()?;
        let queue = self
            .queue
            .iter()
            .map(|(&(tick, nation, sequence), c)| PendingV2 {
                tick,
                nation: nation.0,
                sequence,
                command: match c {
                    Command::Pause(v) => CommandV2::Pause(*v),
                    Command::SetSpeed(v) => CommandV2::SetSpeed(*v),
                    Command::Move { unit, destination } => CommandV2::Move {
                        unit: unit.0,
                        destination: destination.0,
                    },
                    Command::Stop { unit } => CommandV2::Stop { unit: unit.0 },
                    Command::Effects { .. } => unreachable!("trigger requires v4"),
                },
            })
            .collect();
        let units = movement
            .units
            .iter()
            .map(|u| UnitV2 {
                id: u.id.0,
                nation: u.nation.0,
                province: u.province.0,
                speed: u.speed.to_bits(),
                allowed: u.allowed.iter().map(|id| id.0).collect(),
                corrections: u
                    .corrections
                    .iter()
                    .map(|(&(from, to), f)| CorrectionV2 {
                        from: from.0,
                        to: to.0,
                        factors: [
                            f.terrain.to_bits(),
                            f.infrastructure.to_bits(),
                            f.supply.to_bits(),
                            f.river.to_bits(),
                        ],
                    })
                    .collect(),
                route: u
                    .route
                    .iter()
                    .map(|l| LegV2 {
                        from: l.from.0,
                        to: l.to.0,
                        hours: l.hours.to_bits(),
                    })
                    .collect(),
                elapsed: u.elapsed.to_bits(),
            })
            .collect();
        Ok(SimulationSaveV2 { base, queue, units })
    }
    pub fn from_save_v2(dto: SimulationSaveV2, context: &RestoreContext) -> Result<Self, String> {
        legacy_trigger_mode(context)?;
        Self::from_movement_save(dto, context, None)
    }
    pub(crate) fn from_movement_save(
        dto: SimulationSaveV2,
        context: &RestoreContext,
        straits: Option<BTreeMap<oh_core::DivisionId, crate::movement::DirectedStraits>>,
    ) -> Result<Self, String> {
        use crate::movement::{Factors, Leg, Movement, Unit};
        use oh_core::{DivisionId, Fx, ProvinceId};
        if !dto.base.queue.is_empty() {
            return Err("InvalidV2: legacy queue must be empty".into());
        }
        let mut sim = Self::from_save_base(dto.base, context)?;
        let world = sim.world.as_ref().ok_or("V2RequiresWorld")?;
        if !strictly_sorted(dto.units.iter().map(|u| u.id)) {
            return Err("InvalidMovement: duplicate/unsorted units".into());
        }
        let mut units = Vec::new();
        for u in dto.units {
            if !strictly_sorted(u.allowed.iter().copied())
                || !strictly_sorted(u.corrections.iter().map(|f| (f.from, f.to)))
            {
                return Err("InvalidMovement: duplicate/unsorted context".into());
            }
            units.push(Unit {
                id: DivisionId(u.id),
                nation: NationId(u.nation),
                province: ProvinceId(u.province),
                speed: Fx::from_bits(u.speed),
                allowed: u.allowed.into_iter().map(ProvinceId).collect(),
                corrections: u
                    .corrections
                    .into_iter()
                    .map(|c| {
                        (
                            (ProvinceId(c.from), ProvinceId(c.to)),
                            Factors {
                                terrain: Fx::from_bits(c.factors[0]),
                                infrastructure: Fx::from_bits(c.factors[1]),
                                supply: Fx::from_bits(c.factors[2]),
                                river: Fx::from_bits(c.factors[3]),
                            },
                        )
                    })
                    .collect(),
                route: u
                    .route
                    .into_iter()
                    .map(|l| Leg {
                        from: ProvinceId(l.from),
                        to: ProvinceId(l.to),
                        hours: Fx::from_bits(l.hours),
                    })
                    .collect(),
                elapsed: Fx::from_bits(u.elapsed),
            });
        }
        let movement = Movement { units, straits };
        movement.validate(world).map_err(|e| e.to_string())?;
        if !strictly_sorted(dto.queue.iter().map(|q| (q.tick, q.nation, q.sequence))) {
            return Err("InvalidQueue: duplicate/unsorted key".into());
        }
        for q in dto.queue {
            if q.tick < sim.state.tick {
                return Err("InvalidQueue: past tick".into());
            }
            let command = match q.command {
                CommandV2::Pause(v) => Command::Pause(v),
                CommandV2::SetSpeed(v) if (1..=5).contains(&v) => Command::SetSpeed(v),
                CommandV2::Move { unit, destination } => {
                    let u = movement
                        .unit(DivisionId(unit))
                        .ok_or("InvalidQueue: unit")?;
                    if u.nation.0 != q.nation
                        || !crate::movement::land(world.defs().map(), ProvinceId(destination))
                    {
                        return Err("InvalidQueue: owner/destination".into());
                    }
                    Command::Move {
                        unit: DivisionId(unit),
                        destination: ProvinceId(destination),
                    }
                }
                CommandV2::Stop { unit } => {
                    if movement
                        .unit(DivisionId(unit))
                        .is_none_or(|u| u.nation.0 != q.nation)
                    {
                        return Err("InvalidQueue: owner/unit".into());
                    }
                    Command::Stop {
                        unit: DivisionId(unit),
                    }
                }
                _ => return Err("InvalidQueue: speed".into()),
            };
            sim.queue
                .insert((q.tick, NationId(q.nation), q.sequence), command);
        }
        sim.movement = Some(movement);
        Ok(sim)
    }
}

/// Frozen v2 body plus independently typed direction/kind/raw time context.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StraitCorrectionV3 {
    pub from: u16,
    pub to: u16,
    pub kind: u8,
    pub factors: [i64; 4],
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnitStraitsV3 {
    pub unit: u32,
    pub corrections: Vec<StraitCorrectionV3>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationSaveV3 {
    pub base: SimulationSaveV2,
    pub straits: Vec<UnitStraitsV3>,
}
impl Simulation {
    pub fn export_save_v3(&self) -> Result<SimulationSaveV3, String> {
        if self.trigger.is_some() {
            return Err("TriggerRequiresV4".into());
        }
        let contexts = self
            .movement
            .as_ref()
            .and_then(|m| m.straits.as_ref())
            .ok_or("V3RequiresStraitContext")?;
        let straits = contexts
            .iter()
            .map(|(unit, corrections)| UnitStraitsV3 {
                unit: unit.0,
                corrections: corrections
                    .iter()
                    .map(|(&(from, to), c)| StraitCorrectionV3 {
                        from: from.0,
                        to: to.0,
                        kind: c.kind as u8,
                        factors: [
                            c.factors.terrain.to_bits(),
                            c.factors.infrastructure.to_bits(),
                            c.factors.supply.to_bits(),
                            c.factors.strait.to_bits(),
                        ],
                    })
                    .collect(),
            })
            .collect();
        let mut legacy = self.clone();
        legacy
            .movement
            .as_mut()
            .ok_or("V3RequiresMovement")?
            .straits = None;
        Ok(SimulationSaveV3 {
            base: legacy.export_save_v2()?,
            straits,
        })
    }
    pub fn from_save_v3(dto: SimulationSaveV3, context: &RestoreContext) -> Result<Self, String> {
        legacy_trigger_mode(context)?;
        Self::from_strait_save(dto, context)
    }
    pub(crate) fn from_strait_save(
        dto: SimulationSaveV3,
        context: &RestoreContext,
    ) -> Result<Self, String> {
        use crate::movement::{CrossingKind, StraitContext, StraitFactors};
        use oh_core::{DivisionId, Fx, ProvinceId};
        if !strictly_sorted(dto.straits.iter().map(|c| c.unit)) {
            return Err("InvalidStrait: duplicate/unsorted units".into());
        }
        let mut straits = BTreeMap::new();
        for c in dto.straits {
            if !strictly_sorted(c.corrections.iter().map(|e| (e.from, e.to))) {
                return Err("InvalidStrait: duplicate/unsorted directions".into());
            }
            let mut corrections = BTreeMap::new();
            for e in c.corrections {
                if e.kind != CrossingKind::Strait as u8 {
                    return Err("InvalidStrait: crossing kind".into());
                }
                corrections.insert(
                    (ProvinceId(e.from), ProvinceId(e.to)),
                    StraitContext {
                        kind: CrossingKind::Strait,
                        factors: StraitFactors {
                            terrain: Fx::from_bits(e.factors[0]),
                            infrastructure: Fx::from_bits(e.factors[1]),
                            supply: Fx::from_bits(e.factors[2]),
                            strait: Fx::from_bits(e.factors[3]),
                        },
                    },
                );
            }
            straits.insert(DivisionId(c.unit), corrections);
        }
        Self::from_movement_save(dto.base, context, Some(straits))
    }
}
fn legacy_trigger_mode(context: &RestoreContext) -> Result<(), String> {
    if context
        .world
        .as_ref()
        .is_some_and(|w| w.defs().trigger().is_some())
    {
        return Err("TriggerModeMismatch: legacy None save/local Some definitions".into());
    }
    Ok(())
}
