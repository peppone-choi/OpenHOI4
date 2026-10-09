//! Additive V6 production body; frozen V5 is exported without a nested queue.
use crate::{
    Command, Simulation,
    economy_save::*,
    production::{Action, Production},
    save_state::{RestoreContext, strictly_sorted},
};
use oh_core::NationId;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommandV6 {
    Legacy(CommandV5),
    Production(Action),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingV6 {
    pub tick: u64,
    pub nation: u16,
    pub sequence: u64,
    pub command: CommandV6,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationSaveV6 {
    pub base: SimulationSaveV5,
    pub queue: Vec<PendingV6>,
    pub production: Production,
}
impl Simulation {
    pub fn export_save_v6(&self) -> Result<SimulationSaveV6, String> {
        if self.military.is_some() {
            return Err("MilitaryRequiresV7".into());
        }
        let production = self.production.clone().ok_or("V6RequiresProduction")?;
        let mut old = self.clone();
        old.production = None;
        old.queue.clear();
        let base = old.export_save_v5()?;
        let queue = self
            .queue
            .iter()
            .map(|(&(tick, nation, sequence), c)| PendingV6 {
                tick,
                nation: nation.0,
                sequence,
                command: match c {
                    Command::Production(a) => CommandV6::Production(a.clone()),
                    Command::Pause(v) => CommandV6::Legacy(CommandV5::Pause(*v)),
                    Command::SetSpeed(v) => CommandV6::Legacy(CommandV5::SetSpeed(*v)),
                    Command::Move { unit, destination } => CommandV6::Legacy(CommandV5::Move {
                        unit: unit.0,
                        destination: destination.0,
                    }),
                    Command::Stop { unit } => CommandV6::Legacy(CommandV5::Stop { unit: unit.0 }),
                    Command::Effects { program } => CommandV6::Legacy(CommandV5::Effects {
                        program: program.clone(),
                    }),
                    Command::Economy(a) => CommandV6::Legacy(CommandV5::Economy(a.clone())),
                    Command::Military(_) => unreachable!("military requires v7"),
                },
            })
            .collect();
        Ok(SimulationSaveV6 {
            base,
            queue,
            production,
        })
    }
    pub fn from_save_v6(mut dto: SimulationSaveV6, c: &RestoreContext) -> Result<Self, String> {
        if !dto.base.queue.is_empty() {
            return Err("InvalidV6: nested queue".into());
        }
        dto.base.queue = dto
            .queue
            .iter()
            .filter_map(|q| match &q.command {
                CommandV6::Legacy(command) => Some(PendingV5 {
                    tick: q.tick,
                    nation: q.nation,
                    sequence: q.sequence,
                    command: command.clone(),
                }),
                _ => None,
            })
            .collect();
        let mut sim = Self::from_save_v5(dto.base, c)?;
        dto.production
            .validate(
                sim.world.as_ref().ok_or("V6RequiresWorld")?,
                sim.snapshot().tick(),
            )
            .map_err(|e| e.to_string())?;
        sim.production = Some(dto.production);
        if !strictly_sorted(dto.queue.iter().map(|q| (q.tick, q.nation, q.sequence))) {
            return Err("InvalidQueue: duplicate/order".into());
        }
        for q in dto.queue {
            let command = match q.command {
                CommandV6::Production(a) => Command::Production(a),
                CommandV6::Legacy(c) => match c {
                    CommandV5::Pause(v) => Command::Pause(v),
                    CommandV5::SetSpeed(v) => Command::SetSpeed(v),
                    CommandV5::Move { unit, destination } => Command::Move {
                        unit: oh_core::DivisionId(unit),
                        destination: oh_core::ProvinceId(destination),
                    },
                    CommandV5::Stop { unit } => Command::Stop {
                        unit: oh_core::DivisionId(unit),
                    },
                    CommandV5::Effects { program } => Command::Effects { program },
                    CommandV5::Economy(a) => Command::Economy(a),
                },
            };
            // Future semantic commands can become invalid before their tick. Validate
            // structural refs/values on a clone without dropping legitimate future intents.
            sim.validate_production_pending(NationId(q.nation), &command)?;
            if q.tick < sim.snapshot().tick() {
                return Err("InvalidQueue: past tick".into());
            }
            if matches!(command,Command::SetSpeed(v) if !(1..=5).contains(&v)) {
                return Err("InvalidQueue: speed".into());
            }
            sim.queue
                .insert((q.tick, NationId(q.nation), q.sequence), command);
        }
        Ok(sim)
    }
    fn validate_production_pending(&self, n: NationId, c: &Command) -> Result<(), String> {
        let w = self.world.as_ref().ok_or("V6RequiresWorld")?;
        if w.nation(n).is_none() {
            return Err("InvalidQueue: nation".into());
        }
        if let Command::Production(a) = c {
            let d = w.defs().production().ok_or("ProductionModeMismatch")?;
            let allowed = &d
                .nations
                .get(&n.0)
                .ok_or("InvalidQueue: nation")?
                .allowed_models;
            match a {
                Action::Create {
                    model,
                    requested_ic,
                } => {
                    if !allowed.contains(model) || *requested_ic < oh_core::Qty::ZERO {
                        return Err("InvalidQueue: model/IC".into());
                    }
                }
                Action::SetIC { line, requested_ic } => {
                    if *requested_ic < oh_core::Qty::ZERO {
                        return Err("InvalidQueue: IC".into());
                    }
                    self.pending_line(n, *line)?;
                }
                Action::Switch { line, model } => {
                    if !allowed.contains(model) {
                        return Err("InvalidQueue: model".into());
                    }
                    self.pending_line(n, *line)?;
                }
                Action::Pause { line, .. } | Action::Cancel { line } => {
                    self.pending_line(n, *line)?
                }
            }
        }
        Ok(())
    }
    fn pending_line(&self, n: NationId, id: u64) -> Result<(), String> {
        let p = self.production.as_ref().ok_or("ProductionModeMismatch")?;
        match p.line(id) {
            Some(l) if l.nation() != n => Err("InvalidQueue: line owner".into()),
            None if id >= p.next_line_id() => Err("InvalidQueue: never allocated line".into()),
            _ => Ok(()),
        }
    }
}
