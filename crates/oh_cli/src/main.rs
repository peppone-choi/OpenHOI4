use std::{env, path::Path, process::ExitCode};
fn execute() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args == ["--help"] || args == ["-h"] {
        println!("{}", oh_cli::USAGE);
        return Ok(());
    }
    let options = oh_cli::RunOptions::parse(&args)?;
    let sim = oh_cli::run(Path::new(oh_cli::M0_PACK_ROOT), &options)?;
    let hash = sim.state_hash().map_err(|err| err.to_string())?;
    if options.hash_out {
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
