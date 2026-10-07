//! Host elapsed time is outside Simulation; load/report are outside measurement.
use std::{collections::BTreeMap, path::Path, time::Instant};
pub fn execute(args: &[String]) -> Result<(), String> {
    let mut values = BTreeMap::new();
    let mut i = 1;
    while i < args.len() {
        let key = args[i].as_str();
        if !["--pack", "--scenario", "--seed", "--steps"].contains(&key) {
            return Err(format!("unknown bench argument {key}"));
        }
        let value = args
            .get(i + 1)
            .filter(|v| !v.starts_with("--"))
            .ok_or("missing bench value")?;
        if values.insert(key, value.as_str()).is_some() {
            return Err("duplicate bench argument".into());
        }
        i += 2;
    }
    let get = |k: &str| {
        values
            .get(k)
            .copied()
            .ok_or_else(|| format!("missing bench {k}"))
    };
    let seed = get("--seed")?
        .parse::<u64>()
        .map_err(|_| "invalid bench seed")?;
    let steps = get("--steps")?
        .parse::<u64>()
        .map_err(|_| "invalid bench steps")?;
    if steps == 0 {
        return Err("bench workload must be positive".into());
    }
    let options = crate::RunOptions {
        scenario: get("--scenario")?.into(),
        ticks: 0,
        seed,
        hash_out: false,
    };
    let mut sim = crate::run_national(Path::new(get("--pack")?), &options)?;
    let started = Instant::now();
    for _ in 0..steps {
        if !sim.step().map_err(|e| e.to_string())?.advanced {
            return Err("bench incomplete workload".into());
        }
    }
    let elapsed_ns = started.elapsed().as_nanos();
    if elapsed_ns == 0 || sim.snapshot().tick() != steps || sim.is_ended() {
        return Err("bench zero elapsed/incomplete workload".into());
    }
    println!(
        "{}",
        serde_json::json!({"elapsed_ns":elapsed_ns,"steps":steps,"seed":seed,"scenario":options.scenario,"state":oh_save::repro::report(&sim)?})
    );
    Ok(())
}
