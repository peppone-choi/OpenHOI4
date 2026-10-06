use std::process::{Command, Output};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oh_cli"))
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .args(args)
        .output()
        .unwrap()
}
fn hash(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout.clone()).unwrap();
    let hash = text.trim();
    assert_eq!(hash.len(), 16, "{text:?}");
    assert!(
        hash.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    hash.to_owned()
}
#[test]
fn req_gen_06_ac_m0_02_exact_ticks_and_repeated_hash() {
    let args = [
        "run",
        "--scenario",
        "testland",
        "--ticks",
        "1000",
        "--seed",
        "1",
        "--hash-out",
    ];
    assert_eq!(hash(&run(&args)), hash(&run(&args)));
    let output = run(&[
        "run",
        "--scenario",
        "testland",
        "--ticks",
        "1000",
        "--seed",
        "1",
    ]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("tick=1000"), "{text}");
    assert!(text.contains("date=2000-02-11 hour=16"), "{text}");
}
#[test]
fn req_gen_06_days_path_is_exact_and_zero_ticks_stays_initial() {
    let days = hash(&run(&[
        "run",
        "--scenario",
        "testland",
        "--days",
        "365",
        "--seed",
        "1",
        "--hash-out",
    ]));
    let ticks = hash(&run(&[
        "run",
        "--scenario",
        "testland",
        "--ticks",
        "8760",
        "--seed",
        "1",
        "--hash-out",
    ]));
    assert_eq!(days, ticks);
    assert_ne!(
        ticks,
        hash(&run(&[
            "run",
            "--scenario",
            "testland",
            "--ticks",
            "0",
            "--seed",
            "1",
            "--hash-out"
        ]))
    );
}
#[test]
fn cli_rejects_ambiguous_missing_invalid_and_unimplemented_inputs() {
    for args in [
        vec![],
        vec!["run"],
        vec!["validate"],
        vec!["ai-bench"],
        vec!["repro"],
        vec![
            "run",
            "--scenario",
            "../testland",
            "--ticks",
            "1",
            "--seed",
            "1",
        ],
        vec![
            "run",
            "--scenario",
            "missing",
            "--ticks",
            "1",
            "--seed",
            "1",
        ],
        vec![
            "run",
            "--scenario",
            "testland",
            "--ticks",
            "1",
            "--days",
            "1",
            "--seed",
            "1",
        ],
        vec![
            "run",
            "--scenario",
            "testland",
            "--ticks",
            "-1",
            "--seed",
            "1",
        ],
        vec![
            "run",
            "--scenario",
            "testland",
            "--days",
            "18446744073709551615",
            "--seed",
            "1",
        ],
        vec![
            "run",
            "--scenario",
            "testland",
            "--ticks",
            "1",
            "--seed",
            "1",
            "--seed",
            "2",
        ],
    ] {
        let output = run(&args);
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn library_loads_data_epoch_and_rejects_bad_or_nonempty_scenario() {
    use oh_cli::{RunOptions, run};
    use std::fs;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wp04-test-fixtures")
        .join(format!("run-{}", std::process::id()));
    let pack = root.join("sample");
    let scenarios = pack.join("scenarios/sample");
    fs::create_dir_all(&scenarios).unwrap();
    fs::write(
        pack.join("manifest.toml"),
        "id='sample'\nname_key='sample'\nversion='0.1.0'\nengine='>=0.1'\n",
    )
    .unwrap();
    fs::write(
        pack.join("defines.toml"),
        "[time]\nspeed_ms_per_tick=[9,8,7,6,0]\ninitial_speed=2\n",
    )
    .unwrap();
    let options = RunOptions {
        scenario: "sample".into(),
        ticks: 24,
        seed: 42,
        hash_out: true,
    };
    let path = scenarios.join("scenario.toml");
    fs::write(&path, "start_date='1800-12-31'\n").unwrap();
    let sim = run(&root, &options).unwrap();
    assert_eq!(
        sim.snapshot().date(),
        oh_sim::Date::new(1801, 1, 1).unwrap()
    );
    assert_eq!(sim.snapshot().tick(), 24);
    assert_eq!(sim.snapshot().seed(), 42);
    assert_eq!(sim.ms_per_tick(), 8);
    for source in [
        "start_date='1900-02-29'",
        "start_date='not-a-date'",
        "start_date='2000-01-01'\nnations=['X']",
        "start_date=12",
        "",
        "start_date='0-01-01'",
    ] {
        fs::write(&path, source).unwrap();
        assert!(run(&root, &options).is_err(), "{source}");
    }
    fs::write(&path, "start_date='2000-01-01'").unwrap();
    fs::write(pack.join("defines.toml"), "").unwrap();
    assert!(run(&root, &options).is_err());
    // Remove only exact development files created by this test, no recursive delete.
    fs::remove_file(&path).unwrap();
    fs::remove_file(pack.join("defines.toml")).unwrap();
    fs::remove_file(pack.join("manifest.toml")).unwrap();
    fs::remove_dir(&scenarios).unwrap();
    fs::remove_dir(pack.join("scenarios")).unwrap();
    fs::remove_dir(&pack).unwrap();
    fs::remove_dir(&root).unwrap();
}

#[test]
fn m0_load_boundary_and_help_identify_unique_self_contained_pack() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let loaded = oh_cli::load_scenario(&root.join(oh_cli::M0_PACK_ROOT), "testland").unwrap();
    assert_eq!(loaded.pack.manifest.id, "m0_testland");
    assert!(loaded.pack.manifest.depends.is_empty());
    assert_eq!(loaded.scenario_id, "testland");
    let a = loaded.simulation(1).unwrap();
    assert_eq!(a.snapshot().tick(), 0);
    assert_eq!(a.ms_per_tick(), 500);
    let help = run(&["--help"]);
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    for expected in [
        oh_cli::M0_PACK_ROOT,
        "m0_testland",
        "manifest.toml",
        "defines.toml",
        "scenarios/testland/scenario.toml",
    ] {
        assert!(help.contains(expected), "{help}");
    }
}
