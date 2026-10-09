use std::{env, process::ExitCode};
fn execute() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args == ["--help"] || args == ["-h"] {
        println!("{}", oh_cli::USAGE);
        return Ok(());
    }
    if args.first().is_some_and(|s| s == "validate") {
        oh_cli::validate::execute(&args)?;
        return Ok(());
    }
    if args.first().is_some_and(|s| s == "repro") {
        return oh_cli::repro::execute(&args);
    }
    if args.first().is_some_and(|s| s == "bench") {
        return oh_cli::bench::execute(&args);
    }
    if args.first().is_some_and(|s| s == "supply-network") {
        return oh_cli::supply_network::execute(&args);
    }
    if args.first().is_some_and(|s| s == "military-template") {
        return oh_cli::military_templates::execute(&args);
    }
    let (sim, hash_out) = oh_cli::execute_invocation(&args)?;
    let hash = sim.state_hash().map_err(|err| err.to_string())?;
    if hash_out {
        println!("{hash:016x}");
    } else {
        let state = sim.snapshot();
        println!(
            "scenario={} tick={} date={} hour={} seed={} hash={hash:016x}",
            state.scenario(),
            state.tick(),
            state.date(),
            state.hour(),
            state.seed()
        );
    }
    Ok(())
}
fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("oh_cli: {err}");
            ExitCode::FAILURE
        }
    }
}
