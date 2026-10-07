//! Land movement authority. External systems supply explicit travel context;
//! this module does not infer supply, diplomatic access, or division equipment.
use crate::world::World;
use oh_core::{DivisionId, Fx, NationId, ProvinceId};
use oh_data::{
    DefineValue, Defines, Number,
    map::{EdgeKind, MapData, ProvinceKind},
};
use serde::Serialize;
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MovementError {
    MissingContext,
    InvalidReference,
    DuplicateUnit,
    InvalidValue,
    InvalidKind,
    Overflow,
    NoPath,
    NotOwner,
}
impl std::fmt::Display for MovementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "movement {self:?}")
    }
}
impl std::error::Error for MovementError {}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Factors {
    pub terrain: Fx,
    pub infrastructure: Fx,
    pub supply: Fx,
    pub river: Fx,
}
impl Factors {
    /// Definition coefficients come from defines; dynamic values are trusted,
    /// explicit time factors from the owning supply/infrastructure systems.
    pub fn from_defines(
        defs: &Defines,
        map: &MapData,
        from: ProvinceId,
        to: ProvinceId,
        infrastructure: Fx,
        supply: Fx,
    ) -> Result<Self, MovementError> {
        let p = map
            .provinces
            .iter()
            .find(|p| p.id == to.0)
            .ok_or(MovementError::InvalidReference)?;
        let edge = map
            .edge(from.0, to.0)
            .ok_or(MovementError::InvalidReference)?;
        let river = match edge.kind {
            EdgeKind::Normal => "normal",
            EdgeKind::RiverSmall => "small",
            EdgeKind::RiverLarge => "large",
            _ => return Err(MovementError::NoPath),
        };
        let get = |name: &str| match defs.get(name) {
            Some(DefineValue::Number(Number::Fixed(v))) => Ok(*v),
            Some(DefineValue::Number(Number::Integer(v))) => {
                Fx::checked_from_num(*v).ok_or(MovementError::Overflow)
            }
            _ => Err(MovementError::MissingContext),
        };
        let f = Self {
            terrain: get(&format!("movement.terrain_{}", p.terrain))?,
            river: get(&format!("movement.river_{river}"))?,
            infrastructure,
            supply,
        };
        f.validate()?;
        Ok(f)
    }
    pub(crate) fn validate(self) -> Result<(), MovementError> {
        if [self.terrain, self.infrastructure, self.supply, self.river]
            .iter()
            .any(|v| *v <= Fx::ZERO)
        {
            Err(MovementError::InvalidValue)
        } else {
            Ok(())
        }
    }
}
/// Crossing discriminator is authoritative, never supplied in client commands.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[repr(u8)]
pub enum CrossingKind {
    Normal,
    RiverSmall,
    RiverLarge,
    Strait,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct StraitFactors {
    pub terrain: Fx,
    pub infrastructure: Fx,
    pub supply: Fx,
    pub strait: Fx,
}
impl StraitFactors {
    pub fn from_defines(
        defs: &Defines,
        map: &MapData,
        from: ProvinceId,
        to: ProvinceId,
        infrastructure: Fx,
        supply: Fx,
    ) -> Result<Self, MovementError> {
        if !land(map, from) || !land(map, to) {
            return Err(MovementError::InvalidReference);
        }
        let edge = map
            .edge(from.0, to.0)
            .ok_or(MovementError::InvalidReference)?;
        if edge.kind != EdgeKind::Strait {
            return Err(MovementError::InvalidKind);
        }
        let p = &map.provinces[usize::from(
            map.province_index(to.0)
                .ok_or(MovementError::InvalidReference)?,
        )];
        let get = |key: &str| match defs.get(key) {
            Some(DefineValue::Number(Number::Fixed(v))) => Ok(*v),
            Some(DefineValue::Number(Number::Integer(v))) => {
                Fx::checked_from_num(*v).ok_or(MovementError::Overflow)
            }
            _ => Err(MovementError::MissingContext),
        };
        let factors = Self {
            terrain: get(&format!("movement.terrain_{}", p.terrain))?,
            infrastructure,
            supply,
            strait: get("movement.strait")?,
        };
        factors.validate()?;
        Ok(factors)
    }
    pub(crate) fn validate(self) -> Result<(), MovementError> {
        if [self.terrain, self.infrastructure, self.supply, self.strait]
            .iter()
            .any(|v| *v <= Fx::ZERO)
        {
            return Err(MovementError::InvalidValue);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct StraitContext {
    pub kind: CrossingKind,
    pub factors: StraitFactors,
}
pub type DirectedStraits = BTreeMap<(ProvinceId, ProvinceId), StraitContext>;
#[derive(Clone, Debug)]
pub struct StraitInput {
    pub unit: DivisionId,
    pub corrections: DirectedStraits,
}
#[derive(Clone, Debug)]
pub struct UnitInput {
    pub id: DivisionId,
    pub nation: NationId,
    pub province: ProvinceId,
    pub speed: Fx,
    pub allowed: BTreeSet<ProvinceId>,
    pub corrections: BTreeMap<(ProvinceId, ProvinceId), Factors>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct Leg {
    pub from: ProvinceId,
    pub to: ProvinceId,
    pub hours: Fx,
}
/// Read-only authority. Only the simulation command/phase can change it.
/// ```compile_fail
/// fn overwrite(unit: &mut oh_sim::movement::Unit) { unit.speed = oh_core::Fx::ONE; }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Unit {
    pub(crate) id: DivisionId,
    pub(crate) nation: NationId,
    pub(crate) province: ProvinceId,
    pub(crate) speed: Fx,
    pub(crate) allowed: BTreeSet<ProvinceId>,
    pub(crate) corrections: BTreeMap<(ProvinceId, ProvinceId), Factors>,
    pub(crate) route: Vec<Leg>,
    pub(crate) elapsed: Fx,
}
impl Unit {
    pub fn id(&self) -> DivisionId {
        self.id
    }
    pub fn nation(&self) -> NationId {
        self.nation
    }
    pub fn province(&self) -> ProvinceId {
        self.province
    }
    pub fn speed(&self) -> Fx {
        self.speed
    }
    pub fn elapsed(&self) -> Fx {
        self.elapsed
    }
    pub fn remaining_route(&self) -> Vec<ProvinceId> {
        self.route.iter().map(|l| l.to).collect()
    }
    pub fn remaining_hours(&self) -> Result<Fx, MovementError> {
        let remaining = self.elapsed.checked_neg().ok_or(MovementError::Overflow)?;
        self.route.iter().try_fold(remaining, |sum, l| {
            sum.checked_add(l.hours).ok_or(MovementError::Overflow)
        })
    }
    fn validate(
        &self,
        world: &World,
        straits: Option<&DirectedStraits>,
    ) -> Result<(), MovementError> {
        let map = world.defs().map();
        if world.nation(self.nation).is_none()
            || !land(map, self.province)
            || !self.allowed.contains(&self.province)
            || self.allowed.iter().any(|id| !land(map, *id))
        {
            return Err(MovementError::InvalidReference);
        }
        if self.speed <= Fx::ZERO {
            return Err(MovementError::InvalidValue);
        }
        for (&(a, b), f) in &self.corrections {
            if !usable(map, a, b) {
                return Err(MovementError::InvalidReference);
            }
            f.validate()?;
        }
        let mut from = self.province;
        for l in &self.route {
            if l.from != from
                || !self.allowed.contains(&l.to)
                || !route_usable(map, l.from, l.to, straits.is_some())
            {
                return Err(MovementError::InvalidReference);
            }
            if self.duration(map, l.from, l.to, straits)? != l.hours {
                return Err(MovementError::InvalidValue);
            }
            from = l.to;
        }
        self.remaining_hours()?;
        if self.elapsed < Fx::ZERO
            || self.route.first().is_none_or(|l| self.elapsed >= l.hours)
                && (self.elapsed != Fx::ZERO || !self.route.is_empty())
        {
            return Err(MovementError::InvalidValue);
        }
        Ok(())
    }
    fn duration(
        &self,
        map: &MapData,
        a: ProvinceId,
        b: ProvinceId,
        straits: Option<&DirectedStraits>,
    ) -> Result<Fx, MovementError> {
        let edge = map.edge(a.0, b.0).ok_or(MovementError::InvalidReference)?;
        if edge.kind == EdgeKind::Strait {
            let context = straits
                .and_then(|c| c.get(&(a, b)))
                .ok_or(MovementError::MissingContext)?;
            if context.kind != CrossingKind::Strait {
                return Err(MovementError::InvalidKind);
            }
            return crate::formula::strait_hours(edge.distance_km, self.speed, context.factors);
        }
        let f = self
            .corrections
            .get(&(a, b))
            .ok_or(MovementError::MissingContext)?;
        crate::formula::movement_hours(edge.distance_km, self.speed, *f)
    }
    fn path(
        &self,
        map: &MapData,
        start: ProvinceId,
        end: ProvinceId,
        straits: Option<&DirectedStraits>,
    ) -> Result<Vec<Leg>, MovementError> {
        if !land(map, end) || !self.allowed.contains(&end) {
            return Err(MovementError::InvalidReference);
        }
        if start == end {
            return Ok(Vec::new());
        }
        let mut adjacency: BTreeMap<ProvinceId, Vec<ProvinceId>> = BTreeMap::new();
        for e in &map.edges {
            let (a, b) = (ProvinceId(e.a), ProvinceId(e.b));
            if self.allowed.contains(&a)
                && self.allowed.contains(&b)
                && route_usable(map, a, b, straits.is_some())
            {
                adjacency.entry(a).or_default().push(b);
                adjacency.entry(b).or_default().push(a);
            }
        }
        let mut heap = BinaryHeap::from([Reverse((0i64, vec![start]))]);
        let mut best = BTreeMap::from([(start, (0i64, vec![start]))]);
        while let Some(Reverse((cost, path))) = heap.pop() {
            let a = *path.last().expect("nonempty path");
            if best.get(&a).is_none_or(|(c, p)| *c != cost || *p != path) {
                continue;
            }
            if a == end {
                return path
                    .windows(2)
                    .map(|p| {
                        Ok(Leg {
                            from: p[0],
                            to: p[1],
                            hours: self.duration(map, p[0], p[1], straits)?,
                        })
                    })
                    .collect();
            }
            for &b in adjacency.get(&a).into_iter().flatten() {
                if path.contains(&b) {
                    continue;
                }
                let next = cost
                    .checked_add(self.duration(map, a, b, straits)?.to_bits())
                    .ok_or(MovementError::Overflow)?;
                let mut p = path.clone();
                p.push(b);
                let candidate = (next, p);
                if best.get(&b).is_none_or(|old| &candidate < old) {
                    best.insert(b, candidate.clone());
                    heap.push(Reverse(candidate));
                }
            }
        }
        Err(MovementError::NoPath)
    }
}
pub(crate) fn land(map: &MapData, id: ProvinceId) -> bool {
    map.province_index(id.0)
        .is_some_and(|i| map.provinces[usize::from(i)].kind == ProvinceKind::Land)
}
fn usable(map: &MapData, a: ProvinceId, b: ProvinceId) -> bool {
    land(map, a)
        && land(map, b)
        && map.edge(a.0, b.0).is_some_and(|e| {
            matches!(
                e.kind,
                EdgeKind::Normal | EdgeKind::RiverSmall | EdgeKind::RiverLarge
            )
        })
}
fn route_usable(map: &MapData, a: ProvinceId, b: ProvinceId, explicit_straits: bool) -> bool {
    usable(map, a, b)
        || explicit_straits
            && land(map, a)
            && land(map, b)
            && map
                .edge(a.0, b.0)
                .is_some_and(|e| e.kind == EdgeKind::Strait)
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Movement {
    pub(crate) units: Vec<Unit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) straits: Option<BTreeMap<DivisionId, DirectedStraits>>,
}
impl Movement {
    pub fn new(world: &World, inputs: Vec<UnitInput>) -> Result<Self, MovementError> {
        let mut units: Vec<_> = inputs
            .into_iter()
            .map(|i| Unit {
                id: i.id,
                nation: i.nation,
                province: i.province,
                speed: i.speed,
                allowed: i.allowed,
                corrections: i.corrections,
                route: vec![],
                elapsed: Fx::ZERO,
            })
            .collect();
        units.sort_by_key(|u| u.id);
        let m = Self {
            units,
            straits: None,
        };
        m.validate(world)?;
        Ok(m)
    }
    /// Trusted, explicit crossing authority. Legacy constructors keep v2 policy.
    pub fn with_straits(
        world: &World,
        inputs: Vec<UnitInput>,
        contexts: Vec<StraitInput>,
    ) -> Result<Self, MovementError> {
        let mut movement = Self::new(world, inputs)?;
        let mut straits = BTreeMap::new();
        for context in contexts {
            if straits.insert(context.unit, context.corrections).is_some() {
                return Err(MovementError::DuplicateUnit);
            }
        }
        movement.straits = Some(straits);
        movement.validate(world)?;
        Ok(movement)
    }
    pub fn has_strait_context(&self) -> bool {
        self.straits.is_some()
    }
    pub fn strait_context(&self, id: DivisionId) -> Option<&DirectedStraits> {
        self.straits.as_ref()?.get(&id)
    }
    pub(crate) fn validate(&self, world: &World) -> Result<(), MovementError> {
        if !self.units.windows(2).all(|w| w[0].id < w[1].id) {
            return Err(MovementError::DuplicateUnit);
        }
        if let Some(straits) = &self.straits {
            if straits.keys().copied().ne(self.units.iter().map(|u| u.id)) {
                return Err(MovementError::MissingContext);
            }
            for u in &self.units {
                for (&(from, to), context) in &straits[&u.id] {
                    if !land(world.defs().map(), from)
                        || !land(world.defs().map(), to)
                        || !u.allowed.contains(&from)
                        || !u.allowed.contains(&to)
                    {
                        return Err(MovementError::InvalidReference);
                    }
                    let edge = world
                        .defs()
                        .map()
                        .edge(from.0, to.0)
                        .ok_or(MovementError::InvalidReference)?;
                    if context.kind != CrossingKind::Strait
                        || edge.kind != EdgeKind::Strait
                        || u.corrections.contains_key(&(from, to))
                    {
                        return Err(MovementError::InvalidKind);
                    }
                    context.factors.validate()?;
                }
            }
        }
        for u in &self.units {
            u.validate(world, self.strait_context(u.id))?;
        }
        Ok(())
    }
    pub fn units(&self) -> &[Unit] {
        &self.units
    }
    pub fn unit(&self, id: DivisionId) -> Option<&Unit> {
        self.units
            .binary_search_by_key(&id, |u| u.id)
            .ok()
            .map(|i| &self.units[i])
    }
    pub fn path(
        &self,
        world: &World,
        id: DivisionId,
        destination: ProvinceId,
    ) -> Result<Vec<ProvinceId>, MovementError> {
        let u = self.unit(id).ok_or(MovementError::InvalidReference)?;
        Ok(u.path(
            world.defs().map(),
            u.province,
            destination,
            self.strait_context(id),
        )?
        .iter()
        .map(|l| l.to)
        .collect())
    }
    pub(crate) fn command(
        &mut self,
        world: &World,
        nation: NationId,
        id: DivisionId,
        destination: Option<ProvinceId>,
    ) -> Result<(), MovementError> {
        let i = self
            .units
            .binary_search_by_key(&id, |u| u.id)
            .map_err(|_| MovementError::InvalidReference)?;
        let u = &self.units[i];
        if u.nation != nation {
            return Err(MovementError::NotOwner);
        }
        let keep = u.elapsed > Fx::ZERO;
        let mut route = if keep {
            vec![u.route[0].clone()]
        } else {
            vec![]
        };
        if let Some(end) = destination {
            let start = if keep { route[0].to } else { u.province };
            route.extend(u.path(world.defs().map(), start, end, self.strait_context(id))?);
        }
        route.iter().try_fold(-u.elapsed, |sum, l| {
            sum.checked_add(l.hours).ok_or(MovementError::Overflow)
        })?;
        self.units[i].route = route;
        Ok(())
    }
    pub(crate) fn advance(&mut self) -> Result<(), MovementError> {
        for u in &mut self.units {
            let mut budget = Fx::ONE;
            while let Some(l) = u.route.first() {
                let left = l
                    .hours
                    .checked_sub(u.elapsed)
                    .ok_or(MovementError::Overflow)?;
                if budget < left {
                    u.elapsed = u
                        .elapsed
                        .checked_add(budget)
                        .ok_or(MovementError::Overflow)?;
                    break;
                }
                budget = budget.checked_sub(left).ok_or(MovementError::Overflow)?;
                u.province = l.to;
                u.elapsed = Fx::ZERO;
                u.route.remove(0);
                if budget == Fx::ZERO {
                    break;
                }
            }
        }
        Ok(())
    }
}
