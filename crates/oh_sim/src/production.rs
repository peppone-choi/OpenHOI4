//! Private production authority; no resource stock, research or division rules.
use crate::{economy::Economy, formula, world::World};
use oh_core::{Fx, NationId, Qty};
use oh_data::{economy::quantity, production::Definition};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductionError {
    MissingContext,
    InvalidReference,
    InvalidValue,
    Overflow,
    NotOwner,
    InsufficientIC,
    LinesCap,
}
impl std::fmt::Display for ProductionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "production:{self:?}")
    }
}
impl From<formula::EconomyArithmeticError> for ProductionError {
    fn from(e: formula::EconomyArithmeticError) -> Self {
        match e {
            formula::EconomyArithmeticError::Overflow => Self::Overflow,
            _ => Self::InvalidValue,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Action {
    Create { model: String, requested_ic: Qty },
    SetIC { line: u64, requested_ic: Qty },
    Pause { line: u64, paused: bool },
    Switch { line: u64, model: String },
    Cancel { line: u64 },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    id: u64,
    nation: u16,
    model: String,
    requested_ic: Qty,
    paused: bool,
    efficiency: Fx,
    carry: Qty,
}
impl Line {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn nation(&self) -> NationId {
        NationId(self.nation)
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    pub fn requested_ic(&self) -> Qty {
        self.requested_ic
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn efficiency(&self) -> Fx {
        self.efficiency
    }
    pub fn carry(&self) -> Qty {
        self.carry
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Discard {
    pub tick: u64,
    pub line: u64,
    pub nation: u16,
    pub model: String,
    pub carry: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceEntry {
    pub required: Qty,
    pub reserved: Qty,
    pub debited: Qty,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineEntry {
    pub starting: Line,
    pub effective_ic: Qty,
    pub stability_factor: Fx,
    pub planned: Qty,
    pub fulfillment: Fx,
    pub actual: Qty,
    pub output: i64,
    pub ending_efficiency: Fx,
    pub ending_carry: Qty,
    pub resources: BTreeMap<String, ResourceEntry>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NationDay {
    pub budget: Qty,
    pub stability: Fx,
    pub flows: BTreeMap<String, i64>,
    pub unused_ic: Qty,
    pub lines: BTreeMap<u64, LineEntry>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Day {
    pub tick: u64,
    pub nations: BTreeMap<u16, NationDay>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Production {
    definitions_hash: u64,
    next_line_id: u64,
    lines: BTreeMap<u64, Line>,
    stock: BTreeMap<u16, BTreeMap<String, i64>>,
    day: Option<Day>,
    discards: Vec<Discard>,
}
fn defs(w: &World) -> Result<&Definition, ProductionError> {
    w.defs().production().ok_or(ProductionError::MissingContext)
}
fn q(v: &str) -> Result<Qty, ProductionError> {
    quantity(v).map_err(|_| ProductionError::InvalidValue)
}
impl Production {
    pub fn initial(w: &World) -> Result<Self, ProductionError> {
        let d = defs(w)?;
        Ok(Self {
            definitions_hash: d.identity().map_err(|_| ProductionError::InvalidValue)?,
            next_line_id: 0,
            lines: BTreeMap::new(),
            stock: d
                .nations
                .iter()
                .map(|(n, v)| (*n, v.stock.clone()))
                .collect(),
            day: None,
            discards: vec![],
        })
    }
    pub fn definitions_hash(&self) -> u64 {
        self.definitions_hash
    }
    pub fn next_line_id(&self) -> u64 {
        self.next_line_id
    }
    pub fn lines(&self) -> &BTreeMap<u64, Line> {
        &self.lines
    }
    pub fn line(&self, id: u64) -> Option<&Line> {
        self.lines.get(&id)
    }
    pub fn stocks(&self) -> &BTreeMap<u16, BTreeMap<String, i64>> {
        &self.stock
    }
    pub fn stock(&self, n: NationId, m: &str) -> Option<i64> {
        self.stock.get(&n.0)?.get(m).copied()
    }
    pub fn day(&self) -> Option<&Day> {
        self.day.as_ref()
    }
    pub fn discards(&self) -> &[Discard] {
        &self.discards
    }
    /// Trusted equipment producer API. A real consumer must own reservations.
    pub fn adjust_stock(
        &mut self,
        w: &World,
        n: NationId,
        m: &str,
        delta: i64,
    ) -> Result<(), ProductionError> {
        let d = defs(w)?;
        let t = d.tuning.as_ref().ok_or(ProductionError::MissingContext)?;
        let old = self.stock(n, m).ok_or(ProductionError::InvalidReference)?;
        let value = old.checked_add(delta).ok_or(ProductionError::Overflow)?;
        if value < 0 || value > t.inventory_count_limit {
            return Err(ProductionError::InvalidValue);
        }
        *self.stock.get_mut(&n.0).unwrap().get_mut(m).unwrap() = value;
        Ok(())
    }
    pub(crate) fn command(
        &mut self,
        w: &World,
        e: &Economy,
        n: NationId,
        a: &Action,
        tick: u64,
    ) -> Result<(), ProductionError> {
        let mut next = self.clone();
        next.apply(w, e, n, a, tick)?;
        *self = next;
        Ok(())
    }
    fn apply(
        &mut self,
        w: &World,
        e: &Economy,
        n: NationId,
        a: &Action,
        tick: u64,
    ) -> Result<(), ProductionError> {
        let d = defs(w)?;
        let t = d.tuning.as_ref().ok_or(ProductionError::MissingContext)?;
        let allowed = &d
            .nations
            .get(&n.0)
            .ok_or(ProductionError::InvalidReference)?
            .allowed_models;
        if let Action::Create {
            model,
            requested_ic,
        } = a
        {
            if !allowed.contains(model) {
                return Err(ProductionError::InvalidReference);
            }
            if *requested_ic < Qty::ZERO {
                return Err(ProductionError::InvalidValue);
            }
            if self.lines.len() as u64 >= t.lines_max as u64 {
                return Err(ProductionError::LinesCap);
            }
            let id = self.next_line_id;
            self.next_line_id = id.checked_add(1).ok_or(ProductionError::Overflow)?;
            self.lines.insert(
                id,
                Line {
                    id,
                    nation: n.0,
                    model: model.clone(),
                    requested_ic: *requested_ic,
                    paused: false,
                    efficiency: t.initial_efficiency,
                    carry: Qty::ZERO,
                },
            );
        } else {
            let id = match a {
                Action::SetIC { line, .. }
                | Action::Pause { line, .. }
                | Action::Switch { line, .. }
                | Action::Cancel { line } => *line,
                _ => unreachable!(),
            };
            let l = self
                .lines
                .get_mut(&id)
                .ok_or(ProductionError::InvalidReference)?;
            if l.nation != n.0 {
                return Err(ProductionError::NotOwner);
            }
            let mut discard = None;
            match a {
                Action::SetIC { requested_ic, .. } => {
                    if *requested_ic < Qty::ZERO {
                        return Err(ProductionError::InvalidValue);
                    }
                    l.requested_ic = *requested_ic;
                }
                Action::Pause { paused, .. } => l.paused = *paused,
                Action::Switch { model, .. } => {
                    if !allowed.contains(model) {
                        return Err(ProductionError::InvalidReference);
                    }
                    if *model != l.model {
                        let old = &d.models[&l.model];
                        let new = &d.models[model];
                        let retain = if old.family == new.family && new.generation > old.generation
                        {
                            t.same_family_newer_retention
                        } else {
                            t.other_retention
                        };
                        l.efficiency = l
                            .efficiency
                            .checked_mul(retain)
                            .ok_or(ProductionError::Overflow)?
                            .max(t.initial_efficiency)
                            .min(t.efficiency_cap);
                        discard = Some(Discard {
                            tick,
                            line: id,
                            nation: n.0,
                            model: l.model.clone(),
                            carry: l.carry,
                        });
                        l.carry = Qty::ZERO;
                        l.model = model.clone();
                    }
                }
                Action::Cancel { .. } => {
                    discard = Some(Discard {
                        tick,
                        line: id,
                        nation: n.0,
                        model: l.model.clone(),
                        carry: l.carry,
                    })
                }
                _ => unreachable!(),
            }
            if let Some(entry) = discard {
                if self.discards.first().is_some_and(|v| v.tick != tick) {
                    self.discards.clear();
                }
                if self.discards.len() as u64 >= t.lines_max as u64 {
                    return Err(ProductionError::LinesCap);
                }
                self.discards.push(entry);
            }
            if matches!(a, Action::Cancel { .. }) {
                self.lines.remove(&id);
            }
        }
        // Reduced IC budgets must not prevent a player from cancelling/pausing.
        let requires_budget = matches!(
            a,
            Action::Create { .. } | Action::SetIC { .. } | Action::Pause { paused: false, .. }
        );
        if requires_budget {
            let total = self
                .lines
                .values()
                .filter(|v| v.nation == n.0 && !v.paused)
                .try_fold(Qty::ZERO, |v, l| {
                    v.checked_add(l.requested_ic)
                        .ok_or(ProductionError::Overflow)
                })?;
            if total
                > e.nation(n)
                    .ok_or(ProductionError::MissingContext)?
                    .ledger()
                    .allocation[2]
            {
                return Err(ProductionError::InsufficientIC);
            }
        }
        Ok(())
    }
    pub(crate) fn daily(
        &mut self,
        w: &World,
        e: &Economy,
        tick: u64,
    ) -> Result<(), ProductionError> {
        let d = defs(w)?;
        let mut next = self.clone();
        let mut day = Day {
            tick,
            nations: BTreeMap::new(),
        };
        for n in d.nations.keys() {
            let economy = e
                .nation(NationId(*n))
                .ok_or(ProductionError::MissingContext)?;
            if economy.ledger().tick != tick {
                return Err(ProductionError::MissingContext);
            }
            let starts = self
                .lines
                .iter()
                .filter(|(_, l)| l.nation == *n)
                .map(|(id, l)| (*id, l.clone()))
                .collect();
            let entry = calculate_day(
                d,
                &starts,
                economy.ledger().allocation[2],
                &economy.ledger().resources,
                economy.stability(),
            )?;
            for (id, result) in &entry.lines {
                next.adjust_stock(w, NationId(*n), &result.starting.model, result.output)?;
                let l = next.lines.get_mut(id).unwrap();
                l.carry = result.ending_carry;
                l.efficiency = result.ending_efficiency;
            }
            day.nations.insert(*n, entry);
        }
        next.day = Some(day);
        *self = next;
        Ok(())
    }
    pub fn validate(&self, w: &World, tick: u64) -> Result<(), ProductionError> {
        let d = defs(w)?;
        let t = d.tuning.as_ref().ok_or(ProductionError::MissingContext)?;
        if self.definitions_hash != d.identity().map_err(|_| ProductionError::InvalidValue)?
            || self.lines.len() as u64 > t.lines_max as u64
            || self.stock.len() != d.nations.len()
        {
            return Err(ProductionError::InvalidValue);
        }
        for (n, input) in &d.nations {
            let stock = self.stock.get(n).ok_or(ProductionError::InvalidReference)?;
            if stock.keys().ne(input.stock.keys())
                || stock
                    .values()
                    .any(|v| *v < 0 || *v > t.inventory_count_limit)
            {
                return Err(ProductionError::InvalidValue);
            }
        }
        for (id, l) in &self.lines {
            if *id != l.id || *id >= self.next_line_id {
                return Err(ProductionError::InvalidValue);
            }
            validate_line(d, l)?;
        }
        if self.discards.len() as u64 > t.lines_max as u64 {
            return Err(ProductionError::InvalidValue);
        }
        for v in &self.discards {
            if v.tick > tick
                || v.line >= self.next_line_id
                || v.carry < Qty::ZERO
                || v.carry >= Qty::ONE
                || !d
                    .nations
                    .get(&v.nation)
                    .is_some_and(|n| n.allowed_models.contains(&v.model))
            {
                return Err(ProductionError::InvalidValue);
            }
        }
        if let Some(day) = &self.day {
            if day.tick == 0
                || day.tick != tick - tick % 24
                || day.tick % 24 != 0
                || day.nations.keys().ne(d.nations.keys())
            {
                return Err(ProductionError::InvalidValue);
            }
            let mut day_ids = std::collections::BTreeSet::new();
            for (nation, entry) in &day.nations {
                if entry.lines.iter().any(|(id, l)| {
                    *id >= self.next_line_id || l.starting.nation != *nation || !day_ids.insert(*id)
                }) || entry
                    .flows
                    .keys()
                    .any(|r| !w.defs().map().resource_ids().contains(r))
                {
                    return Err(ProductionError::InvalidValue);
                }
                let lines = entry
                    .lines
                    .iter()
                    .map(|(id, r)| (*id, r.starting.clone()))
                    .collect();
                if calculate_day(d, &lines, entry.budget, &entry.flows, entry.stability)? != *entry
                {
                    return Err(ProductionError::InvalidValue);
                }
            }
            if day_ids.len() as u64 > t.lines_max as u64 {
                return Err(ProductionError::InvalidValue);
            }
        } else if tick >= 24 {
            return Err(ProductionError::InvalidValue);
        }
        Ok(())
    }
}
fn validate_line(d: &Definition, l: &Line) -> Result<(), ProductionError> {
    let t = d.tuning.as_ref().ok_or(ProductionError::MissingContext)?;
    if !d
        .nations
        .get(&l.nation)
        .is_some_and(|n| n.allowed_models.contains(&l.model))
    {
        return Err(ProductionError::InvalidReference);
    }
    if l.requested_ic < Qty::ZERO
        || l.carry < Qty::ZERO
        || l.carry >= Qty::ONE
        || l.efficiency < t.initial_efficiency
        || l.efficiency > t.efficiency_cap
    {
        return Err(ProductionError::InvalidValue);
    }
    Ok(())
}
pub fn calculate_day(
    d: &Definition,
    starts: &BTreeMap<u64, Line>,
    budget: Qty,
    flows: &BTreeMap<String, i64>,
    stability: Fx,
) -> Result<NationDay, ProductionError> {
    let t = d.tuning.as_ref().ok_or(ProductionError::MissingContext)?;
    if budget < Qty::ZERO
        || !(Fx::ZERO..=Fx::ONE).contains(&stability)
        || flows.values().any(|v| *v < 0)
    {
        return Err(ProductionError::InvalidValue);
    }
    let weights: Vec<_> = starts
        .iter()
        .map(|(id, l)| {
            (
                *id,
                if l.paused {
                    0
                } else {
                    i128::from(l.requested_ic.to_bits())
                },
            )
        })
        .collect();
    let total = weights.iter().try_fold(0i128, |a, (_, v)| {
        a.checked_add(*v).ok_or(ProductionError::Overflow)
    })?;
    let allocated = if total > i128::from(budget.to_bits()) {
        formula::production_split(i128::from(budget.to_bits()), &weights)?
    } else {
        weights.iter().copied().collect()
    };
    let factor = t
        .stability_output_high
        .checked_sub(t.stability_output_low)
        .and_then(|v| v.checked_mul(stability))
        .and_then(|v| v.checked_add(t.stability_output_low))
        .ok_or(ProductionError::Overflow)?;
    let mut day = NationDay {
        budget,
        stability,
        flows: flows.clone(),
        unused_ic: budget,
        lines: BTreeMap::new(),
    };
    for (id, l) in starts {
        validate_line(d, l)?;
        if *id != l.id {
            return Err(ProductionError::InvalidValue);
        }
        let ic =
            Qty::from_bits(i64::try_from(allocated[id]).map_err(|_| ProductionError::Overflow)?);
        day.unused_ic = day
            .unused_ic
            .checked_sub(ic)
            .ok_or(ProductionError::Overflow)?;
        let work = formula::quantity_times_ratio(
            formula::quantity_times_ratio(ic, l.efficiency)?,
            factor,
        )?;
        let planned = formula::production_divide(work, q(&d.models[&l.model].unit_cost)?)?;
        let mut resources = BTreeMap::new();
        for (r, cost) in &d.models[&l.model].resources_per_item {
            resources.insert(
                r.clone(),
                ResourceEntry {
                    required: formula::production_resource(planned, q(cost)?)?,
                    reserved: Qty::ZERO,
                    debited: Qty::ZERO,
                },
            );
        }
        day.lines.insert(
            *id,
            LineEntry {
                starting: l.clone(),
                effective_ic: ic,
                stability_factor: factor,
                planned,
                fulfillment: Fx::ONE,
                actual: Qty::ZERO,
                output: 0,
                ending_efficiency: l.efficiency,
                ending_carry: l.carry,
                resources,
            },
        );
    }
    let resource_ids: std::collections::BTreeSet<_> = day
        .lines
        .values()
        .flat_map(|l| l.resources.keys().cloned())
        .collect();
    for r in resource_ids {
        let weights: Vec<_> = day
            .lines
            .iter()
            .filter_map(|(id, l)| {
                l.resources
                    .get(&r)
                    .map(|v| (*id, i128::from(v.required.to_bits())))
            })
            .collect();
        let total = weights.iter().try_fold(0i128, |a, (_, v)| {
            a.checked_add(*v).ok_or(ProductionError::Overflow)
        })?;
        let available = i128::from(flows.get(&r).copied().unwrap_or(0))
            .checked_mul(1 << 16)
            .ok_or(ProductionError::Overflow)?;
        let shares = if available < total {
            formula::production_split(available, &weights)?
        } else {
            weights.iter().copied().collect()
        };
        for (id, raw) in shares {
            let line = day.lines.get_mut(&id).unwrap();
            let v = line.resources.get_mut(&r).unwrap();
            v.reserved = Qty::from_bits(i64::try_from(raw).map_err(|_| ProductionError::Overflow)?);
            if v.required > Qty::ZERO {
                line.fulfillment = line.fulfillment.min(formula::production_ratio(
                    raw,
                    i128::from(v.required.to_bits()),
                )?);
            }
        }
    }
    for result in day.lines.values_mut() {
        result.actual = formula::quantity_times_ratio(result.planned, result.fulfillment)?;
        for (r, v) in &mut result.resources {
            v.debited = formula::production_resource(
                result.actual,
                q(&d.models[&result.starting.model].resources_per_item[r])?,
            )?;
            if v.debited > v.reserved {
                return Err(ProductionError::InvalidValue);
            }
        }
        // Whole-item output need not fit in Qty after adding the sub-item carry.
        // Accumulate raw quantities wide, then check the i64 count explicitly.
        let total = i128::from(result.starting.carry.to_bits())
            .checked_add(i128::from(result.actual.to_bits()))
            .ok_or(ProductionError::Overflow)?;
        result.output = i64::try_from(total >> 16).map_err(|_| ProductionError::Overflow)?;
        result.ending_carry = Qty::from_bits((total & 0xffff) as i64);
        if result.actual > Qty::ZERO {
            result.ending_efficiency = result
                .starting
                .efficiency
                .checked_add(t.daily_efficiency_growth)
                .ok_or(ProductionError::Overflow)?
                .min(t.efficiency_cap);
        }
    }
    Ok(day)
}
