//! Actual military authority contracts; whole counts and identifiers remain decimal text.
use oh_sim::military::Action;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum MilitaryCommand {
    Train {
        template: String,
    },
    Cancel {
        job: String,
    },
    Deploy {
        job: String,
        army: String,
        province: u16,
        allow_understrength: bool,
    },
    SetPriority {
        army: String,
        priority: u16,
    },
}
impl MilitaryCommand {
    pub fn into_action(self) -> Result<Action, String> {
        fn id(s: String) -> Result<u64, String> {
            let v = s.parse::<u64>().map_err(|_| "invalid military ID")?;
            if v.to_string() != s {
                return Err("noncanonical military ID".into());
            }
            Ok(v)
        }
        Ok(match self {
            Self::Train { template } => Action::Train { template },
            Self::Cancel { job } => Action::Cancel { job: id(job)? },
            Self::Deploy {
                job,
                army,
                province,
                allow_understrength,
            } => Action::Deploy {
                job: id(job)?,
                army: id(army)?,
                province: oh_core::ProvinceId(province),
                allow_understrength,
            },
            Self::SetPriority { army, priority } => Action::SetPriority {
                army: id(army)?,
                priority,
            },
        })
    }
    pub fn from_action(a: &Action) -> Self {
        match a {
            Action::Train { template } => Self::Train {
                template: template.clone(),
            },
            Action::Cancel { job } => Self::Cancel {
                job: job.to_string(),
            },
            Action::Deploy {
                job,
                army,
                province,
                allow_understrength,
            } => Self::Deploy {
                job: job.to_string(),
                army: army.to_string(),
                province: province.0,
                allow_understrength: *allow_understrength,
            },
            Action::SetPriority { army, priority } => Self::SetPriority {
                army: army.to_string(),
                priority: *priority,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryEquipmentView {
    pub model: String,
    pub count: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryNormalView {
    pub manpower: String,
    pub equipment: Vec<MilitaryEquipmentView>,
    pub strength: crate::FixedValue,
    pub soft_fire: crate::FixedValue,
    pub hard_fire: crate::FixedValue,
    pub defense: crate::FixedValue,
    pub breakthrough: crate::FixedValue,
    pub frontage: crate::FixedValue,
    pub supply_use: crate::FixedValue,
    pub organization: crate::FixedValue,
    pub armor: crate::FixedValue,
    pub piercing: crate::FixedValue,
    pub speed_kmh: crate::FixedValue,
    pub weighted_remainders: Vec<MilitaryRemainderView>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryRemainderView {
    pub field: String,
    pub numerator_remainder: String,
}
fn equipment(e: &std::collections::BTreeMap<String, i64>) -> Vec<MilitaryEquipmentView> {
    e.iter()
        .map(|(model, count)| MilitaryEquipmentView {
            model: model.clone(),
            count: count.to_string(),
        })
        .collect()
}
impl MilitaryNormalView {
    fn from_normal(n: &oh_sim::military_templates::Normal) -> Self {
        let s = &n.stats;
        Self {
            manpower: n.manpower.to_string(),
            equipment: equipment(&n.equipment),
            strength: crate::FixedValue::qty(s.strength),
            soft_fire: crate::FixedValue::qty(s.soft_fire),
            hard_fire: crate::FixedValue::qty(s.hard_fire),
            defense: crate::FixedValue::qty(s.defense),
            breakthrough: crate::FixedValue::qty(s.breakthrough),
            frontage: crate::FixedValue::qty(s.frontage),
            supply_use: crate::FixedValue::qty(s.supply_use),
            organization: crate::FixedValue::qty(s.organization),
            armor: crate::FixedValue::qty(s.armor),
            piercing: crate::FixedValue::qty(s.piercing),
            speed_kmh: crate::FixedValue::fx(s.speed_kmh),
            weighted_remainders: n
                .weighted_remainders
                .iter()
                .map(|(field, v)| MilitaryRemainderView {
                    field: field.clone(),
                    numerator_remainder: v.to_string(),
                })
                .collect(),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryArmyView {
    pub id: String,
    pub nation: u16,
    pub general: String,
    pub division_limit: u32,
    pub priority: u16,
    pub divisions: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryDivisionView {
    pub id: String,
    pub nation: u16,
    pub army: String,
    pub province: u16,
    pub template: String,
    pub normal: MilitaryNormalView,
    pub manpower: String,
    pub equipment: Vec<MilitaryEquipmentView>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub enum MilitaryJobStatus {
    Pending,
    Training,
    Ready,
    Cancelled,
    Deployed,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryJobView {
    pub id: String,
    pub nation: u16,
    pub template: String,
    pub normal: MilitaryNormalView,
    pub training_days: u32,
    pub status: MilitaryJobStatus,
    pub progress_days: u32,
    pub start_tick: Option<String>,
    pub reserved_manpower: String,
    pub equipment: Vec<MilitaryEquipmentView>,
    pub division: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryBackgroundView {
    pub nation: u16,
    pub committed: String,
    pub reserved: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryTemplateView {
    pub template: String,
    pub training_days: u32,
    pub normal: MilitaryNormalView,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryPendingView {
    pub tick: String,
    pub nation: u16,
    pub sequence: String,
    pub command: MilitaryCommand,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct MilitaryView {
    pub state_hash: String,
    pub definitions_hash: String,
    pub next_job_id: String,
    pub next_division_id: String,
    pub templates: Vec<MilitaryTemplateView>,
    pub armies: Vec<MilitaryArmyView>,
    pub divisions: Vec<MilitaryDivisionView>,
    pub jobs: Vec<MilitaryJobView>,
    pub background: Vec<MilitaryBackgroundView>,
    pub pending: Vec<MilitaryPendingView>,
}
impl MilitaryView {
    pub fn from_sim(s: &oh_sim::Simulation) -> Option<Self> {
        let m = s.military()?;
        let d = s.world()?.defs().military()?;
        Some(Self {
            state_hash: format!("{:016x}", s.state_hash().ok()?),
            definitions_hash: format!("{:016x}", m.definitions_hash()),
            next_job_id: m.next_job_id().to_string(),
            next_division_id: m.next_division_id().to_string(),
            templates: d
                .training_days
                .iter()
                .map(|(template, days)| {
                    Some(MilitaryTemplateView {
                        template: template.clone(),
                        training_days: *days,
                        normal: MilitaryNormalView::from_normal(
                            &oh_sim::military_templates::aggregate(d.templates(), template).ok()?,
                        ),
                    })
                })
                .collect::<Option<_>>()?,
            armies: m
                .armies()
                .values()
                .map(|a| MilitaryArmyView {
                    id: a.id().to_string(),
                    nation: a.nation().0,
                    general: a.general().into(),
                    division_limit: a.division_limit(),
                    priority: a.priority(),
                    divisions: m
                        .divisions()
                        .values()
                        .filter(|v| v.army() == a.id())
                        .map(|v| v.id().0.to_string())
                        .collect(),
                })
                .collect(),
            divisions: m
                .divisions()
                .values()
                .map(|v| MilitaryDivisionView {
                    id: v.id().0.to_string(),
                    nation: v.nation().0,
                    army: v.army().to_string(),
                    province: v.province().0,
                    template: v.template().into(),
                    normal: MilitaryNormalView::from_normal(v.normal()),
                    manpower: v.manpower().to_string(),
                    equipment: equipment(v.equipment()),
                })
                .collect(),
            jobs: m
                .jobs()
                .values()
                .map(|j| MilitaryJobView {
                    id: j.id().to_string(),
                    nation: j.nation().0,
                    template: j.template().into(),
                    normal: MilitaryNormalView::from_normal(j.normal()),
                    training_days: j.training_days(),
                    status: match j.status() {
                        oh_sim::military::JobStatus::Pending => MilitaryJobStatus::Pending,
                        oh_sim::military::JobStatus::Training => MilitaryJobStatus::Training,
                        oh_sim::military::JobStatus::Ready => MilitaryJobStatus::Ready,
                        oh_sim::military::JobStatus::Cancelled => MilitaryJobStatus::Cancelled,
                        oh_sim::military::JobStatus::Deployed => MilitaryJobStatus::Deployed,
                    },
                    progress_days: j.progress_days(),
                    start_tick: j.start_tick().map(|v| v.to_string()),
                    reserved_manpower: j.reserved_manpower().to_string(),
                    equipment: equipment(j.equipment()),
                    division: j.division().map(|v| v.0.to_string()),
                })
                .collect(),
            background: m
                .background()
                .iter()
                .map(|(nation, b)| MilitaryBackgroundView {
                    nation: *nation,
                    committed: b.committed.to_string(),
                    reserved: b.reserved.to_string(),
                })
                .collect(),
            pending: s
                .pending_commands()
                .iter()
                .filter_map(|(&(tick, nation, sequence), command)| match command {
                    oh_sim::Command::Military(a) => Some(MilitaryPendingView {
                        tick: tick.to_string(),
                        nation: nation.0,
                        sequence: sequence.to_string(),
                        command: MilitaryCommand::from_action(a),
                    }),
                    _ => None,
                })
                .collect(),
        })
    }
}
pub(crate) fn declarations() -> Vec<String> {
    let c = ts_rs::Config::default();
    vec![
        MilitaryCommand::decl(&c),
        MilitaryEquipmentView::decl(&c),
        MilitaryRemainderView::decl(&c),
        MilitaryNormalView::decl(&c),
        MilitaryArmyView::decl(&c),
        MilitaryDivisionView::decl(&c),
        MilitaryJobStatus::decl(&c),
        MilitaryJobView::decl(&c),
        MilitaryBackgroundView::decl(&c),
        MilitaryTemplateView::decl(&c),
        MilitaryPendingView::decl(&c),
        MilitaryView::decl(&c),
    ]
}
