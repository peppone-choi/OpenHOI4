//! Production authority projection. Decimal raw strings preserve every bit.
use crate::FixedValue;
use oh_core::Qty;
use oh_sim::production::Action;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ProductionCommand {
    Create {
        model: String,
        requested_ic_bits: String,
    },
    SetIC {
        line: String,
        requested_ic_bits: String,
    },
    Pause {
        line: String,
        paused: bool,
    },
    Switch {
        line: String,
        model: String,
    },
    Cancel {
        line: String,
    },
}
impl ProductionCommand {
    pub fn into_action(self) -> Result<Action, String> {
        fn id(s: String) -> Result<u64, String> {
            let v = s.parse::<u64>().map_err(|_| "invalid line ID")?;
            if v.to_string() != s {
                return Err("noncanonical line ID".into());
            }
            Ok(v)
        }
        fn q(s: String) -> Result<Qty, String> {
            let v = s.parse::<i64>().map_err(|_| "invalid IC bits")?;
            if v.to_string() != s || v < 0 {
                return Err("noncanonical/negative IC".into());
            }
            Ok(Qty::from_bits(v))
        }
        Ok(match self {
            Self::Create {
                model,
                requested_ic_bits,
            } => Action::Create {
                model,
                requested_ic: q(requested_ic_bits)?,
            },
            Self::SetIC {
                line,
                requested_ic_bits,
            } => Action::SetIC {
                line: id(line)?,
                requested_ic: q(requested_ic_bits)?,
            },
            Self::Pause { line, paused } => Action::Pause {
                line: id(line)?,
                paused,
            },
            Self::Switch { line, model } => Action::Switch {
                line: id(line)?,
                model,
            },
            Self::Cancel { line } => Action::Cancel { line: id(line)? },
        })
    }
    pub fn from_action(a: &Action) -> Self {
        match a {
            Action::Create {
                model,
                requested_ic,
            } => Self::Create {
                model: model.clone(),
                requested_ic_bits: requested_ic.to_bits().to_string(),
            },
            Action::SetIC { line, requested_ic } => Self::SetIC {
                line: line.to_string(),
                requested_ic_bits: requested_ic.to_bits().to_string(),
            },
            Action::Pause { line, paused } => Self::Pause {
                line: line.to_string(),
                paused: *paused,
            },
            Action::Switch { line, model } => Self::Switch {
                line: line.to_string(),
                model: model.clone(),
            },
            Action::Cancel { line } => Self::Cancel {
                line: line.to_string(),
            },
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionModelView {
    pub model: String,
    pub name_key: String,
    pub family: String,
    pub generation: u32,
    pub unit_cost: FixedValue,
    pub resources: Vec<ProductionModelResourceView>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionModelResourceView {
    pub resource: String,
    pub per_item: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionStockView {
    pub model: String,
    pub available: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionNationView {
    pub nation: u16,
    pub allowed_models: Vec<String>,
    pub stock: Vec<ProductionStockView>,
    pub military_ic: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionLineView {
    pub id: String,
    pub nation: u16,
    pub model: String,
    pub requested_ic: FixedValue,
    pub paused: bool,
    pub efficiency: FixedValue,
    pub carry: FixedValue,
}
impl ProductionLineView {
    fn from_line(l: &oh_sim::production::Line) -> Self {
        Self {
            id: l.id().to_string(),
            nation: l.nation().0,
            model: l.model().into(),
            requested_ic: FixedValue::qty(l.requested_ic()),
            paused: l.paused(),
            efficiency: FixedValue::fx(l.efficiency()),
            carry: FixedValue::qty(l.carry()),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionResourceView {
    pub resource: String,
    pub required: FixedValue,
    pub reserved: FixedValue,
    pub debited: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionLineDayView {
    pub starting: ProductionLineView,
    pub effective_ic: FixedValue,
    pub stability_factor: FixedValue,
    pub planned: FixedValue,
    pub fulfillment: FixedValue,
    pub actual: FixedValue,
    pub output: String,
    pub ending_efficiency: FixedValue,
    pub ending_carry: FixedValue,
    pub resources: Vec<ProductionResourceView>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionFlowView {
    pub resource: String,
    pub flow: String,
    pub required_raw: String,
    pub reserved_raw: String,
    pub debited_raw: String,
    pub unused_raw: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionNationDayView {
    pub nation: u16,
    pub budget: FixedValue,
    pub stability: FixedValue,
    pub unused_ic: FixedValue,
    pub flows: Vec<ProductionFlowView>,
    pub lines: Vec<ProductionLineDayView>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionDayView {
    pub tick: String,
    pub nations: Vec<ProductionNationDayView>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionDiscardView {
    pub tick: String,
    pub line: String,
    pub nation: u16,
    pub model: String,
    pub carry: FixedValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionPendingView {
    pub tick: String,
    pub nation: u16,
    pub sequence: String,
    pub command: ProductionCommand,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct ProductionView {
    pub state_hash: String,
    pub definitions_hash: String,
    pub next_line_id: String,
    pub models: Vec<ProductionModelView>,
    pub nations: Vec<ProductionNationView>,
    pub lines: Vec<ProductionLineView>,
    pub day: Option<ProductionDayView>,
    pub discards: Vec<ProductionDiscardView>,
    pub pending: Vec<ProductionPendingView>,
}
impl ProductionView {
    pub fn from_sim(s: &oh_sim::Simulation) -> Option<Self> {
        let p = s.production()?;
        let d = s.world()?.defs().production()?;
        let e = s.economy()?;
        Some(Self {
            state_hash: format!("{:016x}", s.state_hash().ok()?),
            definitions_hash: format!("{:016x}", p.definitions_hash()),
            next_line_id: p.next_line_id().to_string(),
            models: d
                .models
                .iter()
                .map(|(id, m)| {
                    Some(ProductionModelView {
                        model: id.clone(),
                        name_key: m.name_key.clone(),
                        family: m.family.clone(),
                        generation: m.generation,
                        unit_cost: FixedValue::qty(m.unit_cost.parse::<Qty>().ok()?),
                        resources: m
                            .resources_per_item
                            .iter()
                            .map(|(r, c)| {
                                Some(ProductionModelResourceView {
                                    resource: r.clone(),
                                    per_item: FixedValue::qty(c.parse::<Qty>().ok()?),
                                })
                            })
                            .collect::<Option<_>>()?,
                    })
                })
                .collect::<Option<_>>()?,
            nations: d
                .nations
                .iter()
                .map(|(n, input)| {
                    Some(ProductionNationView {
                        nation: *n,
                        allowed_models: input.allowed_models.iter().cloned().collect(),
                        stock: p
                            .stocks()
                            .get(n)?
                            .iter()
                            .map(|(m, v)| ProductionStockView {
                                model: m.clone(),
                                available: v.to_string(),
                            })
                            .collect(),
                        military_ic: FixedValue::qty(
                            e.nation(oh_core::NationId(*n))?.ledger().allocation[2],
                        ),
                    })
                })
                .collect::<Option<_>>()?,
            lines: p
                .lines()
                .values()
                .map(ProductionLineView::from_line)
                .collect(),
            day: p.day().map(|day| ProductionDayView {
                tick: day.tick.to_string(),
                nations: day
                    .nations
                    .iter()
                    .map(|(n, v)| ProductionNationDayView {
                        nation: *n,
                        budget: FixedValue::qty(v.budget),
                        stability: FixedValue::fx(v.stability),
                        unused_ic: FixedValue::qty(v.unused_ic),
                        flows: v
                            .flows
                            .keys()
                            .cloned()
                            .chain(v.lines.values().flat_map(|l| l.resources.keys().cloned()))
                            .collect::<std::collections::BTreeSet<_>>()
                            .iter()
                            .map(|r| {
                                let flow = v.flows.get(r).copied().unwrap_or(0);
                                let mut required = 0i128;
                                let mut reserved = 0i128;
                                let mut debited = 0i128;
                                for l in v.lines.values() {
                                    if let Some(x) = l.resources.get(r) {
                                        required += i128::from(x.required.to_bits());
                                        reserved += i128::from(x.reserved.to_bits());
                                        debited += i128::from(x.debited.to_bits());
                                    }
                                }
                                ProductionFlowView {
                                    resource: r.clone(),
                                    flow: flow.to_string(),
                                    required_raw: required.to_string(),
                                    reserved_raw: reserved.to_string(),
                                    debited_raw: debited.to_string(),
                                    unused_raw: (i128::from(flow) * (1 << 16) - debited)
                                        .to_string(),
                                }
                            })
                            .collect(),
                        lines: v
                            .lines
                            .values()
                            .map(|l| ProductionLineDayView {
                                starting: ProductionLineView::from_line(&l.starting),
                                effective_ic: FixedValue::qty(l.effective_ic),
                                stability_factor: FixedValue::fx(l.stability_factor),
                                planned: FixedValue::qty(l.planned),
                                fulfillment: FixedValue::fx(l.fulfillment),
                                actual: FixedValue::qty(l.actual),
                                output: l.output.to_string(),
                                ending_efficiency: FixedValue::fx(l.ending_efficiency),
                                ending_carry: FixedValue::qty(l.ending_carry),
                                resources: l
                                    .resources
                                    .iter()
                                    .map(|(r, x)| ProductionResourceView {
                                        resource: r.clone(),
                                        required: FixedValue::qty(x.required),
                                        reserved: FixedValue::qty(x.reserved),
                                        debited: FixedValue::qty(x.debited),
                                    })
                                    .collect(),
                            })
                            .collect(),
                    })
                    .collect(),
            }),
            discards: p
                .discards()
                .iter()
                .map(|v| ProductionDiscardView {
                    tick: v.tick.to_string(),
                    line: v.line.to_string(),
                    nation: v.nation,
                    model: v.model.clone(),
                    carry: FixedValue::qty(v.carry),
                })
                .collect(),
            pending: s
                .pending_commands()
                .iter()
                .filter_map(|(&(tick, n, seq), c)| {
                    if let oh_sim::Command::Production(a) = c {
                        Some(ProductionPendingView {
                            tick: tick.to_string(),
                            nation: n.0,
                            sequence: seq.to_string(),
                            command: ProductionCommand::from_action(a),
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        })
    }
}
pub(crate) fn declarations() -> Vec<String> {
    let c = ts_rs::Config::default();
    vec![
        ProductionCommand::decl(&c),
        ProductionModelView::decl(&c),
        ProductionModelResourceView::decl(&c),
        ProductionStockView::decl(&c),
        ProductionNationView::decl(&c),
        ProductionLineView::decl(&c),
        ProductionResourceView::decl(&c),
        ProductionLineDayView::decl(&c),
        ProductionFlowView::decl(&c),
        ProductionNationDayView::decl(&c),
        ProductionDayView::decl(&c),
        ProductionDiscardView::decl(&c),
        ProductionPendingView::decl(&c),
        ProductionView::decl(&c),
    ]
}
