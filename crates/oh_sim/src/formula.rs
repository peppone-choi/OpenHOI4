//! Authoritative formulas and calendar unit conversions (DR-09).
use crate::ledger::{LedgerError, Modifier, ModifierOp};
use crate::{Date, Error};
use oh_core::Fx;

/// 02 §6.3: base, then source-ordered Adds, then source-ordered Muls.
/// checked_mul uses fixed's default floor rounding at each multiplication.
/// Never regroup/fuse products: that changes intermediate fixed-point bits.
pub fn stat_value(
    base: Fx,
    target_stat: &str,
    modifiers: &[Modifier],
    at_tick: u64,
) -> Result<Fx, LedgerError> {
    visit_stat(base, target_stat, modifiers, at_tick, |_, _| {})
}

/// Shared arithmetic path for actual application and ledger trace generation.
pub(crate) fn visit_stat(
    base: Fx,
    target_stat: &str,
    modifiers: &[Modifier],
    at_tick: u64,
    mut record: impl FnMut(&Modifier, Fx),
) -> Result<Fx, LedgerError> {
    if target_stat.is_empty() {
        return Err(LedgerError::EmptyTargetStat);
    }
    let mut active: Vec<_> = modifiers
        .iter()
        .filter(|m| m.target_stat == target_stat && m.expires.is_none_or(|expiry| at_tick < expiry))
        .collect();
    active.sort_by_key(|m| (m.op, &m.source));
    // Reject ambiguous ties before arithmetic; never rely on insertion order.
    for (index, m) in active.iter().enumerate() {
        if m.source.is_empty() {
            return Err(LedgerError::EmptySource);
        }
        if index > 0 && active[index - 1].op == m.op && active[index - 1].source == m.source {
            return Err(LedgerError::DuplicateModifier {
                source: m.source.clone(),
                op: m.op,
            });
        }
    }
    let mut value = base;
    for m in active {
        value = match m.op {
            ModifierOp::Add => value.checked_add(m.value),
            ModifierOp::Mul => value.checked_mul(m.value),
        }
        .ok_or_else(|| LedgerError::Overflow {
            source: m.source.clone(),
            op: m.op,
        })?;
        record(m, value);
    }
    Ok(value)
}
/// 01 §3.1 fixes one tick to one hour. A civil day contains 24 hours.
pub const HOURS_PER_DAY: u64 = 24;
pub fn ticks_for_days(days: u64) -> Result<u64, Error> {
    days.checked_mul(HOURS_PER_DAY).ok_or(Error::ClockOverflow)
}
pub(crate) fn days_in_month(year: u32, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        _ => 31,
    }
}
pub(crate) fn next_hour(date: Date, hour: u8) -> Result<(Date, u8), Error> {
    let hour = hour + 1;
    if u64::from(hour) < HOURS_PER_DAY {
        return Ok((date, hour));
    }
    let (year, month, day) = (date.year(), date.month(), date.day());
    let date = if day < days_in_month(year, month) {
        Date::new(year, month, day + 1)?
    } else if month < 12 {
        Date::new(year, month + 1, 1)?
    } else {
        Date::new(year.checked_add(1).ok_or(Error::ClockOverflow)?, 1, 1)?
    };
    Ok((date, 0))
}

/// REQ-MIL-04: time factors, in the exact ADR-1701 evaluation order.
pub fn movement_hours(
    distance: Fx,
    speed: Fx,
    factors: crate::movement::Factors,
) -> Result<Fx, crate::movement::MovementError> {
    use crate::movement::MovementError;
    if distance <= Fx::ZERO || speed <= Fx::ZERO {
        return Err(MovementError::InvalidValue);
    }
    factors.validate()?;
    let mut value = distance.checked_div(speed).ok_or(MovementError::Overflow)?;
    for factor in [
        factors.terrain,
        factors.infrastructure,
        factors.supply,
        factors.river,
    ] {
        value = value.checked_mul(factor).ok_or(MovementError::Overflow)?;
    }
    if value <= Fx::ZERO {
        return Err(MovementError::InvalidValue);
    }
    Ok(value)
}

/// Strait uses its own final time slot. No river factor participates.
pub fn strait_hours(
    distance: Fx,
    speed: Fx,
    factors: crate::movement::StraitFactors,
) -> Result<Fx, crate::movement::MovementError> {
    use crate::movement::MovementError;
    if distance <= Fx::ZERO || speed <= Fx::ZERO {
        return Err(MovementError::InvalidValue);
    }
    factors.validate()?;
    let mut value = distance.checked_div(speed).ok_or(MovementError::Overflow)?;
    for factor in [
        factors.terrain,
        factors.infrastructure,
        factors.supply,
        factors.strait,
    ] {
        value = value.checked_mul(factor).ok_or(MovementError::Overflow)?;
    }
    if value <= Fx::ZERO {
        return Err(MovementError::InvalidValue);
    }
    Ok(value)
}

/// Standalone arithmetic awaits real four-axis producers; missing nonzero input fails.
pub fn weighted_score(
    weights: [Fx; 4],
    inputs: [Option<Fx>; 4],
) -> Result<([Option<Fx>; 4], Fx), crate::trigger::TriggerError> {
    let mut terms = [None; 4];
    let mut total = Fx::ZERO;
    for i in 0..4 {
        if weights[i] == Fx::ZERO {
            continue;
        }
        let input = inputs[i].ok_or(crate::trigger::TriggerError::MissingContext)?;
        let term = weights[i]
            .checked_mul(input)
            .ok_or(crate::trigger::TriggerError::InvalidArgument)?;
        total = total
            .checked_add(term)
            .ok_or(crate::trigger::TriggerError::InvalidArgument)?;
        terms[i] = Some(term);
    }
    Ok((terms, total))
}

