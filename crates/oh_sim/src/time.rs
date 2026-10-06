use crate::{Error, formula};
use oh_data::{DefineValue, Defines, Number};
use serde::Serialize;
/// Proleptic Gregorian calendar, years 1..=u32::MAX (technical M0 convention).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Date {
    year: u32,
    month: u8,
    day: u8,
}
impl Date {
    pub fn new(year: u32, month: u8, day: u8) -> Result<Self, Error> {
        if year == 0
            || !(1..=12).contains(&month)
            || day == 0
            || day > formula::days_in_month(year, month)
        {
            return Err(Error::InvalidDate);
        }
        Ok(Self { year, month, day })
    }
    pub fn year(self) -> u32 {
        self.year
    }
    pub fn month(self) -> u8 {
        self.month
    }
    pub fn day(self) -> u8 {
        self.day
    }
}
impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
/// Host pacing values are data, never a simulation sleep or time measurement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TimeConfig {
    speed_ms_per_tick: [u64; 5],
    initial_speed: u8,
}
impl TimeConfig {
    pub fn new(speed_ms_per_tick: [u64; 5], initial_speed: u8) -> Result<Self, Error> {
        let config = Self {
            speed_ms_per_tick,
            initial_speed,
        };
        if !config.valid_speed(initial_speed) {
            return Err(Error::InvalidSpeed);
        }
        Ok(config)
    }
    pub fn from_defines(defines: &Defines) -> Result<Self, Error> {
        let Some(DefineValue::Array(values)) = defines.get("time.speed_ms_per_tick") else {
            return Err(Error::InvalidTimeDefines);
        };
        let speeds: Vec<_> = values
            .iter()
            .map(|value| match value {
                Number::Integer(n) => u64::try_from(*n).map_err(|_| Error::InvalidTimeDefines),
                _ => Err(Error::InvalidTimeDefines),
            })
            .collect::<Result<_, _>>()?;
        let speeds = speeds.try_into().map_err(|_| Error::InvalidTimeDefines)?;
        let Some(DefineValue::Number(Number::Integer(initial))) = defines.get("time.initial_speed")
        else {
            return Err(Error::InvalidTimeDefines);
        };
        let initial = u8::try_from(*initial).map_err(|_| Error::InvalidTimeDefines)?;
        Self::new(speeds, initial).map_err(|_| Error::InvalidTimeDefines)
    }
    pub fn initial_speed(&self) -> u8 {
        self.initial_speed
    }
    pub(crate) fn valid_speed(&self, speed: u8) -> bool {
        speed != 0 && usize::from(speed) <= self.speed_ms_per_tick.len()
    }
    pub(crate) fn ms_per_tick(&self, speed: u8) -> u64 {
        self.speed_ms_per_tick[usize::from(speed - 1)]
    }
}
