//! Standalone daily supply graph/formulas; no live simulation adapter yet.
use oh_core::{DivisionId, Fx, NationId, ProvinceId, Qty};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceId(pub u64);
pub type RailKey = (ProvinceId, ProvinceId);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceKind {
    Capital,
    Hub,
    Port,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LandKind {
    Normal,
    RiverSmall,
    RiverLarge,
    Impassable,
    Strait,
    Sea,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub controller: NationId,
    pub land: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandEdge {
    pub cost: Fx,
    pub kind: LandKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rail {
    pub cost: Fx,
    pub capacity: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    pub id: SourceId,
    pub province: ProvinceId,
    pub kind: SourceKind,
    pub capacity: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Demand {
    pub division: DivisionId,
    pub province: ProvinceId,
    pub amount: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Input {
    pub nation: NationId,
    pub capital: ProvinceId,
    pub nodes: BTreeMap<ProvinceId, Node>,
    pub land: BTreeMap<RailKey, LandEdge>,
    pub rails: BTreeMap<RailKey, Rail>,
    pub sources: Vec<Source>,
    pub demands: Vec<Demand>,
    pub decay: Fx,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidValue,
    InvalidReference,
    Duplicate,
    Overflow,
    Invariant,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLedger {
    pub feeder: Option<Vec<ProvinceId>>,
    pub dispatch: Qty,
    pub delivered: Qty,
    pub loss: Qty,
    pub unused: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Share {
    pub source: SourceId,
    pub distance: Fx,
    pub attenuation: Fx,
    pub weight: Fx,
    pub dispatch: Qty,
    pub delivered: Qty,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    ZeroDemand,
    Isolated,
    OutOfRange,
    NoCapacity,
    Allocated,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvinceLedger {
    pub demand: Qty,
    pub delivered: Qty,
    pub ratio: Fx,
    pub target: Fx,
    pub rounding_residual: Qty,
    pub reason: Reason,
    pub shares: Vec<Share>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RailLedger {
    pub dispatch: Qty,
    pub unused: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Day {
    pub sources: BTreeMap<SourceId, SourceLedger>,
    pub rails: BTreeMap<RailKey, RailLedger>,
    pub provinces: BTreeMap<ProvinceId, ProvinceLedger>,
}
mod allocation;
mod graph;
pub use allocation::calculate;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PenaltyDefines {
    pub organization_floor: Fx,
    pub attack_floor: Fx,
    pub speed_floor: Fx,
    pub starvation_threshold: Fx,
    pub grace_days: u32,
    pub attrition_at_zero: Fx,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Effects {
    pub organization: Fx,
    pub attack: Fx,
    pub speed: Fx,
    pub movement_time: Fx,
    pub starvation_days: u32,
    pub attrition_rate: Fx,
}
mod penalties;
pub use penalties::shortage_effects;
