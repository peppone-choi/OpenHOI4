//! Calendar unit conversions (not configurable gameplay coefficients).
use crate::{Date, Error};
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
