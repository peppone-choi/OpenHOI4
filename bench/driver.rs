//! The identical external driver is compiled against each source's actual library.
use std::{path::Path, time::Instant};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("pack scenario seed steps".into());
    }
    let seed = args[2].parse::<u64>().map_err(|e| e.to_string())?;
    let steps = args[3].parse::<u64>().map_err(|e| e.to_string())?;
    let options = oh_cli::RunOptions {
        scenario: args[1].clone(),
        ticks: 0,
        seed,
        hash_out: false,
    };
    let mut sim = oh_cli::run_national(Path::new(&args[0]), &options)?;
    let started = Instant::now();
    for _ in 0..steps {
        let result = sim.step().map_err(|e| e.to_string())?;
        if !result.advanced {
            return Err("benchmark workload did not advance".into());
        }
    }
    let elapsed_ns = started.elapsed().as_nanos();
    if elapsed_ns == 0 || sim.snapshot().tick() != steps || sim.is_ended() {
        return Err("benchmark incomplete/zero elapsed".into());
    }
    let dto = if sim.trigger_state().is_some() {
        serde_json::to_value(sim.export_save_v4()?)
    } else {
        serde_json::to_value(sim.export_save()?)
    }
    .map_err(|e| e.to_string())?;
    let bytes = oh_core::canonical_bytes(&sim).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({"elapsed_ns":elapsed_ns,"steps":steps,"scenario":args[1],"seed":seed,"tick":sim.snapshot().tick(),"ended":sim.is_ended(),"dto":dto,"canonical_hex":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>(),"hash":format!("{:016x}",sim.state_hash().map_err(|e|e.to_string())?)})
    );
    Ok(())
}
