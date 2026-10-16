//! Read-only movement authority and commands; all travel rules stay in oh_sim.
use oh_core::{DivisionId, ProvinceId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum MovementCommand {
    Move { division: String, destination: u16 },
    Stop { division: String },
}
impl MovementCommand {
    pub fn into_action(self) -> Result<oh_sim::Command, &'static str> {
        fn id(text: String) -> Result<DivisionId, &'static str> {
            let value = text.parse::<u32>().map_err(|_| "invalid-message")?;
            if value.to_string() != text {
                return Err("invalid-message");
            }
            Ok(DivisionId(value))
        }
        Ok(match self {
            Self::Move {
                division,
                destination,
            } => oh_sim::Command::Move {
                unit: id(division)?,
                destination: ProvinceId(destination),
            },
            Self::Stop { division } => oh_sim::Command::Stop {
                unit: id(division)?,
            },
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MovementUnitView {
    pub division: String,
    pub nation: u16,
    pub province: u16,
    /// Current position is excluded; destinations of the remaining legs are ordered.
    pub route: Vec<u16>,
    pub elapsed: crate::FixedValue,
    /// Zero for a stationary unit.
    pub current_leg_hours: crate::FixedValue,
    pub remaining_hours: crate::FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MovementView {
    pub state_hash: String,
    pub tick: String,
    pub units: Vec<MovementUnitView>,
}
impl MovementView {
    pub fn from_sim(sim: &oh_sim::Simulation) -> Result<Self, &'static str> {
        let units = sim
            .movement()
            .map_or(&[][..], |movement| movement.units())
            .iter()
            .map(|unit| {
                Ok(MovementUnitView {
                    division: unit.id().0.to_string(),
                    nation: unit.nation().0,
                    province: unit.province().0,
                    route: unit.remaining_route().iter().map(|id| id.0).collect(),
                    elapsed: crate::FixedValue::fx(unit.elapsed()),
                    current_leg_hours: crate::FixedValue::fx(unit.current_leg_hours()),
                    remaining_hours: crate::FixedValue::fx(
                        unit.remaining_hours().map_err(|_| "simulation-error")?,
                    ),
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?;
        Ok(Self {
            state_hash: format!("{:016x}", sim.state_hash().map_err(|_| "simulation-error")?),
            tick: sim.snapshot().tick().to_string(),
            units,
        })
    }
}
pub(crate) fn declarations() -> Vec<String> {
    let config = ts_rs::Config::default();
    vec![
        MovementCommand::decl(&config),
        MovementUnitView::decl(&config),
        MovementView::decl(&config),
    ]
}
