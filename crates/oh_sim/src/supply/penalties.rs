use super::*;
/// Scalar effect plan only. A real owner must apply effects to its actual division
/// strength/org/attack and future-leg state; this function does not imply that adapter exists.
pub fn shortage_effects(
    ratio: Fx,
    demand: Qty,
    previous_days: u32,
    defines: PenaltyDefines,
) -> Result<Effects, Error> {
    for value in [
        ratio,
        defines.organization_floor,
        defines.attack_floor,
        defines.speed_floor,
        defines.starvation_threshold,
        defines.attrition_at_zero,
    ] {
        if !(Fx::ZERO..=Fx::ONE).contains(&value) {
            return Err(Error::InvalidValue);
        }
    }
    if demand < Qty::ZERO || defines.speed_floor == Fx::ZERO {
        return Err(Error::InvalidValue);
    }
    let r = if demand == Qty::ZERO { Fx::ONE } else { ratio };
    let multiplier = |floor: Fx| {
        floor
            .checked_add((Fx::ONE - floor).checked_mul(r).ok_or(Error::Overflow)?)
            .ok_or(Error::Overflow)
    };
    let speed = multiplier(defines.speed_floor)?;
    let movement_time = Fx::ONE.checked_div(speed).ok_or(Error::Overflow)?;
    if movement_time <= Fx::ZERO {
        return Err(Error::InvalidValue);
    }
    let starvation_days = if demand > Qty::ZERO && r < defines.starvation_threshold {
        previous_days.checked_add(1).ok_or(Error::Overflow)?
    } else {
        0
    };
    let attrition_rate = if starvation_days > defines.grace_days {
        defines
            .attrition_at_zero
            .checked_mul(Fx::ONE - r)
            .ok_or(Error::Overflow)?
    } else {
        Fx::ZERO
    };
    Ok(Effects {
        organization: multiplier(defines.organization_floor)?,
        attack: multiplier(defines.attack_floor)?,
        speed,
        movement_time,
        starvation_days,
        attrition_rate,
    })
}
