//! V5 preserves economy presence independently of trigger/movement presence.
use crate::{
    Command, Simulation,
    economy::{Action, Economy},
    save_state::*,
    trigger::TriggerState,
};
use oh_core::{DivisionId, NationId, ProvinceId};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommandV5 {
    Pause(bool),
    SetSpeed(u8),
    Move { unit: u32, destination: u16 },
    Stop { unit: u32 },
    Effects { program: String },
    Economy(Action),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingV5 {
    pub tick: u64,
    pub nation: u16,
    pub sequence: u64,
    pub command: CommandV5,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationSaveV5 {
    pub base: SimulationSaveV3,
    pub movement_present: bool,
    pub strait_present: bool,
    pub trigger: Option<TriggerState>,
    pub queue: Vec<PendingV5>,
    pub economy: Economy,
}
impl Simulation {
    pub fn export_save_v5(&self) -> Result<SimulationSaveV5, String> {
        if self.production.is_some() {
            return Err("ProductionRequiresV6".into());
        }
        let economy = self.economy.clone().ok_or("V5RequiresEconomy")?;
        let mut legacy = self.clone();
        legacy.economy = None;
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
                straits: vec![],
            }
        } else {
            SimulationSaveV3 {
                base: SimulationSaveV2 {
                    base: legacy.export_save()?,
                    queue: vec![],
                    units: vec![],
                },
                straits: vec![],
            }
        };
        let queue = self
            .queue
            .iter()
            .map(|(&(tick, nation, sequence), c)| PendingV5 {
                tick,
                nation: nation.0,
                sequence,
                command: match c {
                    Command::Pause(v) => CommandV5::Pause(*v),
                    Command::SetSpeed(v) => CommandV5::SetSpeed(*v),
                    Command::Move { unit, destination } => CommandV5::Move {
                        unit: unit.0,
                        destination: destination.0,
                    },
                    Command::Stop { unit } => CommandV5::Stop { unit: unit.0 },
                    Command::Effects { program } => CommandV5::Effects {
                        program: program.clone(),
                    },
                    Command::Economy(action) => CommandV5::Economy(action.clone()),
                    Command::Production(_) => unreachable!("production requires v6"),
                },
            })
            .collect();
        Ok(SimulationSaveV5 {
            base,
            movement_present,
            strait_present,
            trigger: self.trigger.clone(),
            queue,
            economy,
        })
    }
    pub fn from_save_v5(dto: SimulationSaveV5, context: &RestoreContext) -> Result<Self, String> {
        if !dto.base.base.base.queue.is_empty() || !dto.base.base.queue.is_empty() {
            return Err("InvalidV5: nested queue".into());
        }
        if (!dto.movement_present && (!dto.base.base.units.is_empty() || dto.strait_present))
            || (!dto.strait_present && !dto.base.straits.is_empty())
        {
            return Err("InvalidV5: movement presence".into());
        }
        if context
            .world
            .as_ref()
            .is_none_or(|w| w.defs().economy().is_none())
        {
            return Err("EconomyModeMismatch".into());
        }
        let mut sim = if dto.strait_present {
            Self::from_strait_save(dto.base, context)?
        } else if dto.movement_present {
            Self::from_movement_save(dto.base.base, context, None)?
        } else {
            Self::from_save_base(dto.base.base.base, context)?
        };
        let world = sim.world.as_ref().ok_or("V5RequiresWorld")?;
        match (&dto.trigger, world.defs().trigger()) {
            (Some(t), Some(d)) => t.validate(d, world, &sim.state)?,
            (None, None) => {}
            _ => return Err("TriggerModeMismatch".into()),
        }
        dto.economy
            .validate(world, sim.state.tick())
            .map_err(|e| e.to_string())?;
        dto.economy
            .validate_scores(
                world,
                sim.state.tick(),
                dto.trigger.as_ref().is_some_and(|t| t.ended.is_some()),
            )
            .map_err(|e| e.to_string())?;
        if !strictly_sorted(dto.queue.iter().map(|q| (q.tick, q.nation, q.sequence))) {
            return Err("InvalidQueue: duplicate/order".into());
        }
        for q in dto.queue {
            if q.tick < sim.state.tick() {
                return Err("InvalidQueue: past tick".into());
            }
            let command = match q.command {
                CommandV5::Pause(v) => Command::Pause(v),
                CommandV5::SetSpeed(v) if (1..=5).contains(&v) => Command::SetSpeed(v),
                CommandV5::Move { unit, destination } => {
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
                CommandV5::Stop { unit } => {
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
                CommandV5::Effects { program } => {
                    let p = world
                        .defs()
                        .trigger()
                        .and_then(|d| d.effect_programs.as_ref())
                        .and_then(|ps| ps.get(&program))
                        .ok_or("InvalidQueue: program")?;
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
                CommandV5::Economy(action) => {
                    dto.economy
                        .validate_action(world, NationId(q.nation), &action)
                        .map_err(|e| e.to_string())?;
                    Command::Economy(action)
                }
                _ => return Err("InvalidQueue: speed".into()),
            };
            sim.queue
                .insert((q.tick, NationId(q.nation), q.sequence), command);
        }
        sim.trigger = dto.trigger;
        sim.economy = Some(dto.economy);
        Ok(sim)
    }
}
