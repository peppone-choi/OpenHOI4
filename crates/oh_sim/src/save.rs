//! The one current save body. Each variant wraps the layered DTO that matches
//! the systems the simulation runs; the variant is chosen from live state, so
//! the file carries one format version and no compatibility promise (ADR-1001).
use crate::{
    Simulation,
    economy_save::SimulationSaveV5,
    military_save::SimulationSaveV7,
    production_save::SimulationSaveV6,
    save_state::{RestoreContext, SimulationSaveV1, SimulationSaveV2, SimulationSaveV3},
    trigger_save::SimulationSaveV4,
};
use serde::{Deserialize, Serialize};

/// Postcard writes the variant index first; `bounds` in oh_save relies on this order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SimulationSave {
    Core(SimulationSaveV1),
    Movement(SimulationSaveV2),
    Strait(SimulationSaveV3),
    Trigger(SimulationSaveV4),
    Economy(SimulationSaveV5),
    Production(SimulationSaveV6),
    Military(SimulationSaveV7),
}

impl SimulationSave {
    /// The innermost layer, which holds date, tick, seed and world identity.
    pub fn core(&self) -> &SimulationSaveV1 {
        match self {
            Self::Core(d) => d,
            Self::Movement(d) => &d.base,
            Self::Strait(d) => &d.base.base,
            Self::Trigger(d) => &d.base.base.base,
            Self::Economy(d) => &d.base.base.base,
            Self::Production(d) => &d.base.base.base.base,
            Self::Military(d) => &d.base.base.base.base.base,
        }
    }
    pub fn has_military(&self) -> bool {
        matches!(self, Self::Military(_))
    }
    pub fn has_production(&self) -> bool {
        matches!(self, Self::Production(_) | Self::Military(_))
    }
    pub fn has_economy(&self) -> bool {
        matches!(
            self,
            Self::Economy(_) | Self::Production(_) | Self::Military(_)
        )
    }
    /// Only the dedicated trigger layer stores trigger state outside economy.
    pub fn is_trigger(&self) -> bool {
        matches!(self, Self::Trigger(_))
    }
}

impl Simulation {
    pub fn export_current_save(&self) -> Result<SimulationSave, String> {
        Ok(if self.military.is_some() {
            SimulationSave::Military(self.export_save_v8()?)
        } else if self.production.is_some() {
            SimulationSave::Production(self.export_save_v6()?)
        } else if self.economy.is_some() {
            SimulationSave::Economy(self.export_save_v5()?)
        } else if self.trigger_state().is_some() {
            SimulationSave::Trigger(self.export_save_v4()?)
        } else if self.movement().is_some_and(|m| m.has_strait_context()) {
            SimulationSave::Strait(self.export_save_v3()?)
        } else if self.movement().is_some() {
            SimulationSave::Movement(self.export_save_v2()?)
        } else {
            SimulationSave::Core(self.export_save()?)
        })
    }
    pub fn from_current_save(dto: SimulationSave, c: &RestoreContext) -> Result<Self, String> {
        match dto {
            SimulationSave::Core(d) => Self::from_save(d, c),
            SimulationSave::Movement(d) => Self::from_save_v2(d, c),
            SimulationSave::Strait(d) => Self::from_save_v3(d, c),
            SimulationSave::Trigger(d) => Self::from_save_v4(d, c),
            SimulationSave::Economy(d) => Self::from_save_v5(d, c),
            SimulationSave::Production(d) => Self::from_save_v6(d, c),
            SimulationSave::Military(d) => Self::from_save_v8(d, c),
        }
    }
}
