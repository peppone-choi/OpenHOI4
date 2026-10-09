//! Typed economic authority query and commands; browsers do no game arithmetic.
use oh_core::{Fx, Qty};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum EconomyCommand {
    Allocate {
        ratios_bits: [String; 4],
    },
    Construct {
        project: String,
        state: u16,
        building: String,
    },
    Cancel {
        project: String,
    },
    Reorder {
        projects: Vec<String>,
    },
    ChangeLaw {
        law: String,
    },
}
impl EconomyCommand {
    pub fn into_action(self) -> Result<oh_sim::economy::Action, String> {
        fn number(v: &str) -> Result<u64, String> {
            let n = v.parse::<u64>().map_err(|_| "invalid project ID")?;
            if n.to_string() != v {
                return Err("noncanonical project ID".into());
            }
            Ok(n)
        }
        Ok(match self {
            Self::Allocate { ratios_bits } => {
                let mut ratios = [Fx::ZERO; 4];
                for (i, v) in ratios_bits.iter().enumerate() {
                    let n = v.parse::<i64>().map_err(|_| "invalid ratio bits")?;
                    if n.to_string() != *v {
                        return Err("noncanonical ratio bits".into());
                    }
                    ratios[i] = Fx::from_bits(n);
                }
                oh_sim::economy::Action::Allocate { ratios }
            }
            Self::Construct {
                project,
                state,
                building,
            } => oh_sim::economy::Action::Construct {
                project: number(&project)?,
                state,
                building,
            },
            Self::Cancel { project } => oh_sim::economy::Action::Cancel {
                project: number(&project)?,
            },
            Self::Reorder { projects } => oh_sim::economy::Action::Reorder {
                projects: projects
                    .iter()
                    .map(|v| number(v))
                    .collect::<Result<_, _>>()?,
            },
            Self::ChangeLaw { law } => oh_sim::economy::Action::ChangeLaw { law },
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct FixedValue {
    pub value: String,
    pub bits: String,
    pub fractional_bits: u8,
}
impl FixedValue {
    pub(crate) fn qty(v: Qty) -> Self {
        Self {
            value: v.to_string(),
            bits: v.to_bits().to_string(),
            fractional_bits: 16,
        }
    }
    pub(crate) fn fx(v: Fx) -> Self {
        Self {
            value: v.to_string(),
            bits: v.to_bits().to_string(),
            fractional_bits: 32,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct LawSelection {
    pub category: String,
    pub law: String,
    pub name_key: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct EconomyResource {
    pub resource: String,
    pub value: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct EconomyStateFlow {
    pub state: u16,
    pub population: String,
    pub resources: Vec<EconomyResource>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct IndustryContribution {
    pub state: u16,
    pub building: String,
    pub levels: String,
    pub unit_ic: FixedValue,
    pub value: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct IndustryMultiplier {
    pub source: String,
    pub factor: FixedValue,
    pub applied: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct EconomyLedgerView {
    pub tick: String,
    pub laws: Vec<LawSelection>,
    pub stability: FixedValue,
    pub capacity: String,
    pub contributions: Vec<IndustryContribution>,
    pub state_flows: Vec<EconomyStateFlow>,
    pub population: String,
    pub resources: Vec<EconomyResource>,
    pub multipliers: Vec<IndustryMultiplier>,
    pub total_ic: FixedValue,
    pub minimum: FixedValue,
    pub ratios: [FixedValue; 4],
    pub allocation: [FixedValue; 4],
    pub consumer_residual: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ConstructionProjectView {
    pub id: String,
    pub state: u16,
    pub building: String,
    pub target: String,
    pub progress: FixedValue,
    pub dormancy: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ConstructionEntryView {
    pub state: u16,
    pub building: String,
    pub target: String,
    pub starting_progress: FixedValue,
    pub cost: FixedValue,
    pub daily_cap: FixedValue,
    pub infrastructure: FixedValue,
    pub owner: u16,
    pub factor_evaluated: bool,
    pub project: String,
    pub factor: FixedValue,
    pub consumed: FixedValue,
    pub applied: FixedValue,
    pub discarded: FixedValue,
    pub completed: bool,
    pub dormancy: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ConstructionLedgerView {
    pub tick: String,
    pub budget: FixedValue,
    pub entries: Vec<ConstructionEntryView>,
    pub unused: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct EconomyNationView {
    pub nation: u16,
    pub laws: Vec<LawSelection>,
    pub political_capital: FixedValue,
    pub stability: FixedValue,
    pub mobilization: FixedValue,
    pub ratios: [FixedValue; 4],
    pub committed: String,
    pub reserved: String,
    pub capacity: String,
    pub available: String,
    pub overcommitted: String,
    pub projects: Vec<ConstructionProjectView>,
    pub ledger: EconomyLedgerView,
    pub construction: ConstructionLedgerView,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct EconomyView {
    pub state_hash: String,
    pub pending: Vec<EconomyPendingView>,
    pub definitions_hash: String,
    pub nations: Vec<EconomyNationView>,
    pub industrial_scores: Option<Vec<IndustrialScoreView>>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct IndustrialScoreView {
    pub nation: u16,
    pub tick: String,
    pub input_tick: String,
    pub weight: FixedValue,
    pub input: FixedValue,
    pub term: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
pub enum AuthorityCommandView {
    Pause { paused: bool },
    SetSpeed { speed: u8 },
    Move { unit: u32, destination: u16 },
    Stop { unit: u32 },
    Effects { program: String },
    Economy { command: EconomyCommand },
    Production { command: crate::ProductionCommand },
    Military { command: crate::MilitaryCommand },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct EconomyPendingView {
    pub tick: String,
    pub nation: u16,
    pub sequence: String,
    pub command: AuthorityCommandView,
}
fn action_view(action: &oh_sim::economy::Action) -> EconomyCommand {
    match action {
        oh_sim::economy::Action::Allocate { ratios } => EconomyCommand::Allocate {
            ratios_bits: ratios.map(|v| v.to_bits().to_string()),
        },
        oh_sim::economy::Action::Construct {
            project,
            state,
            building,
        } => EconomyCommand::Construct {
            project: project.to_string(),
            state: *state,
            building: building.clone(),
        },
        oh_sim::economy::Action::Cancel { project } => EconomyCommand::Cancel {
            project: project.to_string(),
        },
        oh_sim::economy::Action::Reorder { projects } => EconomyCommand::Reorder {
            projects: projects.iter().map(|v| v.to_string()).collect(),
        },
        oh_sim::economy::Action::ChangeLaw { law } => {
            EconomyCommand::ChangeLaw { law: law.clone() }
        }
    }
}
fn dormancy(v: Option<oh_sim::economy::Dormancy>) -> Option<String> {
    v.map(|v| {
        match v {
            oh_sim::economy::Dormancy::NoOwnership => "economy-no-ownership",
            oh_sim::economy::Dormancy::TargetConflict => "economy-target-conflict",
            oh_sim::economy::Dormancy::SlotCap => "economy-slot-cap",
            oh_sim::economy::Dormancy::ZeroCap => "economy-zero-cap",
            oh_sim::economy::Dormancy::ZeroFactor => "economy-zero-factor",
        }
        .into()
    })
}
impl EconomyView {
    pub fn from_sim(sim: &oh_sim::Simulation) -> Option<Self> {
        let e = sim.economy()?;
        let d = sim.world()?.defs().economy()?;
        Some(Self {
            state_hash: format!("{:016x}", sim.state_hash().ok()?),
            pending: sim
                .pending_commands()
                .iter()
                .map(|(&(tick, nation, sequence), c)| EconomyPendingView {
                    tick: tick.to_string(),
                    nation: nation.0,
                    sequence: sequence.to_string(),
                    command: match c {
                        oh_sim::Command::Pause(paused) => {
                            AuthorityCommandView::Pause { paused: *paused }
                        }
                        oh_sim::Command::SetSpeed(speed) => {
                            AuthorityCommandView::SetSpeed { speed: *speed }
                        }
                        oh_sim::Command::Move { unit, destination } => AuthorityCommandView::Move {
                            unit: unit.0,
                            destination: destination.0,
                        },
                        oh_sim::Command::Stop { unit } => {
                            AuthorityCommandView::Stop { unit: unit.0 }
                        }
                        oh_sim::Command::Effects { program } => AuthorityCommandView::Effects {
                            program: program.clone(),
                        },
                        oh_sim::Command::Production(action) => AuthorityCommandView::Production {
                            command: crate::ProductionCommand::from_action(action),
                        },
                        oh_sim::Command::Military(action) => AuthorityCommandView::Military {
                            command: crate::MilitaryCommand::from_action(action),
                        },
                        oh_sim::Command::Economy(action) => AuthorityCommandView::Economy {
                            command: action_view(action),
                        },
                    },
                })
                .collect(),
            definitions_hash: format!("{:016x}", e.definitions_hash()),
            nations: e
                .nations()
                .iter()
                .map(|(id, n)| {
                    let l = n.ledger();
                    let c = n.construction_ledger();
                    EconomyNationView {
                        nation: *id,
                        laws: n
                            .laws()
                            .iter()
                            .map(|(category, law)| LawSelection {
                                category: category.clone(),
                                law: law.clone(),
                                name_key: d.laws[law].name_key.clone(),
                            })
                            .collect(),
                        political_capital: FixedValue::qty(n.political_capital()),
                        stability: FixedValue::fx(n.stability()),
                        mobilization: FixedValue::fx(n.mobilization()),
                        ratios: n.allocation().map(FixedValue::fx),
                        committed: n.committed().to_string(),
                        reserved: n.reserved().to_string(),
                        capacity: n.capacity().to_string(),
                        available: n.available().to_string(),
                        overcommitted: n.overcommitted().to_string(),
                        projects: n
                            .projects()
                            .iter()
                            .map(|p| ConstructionProjectView {
                                id: p.id.to_string(),
                                state: p.state,
                                building: p.building.clone(),
                                target: p.target.to_string(),
                                progress: FixedValue::qty(p.progress),
                                dormancy: dormancy(p.dormancy),
                            })
                            .collect(),
                        ledger: EconomyLedgerView {
                            tick: l.tick.to_string(),
                            laws: l
                                .laws
                                .iter()
                                .map(|(category, law)| LawSelection {
                                    category: category.clone(),
                                    law: law.clone(),
                                    name_key: d.laws[law].name_key.clone(),
                                })
                                .collect(),
                            stability: FixedValue::fx(l.stability),
                            capacity: l.capacity.to_string(),
                            contributions: l
                                .contributions
                                .iter()
                                .map(|c| IndustryContribution {
                                    state: c.state,
                                    building: c.building.clone(),
                                    levels: c.levels.to_string(),
                                    unit_ic: FixedValue::qty(c.unit_ic),
                                    value: FixedValue::qty(c.value),
                                })
                                .collect(),
                            state_flows: l
                                .population_by_state
                                .iter()
                                .map(|(state, population)| EconomyStateFlow {
                                    state: *state,
                                    population: population.to_string(),
                                    resources: l.resources_by_state[state]
                                        .iter()
                                        .map(|(resource, value)| EconomyResource {
                                            resource: resource.clone(),
                                            value: value.to_string(),
                                        })
                                        .collect(),
                                })
                                .collect(),
                            population: l.population.to_string(),
                            resources: l
                                .resources
                                .iter()
                                .map(|(resource, value)| EconomyResource {
                                    resource: resource.clone(),
                                    value: value.to_string(),
                                })
                                .collect(),
                            multipliers: l
                                .multipliers
                                .iter()
                                .map(|m| IndustryMultiplier {
                                    source: m.source.clone(),
                                    factor: FixedValue::fx(m.factor),
                                    applied: FixedValue::qty(m.applied),
                                })
                                .collect(),
                            total_ic: FixedValue::qty(l.total_ic),
                            minimum: FixedValue::fx(l.minimum),
                            ratios: l.ratios.map(FixedValue::fx),
                            allocation: l.allocation.map(FixedValue::qty),
                            consumer_residual: FixedValue::qty(l.consumer_residual),
                        },
                        construction: ConstructionLedgerView {
                            tick: c.tick.to_string(),
                            budget: FixedValue::qty(c.budget),
                            entries: c
                                .entries
                                .iter()
                                .map(|v| ConstructionEntryView {
                                    state: v.state,
                                    building: v.building.clone(),
                                    target: v.target.to_string(),
                                    starting_progress: FixedValue::qty(v.starting_progress),
                                    cost: FixedValue::qty(v.cost),
                                    daily_cap: FixedValue::qty(v.daily_cap),
                                    infrastructure: FixedValue::fx(v.infrastructure),
                                    owner: v.owner,
                                    factor_evaluated: v.factor_evaluated,
                                    project: v.project.to_string(),
                                    factor: FixedValue::fx(v.factor),
                                    consumed: FixedValue::qty(v.consumed),
                                    applied: FixedValue::qty(v.applied),
                                    discarded: FixedValue::qty(v.discarded),
                                    completed: v.completed,
                                    dormancy: dormancy(v.dormancy),
                                })
                                .collect(),
                            unused: FixedValue::qty(c.unused),
                        },
                    }
                })
                .collect(),
            industrial_scores: e.industrial_scores().map(|scores| {
                scores
                    .iter()
                    .map(|(id, s)| IndustrialScoreView {
                        nation: *id,
                        tick: s.tick.to_string(),
                        input_tick: s.input_tick.to_string(),
                        weight: FixedValue::fx(s.weight),
                        input: FixedValue::qty(s.input),
                        term: FixedValue::qty(s.term),
                    })
                    .collect()
            }),
        })
    }
}
pub(crate) fn declarations() -> Vec<String> {
    let config = ts_rs::Config::default();
    vec![
        EconomyCommand::decl(&config),
        FixedValue::decl(&config),
        LawSelection::decl(&config),
        EconomyResource::decl(&config),
        EconomyStateFlow::decl(&config),
        IndustryContribution::decl(&config),
        IndustryMultiplier::decl(&config),
        EconomyLedgerView::decl(&config),
        ConstructionProjectView::decl(&config),
        ConstructionEntryView::decl(&config),
        ConstructionLedgerView::decl(&config),
        EconomyNationView::decl(&config),
        EconomyView::decl(&config),
        IndustrialScoreView::decl(&config),
        AuthorityCommandView::decl(&config),
        EconomyPendingView::decl(&config),
    ]
}
