//! Additive format. None and the frozen v1/v2/v3 DTOs are unchanged.
use crate::{Command, Simulation, save_state::*, trigger::TriggerState};
use oh_core::{DivisionId, NationId, ProvinceId};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommandV4 {
    Pause(bool),
    SetSpeed(u8),
    Move { unit: u32, destination: u16 },
    Stop { unit: u32 },
    Effects { program: String },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingV4 {
    pub tick: u64,
    pub nation: u16,
    pub sequence: u64,
    pub command: CommandV4,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationSaveV4 {
    pub base: SimulationSaveV3,
    pub movement_present: bool,
    pub strait_present: bool,
    pub queue: Vec<PendingV4>,
    pub trigger: TriggerState,
}
impl Simulation {
    pub fn export_save_v4(&self) -> Result<SimulationSaveV4, String> {
        let trigger = self.trigger.clone().ok_or("V4RequiresTrigger")?;
        let mut legacy = self.clone();
        legacy.trigger = None;
        legacy.queue.clear();
        let movement_present = legacy.movement.is_some();
        let strait_present = legacy
            .movement
            .as_ref()
            .is_some_and(|m| m.has_strait_context());
        let base = if strait_present {
            legacy.export_save_v3()?
        } else if movement_present {
            SimulationSaveV3 {
                base: legacy.export_save_v2()?,
                straits: Vec::new(),
            }
        } else {
            SimulationSaveV3 {
                base: SimulationSaveV2 {
                    base: legacy.export_save()?,
                    queue: Vec::new(),
                    units: Vec::new(),
                },
                straits: Vec::new(),
            }
        };
        let queue = self
            .queue
            .iter()
            .map(|(&(tick, nation, sequence), c)| PendingV4 {
                tick,
                nation: nation.0,
                sequence,
                command: match c {
                    Command::Pause(v) => CommandV4::Pause(*v),
                    Command::SetSpeed(v) => CommandV4::SetSpeed(*v),
                    Command::Move { unit, destination } => CommandV4::Move {
                        unit: unit.0,
                        destination: destination.0,
                    },
                    Command::Stop { unit } => CommandV4::Stop { unit: unit.0 },
                    Command::Effects { program } => CommandV4::Effects {
                        program: program.clone(),
                    },
                },
            })
            .collect();
        Ok(SimulationSaveV4 {
            base,
            movement_present,
            strait_present,
            queue,
            trigger,
        })
    }
    pub fn from_save_v4(dto: SimulationSaveV4, context: &RestoreContext) -> Result<Self, String> {
        if !dto.base.base.base.queue.is_empty() || !dto.base.base.queue.is_empty() {
            return Err("InvalidV4: nested queue must be empty".into());
        }
        if (!dto.movement_present && (!dto.base.base.units.is_empty() || dto.strait_present))
            || (!dto.strait_present && !dto.base.straits.is_empty())
        {
            return Err("InvalidV4: movement presence".into());
        }
        let mut sim = if dto.strait_present {
            Self::from_strait_save(dto.base, context)?
        } else if dto.movement_present {
            Self::from_movement_save(dto.base.base, context, None)?
        } else {
            Self::from_save_base(dto.base.base.base, context)?
        };
        let world = sim.world.as_ref().ok_or("V4RequiresWorld")?;
        let defs = world.defs().trigger().ok_or("V4RequiresLocalDefinitions")?;
        dto.trigger.validate(defs, world, &sim.state)?;
        if !strictly_sorted(dto.queue.iter().map(|q| (q.tick, q.nation, q.sequence))) {
            return Err("InvalidQueue: duplicate/order".into());
        }
        for q in dto.queue {
            if q.tick < sim.state.tick {
                return Err("InvalidQueue: past tick".into());
            }
            let command = match q.command {
                CommandV4::Pause(v) => Command::Pause(v),
                CommandV4::SetSpeed(v) if (1..=5).contains(&v) => Command::SetSpeed(v),
                CommandV4::Effects { program } => {
                    let p = defs
                        .effect_programs
                        .as_ref()
                        .and_then(|ps| ps.get(&program))
                        .ok_or("InvalidQueue: program ID")?;
                    if world.nation(NationId(q.nation)).is_none()
                        || p.root.as_ref().is_some_and(|tag| {
                            world
                                .nation(NationId(q.nation))
                                .is_none_or(|n| n.tag() != tag)
                        })
                    {
                        return Err("InvalidQueue: program authority".into());
                    }
                    Command::Effects { program }
                }
                CommandV4::Move { unit, destination } => {
                    let u = sim
                        .movement
                        .as_ref()
                        .and_then(|m| m.unit(DivisionId(unit)))
                        .ok_or("InvalidQueue: unit")?;
                    if u.nation.0 != q.nation
                        || !crate::movement::land(world.defs().map(), ProvinceId(destination))
                    {
                        return Err("InvalidQueue: unit owner/destination".into());
                    }
                    Command::Move {
                        unit: DivisionId(unit),
                        destination: ProvinceId(destination),
                    }
                }
                CommandV4::Stop { unit } => {
                    if sim
                        .movement
                        .as_ref()
                        .and_then(|m| m.unit(DivisionId(unit)))
                        .is_none_or(|u| u.nation.0 != q.nation)
                    {
                        return Err("InvalidQueue: unit owner".into());
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
        sim.trigger = Some(dto.trigger);
        Ok(sim)
    }
}