// Economy arithmetic uses wide integer intermediates; no Qty -> Fx narrowing.
use oh_core::Qty;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EconomyArithmeticError {
    InvalidValue,
    Overflow,
}
fn qty_bits(bits: i128) -> Result<Qty, EconomyArithmeticError> {
    Ok(Qty::from_bits(
        i64::try_from(bits).map_err(|_| EconomyArithmeticError::Overflow)?,
    ))
}
pub fn quantity_times_ratio(quantity: Qty, ratio: Fx) -> Result<Qty, EconomyArithmeticError> {
    if quantity < Qty::ZERO || ratio < Fx::ZERO {
        return Err(EconomyArithmeticError::InvalidValue);
    }
    qty_bits((i128::from(quantity.to_bits()) * i128::from(ratio.to_bits())) >> 32)
}
pub fn economy_minimum(base: Fx, slope: Fx, stability: Fx) -> Result<Fx, EconomyArithmeticError> {
    if !(Fx::ZERO..=Fx::ONE).contains(&base)
        || slope < Fx::ZERO
        || !(Fx::ZERO..=Fx::ONE).contains(&stability)
    {
        return Err(EconomyArithmeticError::InvalidValue);
    }
    let product = slope
        .checked_mul(Fx::ONE - stability)
        .ok_or(EconomyArithmeticError::Overflow)?;
    let minimum = base
        .checked_add(product)
        .ok_or(EconomyArithmeticError::Overflow)?;
    Ok(minimum.min(Fx::ONE))
}
pub fn economy_allocate(
    total: Qty,
    ratios: [Fx; 4],
    minimum: Fx,
) -> Result<[Qty; 4], EconomyArithmeticError> {
    if total < Qty::ZERO
        || !(Fx::ZERO..=Fx::ONE).contains(&minimum)
        || ratios.iter().any(|r| !(Fx::ZERO..=Fx::ONE).contains(r))
        || ratios.iter().map(|r| i128::from(r.to_bits())).sum::<i128>()
            != i128::from(Fx::ONE.to_bits())
        || ratios[0] < minimum
    {
        return Err(EconomyArithmeticError::InvalidValue);
    }
    let mut values = [Qty::ZERO; 4];
    let mut used = Qty::ZERO;
    for i in 1..4 {
        values[i] = quantity_times_ratio(total, ratios[i])?;
        used = used
            .checked_add(values[i])
            .ok_or(EconomyArithmeticError::Overflow)?;
    }
    values[0] = total
        .checked_sub(used)
        .ok_or(EconomyArithmeticError::Overflow)?;
    Ok(values)
}
/// (raw IC consumed, progress applied up to cost, effective rounding discard).
pub fn economy_construction(
    available: Qty,
    remaining: Qty,
    factor: Fx,
) -> Result<(Qty, Qty, Qty), EconomyArithmeticError> {
    if available < Qty::ZERO || remaining <= Qty::ZERO || factor < Fx::ZERO {
        return Err(EconomyArithmeticError::InvalidValue);
    }
    if available == Qty::ZERO || factor == Fx::ZERO {
        return Ok((Qty::ZERO, Qty::ZERO, Qty::ZERO));
    }
    let numerator = i128::from(remaining.to_bits()) << 32;
    let denominator = i128::from(factor.to_bits());
    let needed = (numerator + denominator - 1) / denominator;
    let consumed = qty_bits(needed.min(i128::from(available.to_bits())))?;
    let effective = quantity_times_ratio(consumed, factor)?;
    let applied = effective.min(remaining);
    Ok((consumed, applied, effective - applied))
}
pub fn economy_capacity(population: i64, ratio: Fx) -> Result<i64, EconomyArithmeticError> {
    if population < 0 || !(Fx::ZERO..=Fx::ONE).contains(&ratio) {
        return Err(EconomyArithmeticError::InvalidValue);
    }
    i64::try_from((i128::from(population) * i128::from(ratio.to_bits())) >> 32)
        .map_err(|_| EconomyArithmeticError::Overflow)
}
pub fn economy_political_income(
    current: Qty,
    daily: Qty,
    cap: Qty,
) -> Result<Qty, EconomyArithmeticError> {
    if current < Qty::ZERO || daily < Qty::ZERO || cap < Qty::ZERO || current > cap {
        return Err(EconomyArithmeticError::InvalidValue);
    }
    qty_bits(
        (i128::from(current.to_bits()) + i128::from(daily.to_bits()))
            .min(i128::from(cap.to_bits())),
    )
}

/// A score term can be signed; its actual IC input is a nonnegative Qty.
pub fn economy_weighted_quantity(quantity: Qty, weight: Fx) -> Result<Qty, EconomyArithmeticError> {
    if quantity < Qty::ZERO {
        return Err(EconomyArithmeticError::InvalidValue);
    }
    qty_bits((i128::from(quantity.to_bits()) * i128::from(weight.to_bits())) >> 32)
}
