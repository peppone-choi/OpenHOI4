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
