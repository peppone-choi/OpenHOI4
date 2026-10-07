//! Explicit synthetic valid clock-boundary save, never a shipped game default.
use oh_save::SaveContext;
use std::path::Path;
fn ordinal(year: u32, month: u8, day: u8) -> u64 {
    let y = u64::from(year - 1);
    let mut days = y * 365 + y / 4 - y / 100 + y / 400;
    for m in 1..month {
        let month_days = match m {
            2 if year.is_multiple_of(4)
                && (!year.is_multiple_of(100) || year.is_multiple_of(400)) =>
            {
                29
            }
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        days += month_days;
    }
    days + u64::from(day - 1)
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        return Err("output save path".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let c = SaveContext::national(&root, "m1")?;
    let mut dto = c.simulation(7)?.export_save()?;
    dto.state.date = oh_sim::save_state::DateV1 {
        year: u32::MAX,
        month: 12,
        day: 31,
    };
    dto.state.hour = 23;
    dto.state.tick = (ordinal(u32::MAX, 12, 31) - ordinal(2000, 1, 1)) * 24 + 23;
    if let Some(world) = dto.world.as_mut() {
        for state in &mut world.inputs.states {
            state.ledger.tick = dto.state.tick;
        }
    }
    let sim = oh_sim::Simulation::from_save(dto, c.restore_context())?;
    let before = oh_save::repro::report(&sim)?;
    let mut candidate = sim.clone();
    if candidate.step() != Err(oh_sim::Error::ClockOverflow)
        || oh_save::repro::report(&candidate)? != before
    {
        return Err("clock boundary changed state".into());
    }
    std::fs::write(&args[0], oh_save::encode(&sim, &c, 0, vec![])?).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string(&before).map_err(|e| e.to_string())?
    );
    Ok(())
}
