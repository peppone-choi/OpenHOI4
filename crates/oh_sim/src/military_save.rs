//! Opt-in V7 actual military body. V1–V6 DTO shapes and queues remain frozen.
use crate::{
    Command, Simulation,
    military::{Action, Military},
    production_save::{CommandV6, PendingV6, SimulationSaveV6},
    save_state::{RestoreContext, strictly_sorted},
};
use oh_core::NationId;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommandV7 {
    Legacy(CommandV6),
    Military(Action),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingV7 {
    pub tick: u64,
    pub nation: u16,
    pub sequence: u64,
    pub command: CommandV7,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationSaveV7 {
    pub base: SimulationSaveV6,
    pub queue: Vec<PendingV7>,
    pub military: Military,
}
impl Simulation {
    pub fn export_save_v7(&self) -> Result<SimulationSaveV7, String> {
        let military = self.military.clone().ok_or("V7RequiresMilitary")?;
        let mut old = self.clone();
        old.military = None;
        old.queue
            .retain(|_, command| !matches!(command, Command::Military(_)));
        let mut base = old.export_save_v6()?;
        let legacy = base
            .queue
            .iter()
            .map(|q| ((q.tick, q.nation, q.sequence), q.command.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        base.queue.clear();
        let queue = self
            .queue
            .iter()
            .map(|(&(tick, nation, sequence), command)| {
                let command = match command {
                    Command::Military(a) => CommandV7::Military(a.clone()),
                    _ => CommandV7::Legacy(
                        legacy
                            .get(&(tick, nation.0, sequence))
                            .ok_or("InvalidV7: missing legacy command")?
                            .clone(),
                    ),
                };
                Ok(PendingV7 {
                    tick,
                    nation: nation.0,
                    sequence,
                    command,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(SimulationSaveV7 {
            base,
            queue,
            military,
        })
    }
    pub fn from_save_v7(mut dto: SimulationSaveV7, c: &RestoreContext) -> Result<Self, String> {
        if !dto.base.queue.is_empty() {
            return Err("InvalidV7: nested queue".into());
        }
        if !strictly_sorted(dto.queue.iter().map(|q| (q.tick, q.nation, q.sequence))) {
            return Err("InvalidQueue: duplicate/order".into());
        }
        dto.base.queue = dto
            .queue
            .iter()
            .filter_map(|q| match &q.command {
                CommandV7::Legacy(command) => Some(PendingV6 {
                    tick: q.tick,
                    nation: q.nation,
                    sequence: q.sequence,
                    command: command.clone(),
                }),
                CommandV7::Military(_) => None,
            })
            .collect();
        let mut sim = Self::from_save_v6_base(dto.base, c)?;
        dto.military
            .validate(
                sim.world.as_ref().ok_or("V7RequiresWorld")?,
                sim.economy.as_ref().ok_or("V7RequiresEconomy")?,
                sim.production.as_ref().ok_or("V7RequiresProduction")?,
                sim.snapshot().tick(),
            )
            .map_err(|e| e.to_string())?;
        sim.military = Some(dto.military);
        for q in dto.queue {
            if let CommandV7::Military(action) = q.command {
                if q.tick < sim.snapshot().tick() {
                    return Err("InvalidQueue: past tick".into());
                }
                sim.validate_military_pending(NationId(q.nation), &action)?;
                sim.queue.insert(
                    (q.tick, NationId(q.nation), q.sequence),
                    Command::Military(action),
                );
            }
        }
        Ok(sim)
    }
}
