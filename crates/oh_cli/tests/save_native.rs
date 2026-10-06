use std::{path::Path, process::Command};

#[test]
fn req_sav_02_new_native_process_resumes_real_m1() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = root.join("target/wp11-native-red");
    std::fs::create_dir_all(&out).unwrap();
    let save = out.join(format!("{}.ohsave", std::process::id()));
    let invoke = |arguments: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_oh_cli"))
            .current_dir(&root)
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let continuous = invoke(&[
        "run",
        "--pack",
        "data/packs/testland",
        "--scenario",
        "m1",
        "--days",
        "4",
        "--seed",
        "7",
        "--hash-out",
    ]);
    invoke(&[
        "run",
        "--pack",
        "data/packs/testland",
        "--scenario",
        "m1",
        "--days",
        "2",
        "--seed",
        "7",
        "--save-out",
        save.to_str().unwrap(),
        "--hash-out",
    ]);
    let resumed = invoke(&[
        "resume",
        "--load",
        save.to_str().unwrap(),
        "--pack",
        "data/packs/testland",
        "--days",
        "2",
        "--hash-out",
    ]);
    assert_eq!(continuous, resumed);
}
#[test]
fn req_sav_01_rejects_resume_overrides_and_duplicate_options() {
    for args in [
        vec![
            "resume", "--load", "x", "--pack", "y", "--days", "1", "--seed", "9",
        ],
        vec![
            "resume", "--load", "x", "--pack", "y", "--ticks", "1", "--days", "1",
        ],
        vec!["run", "--save-out", "x", "--save-out", "y"],
    ] {
        let arguments = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(oh_cli::execute_invocation(&arguments).is_err());
    }
}
#[test]
fn req_sav_02_paused_resume_error_is_atomic_and_zero_advance_valid() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut dto = c.simulation(7).unwrap().export_save().unwrap();
    dto.state.paused = true;
    dto.queue.push(oh_sim::save_state::PendingV1 {
        tick: 48,
        nation: 0,
        sequence: 1,
        command: oh_sim::save_state::CommandV1::SetSpeed(5),
    });
    let mut sim = oh_sim::Simulation::from_save(dto, c.restore_context()).unwrap();
    let before = sim.state_hash().unwrap();
    assert!(
        oh_cli::advance(&mut sim, 1)
            .unwrap_err()
            .contains("PausedCannotAdvance")
    );
    assert_eq!(sim.state_hash().unwrap(), before);
    assert_eq!(sim.pending_commands().len(), 1);
    oh_cli::advance(&mut sim, 0).unwrap();
    assert_eq!(sim.state_hash().unwrap(), before);
}
