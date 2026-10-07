use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
fn fixture() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let base = std::env::var_os("OH_WP24_EVIDENCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/evidence/WP-24")
        });
    let p = base.join("fixtures").join(format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&p).unwrap();
    p
}
fn pack(parent: &Path, id: &str, extra: &str) -> PathBuf {
    let p = parent.join(id);
    fs::create_dir_all(&p).unwrap();
    fs::write(p.join("manifest.toml"), format!("id = \"{id}\"\nname_key = \"pack-name\"\nversion = \"0.1.0\"\nengine = \">=0.1, <0.2\"\n{extra}\n")).unwrap();
    fs::write(p.join("defines.toml"), "").unwrap();
    for lang in ["ko", "en"] {
        fs::create_dir_all(p.join("localisation").join(lang)).unwrap();
        fs::write(
            p.join("localisation").join(lang).join("pack.ftl"),
            "pack-name = Pack\n",
        )
        .unwrap();
    }
    p
}
fn validate(paths: &[&Path], deny: bool) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_oh_cli"));
    c.arg("validate");
    if deny {
        c.arg("--deny-warnings");
    }
    c.args(paths);
    c.output().unwrap()
}
fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into()
}
fn copy(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for e in fs::read_dir(source).unwrap() {
        let p = e.unwrap().path();
        let t = target.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &t)
        } else {
            fs::copy(p, t).unwrap();
        }
    }
}
#[test]
fn req_mod_01_diamond_order_and_identity_are_input_independent() {
    let p = fixture();
    let base = pack(&p, "base", "");
    let a = pack(
        &p,
        "a",
        "depends = [{ id = \"base\", version = \">=0.1\" }]",
    );
    let b = pack(
        &p,
        "b",
        "depends = [{ id = \"base\", version = \"=0.1.0\" }]",
    );
    let top = pack(
        &p,
        "top",
        "depends = [{ id = \"b\", version = \"*\" }, { id = \"a\", version = \"*\" }]",
    );
    let x = validate(&[&top, &b, &base, &a], true);
    let y = validate(&[&a, &base, &top, &b], true);
    assert!(x.status.success(), "{}", stderr(&x));
    assert_eq!(stdout(&x), stdout(&y));
    let ids: Vec<_> = stdout(&x)
        .lines()
        .filter_map(|l| {
            l.strip_prefix("pack=")
                .map(|s| s.split_whitespace().next().unwrap().to_owned())
        })
        .collect();
    assert_eq!(ids, ["base", "a", "b", "top"]);
}
#[test]
fn req_mod_02_shipped_pack_loads_real_map_and_national_data() {
    let p = fixture();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &p,
    );
    // Local fixture supplies the pre-existing missing manifest name without changing shipped content.
    for lang in ["ko", "en"] {
        fs::write(
            p.join("localisation").join(lang).join("pack.ftl"),
            "testland_name = Testland\n",
        )
        .unwrap();
    }
    let o = validate(&[&p], true);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stdout(&o).contains("maps=1 scenarios=1"), "{}", stdout(&o));
    let n = p.join("scenarios/m1/nations/NTH.toml");
    let text = fs::read_to_string(&n).unwrap();
    fs::write(&n, text.replace("capital = 10", "capital = 65535")).unwrap();
    let o = validate(&[&p], false);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("capital"));
    assert!(stderr(&o).contains("NTH.toml:"));
}
#[test]
fn req_mod_04_warnings_exit_policy_and_source_context() {
    let p = fixture();
    let a = pack(&p, "a", "");
    for lang in ["ko", "en"] {
        fs::write(
            a.join("localisation").join(lang).join("unused.ftl"),
            "unused-key = Unused\n",
        )
        .unwrap();
    }
    let o = validate(&[&a], false);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stderr(&o).contains("unused.ftl:1:1"));
    assert!(stderr(&o).contains("warning"));
    let o = validate(&[&a], true);
    assert_eq!(o.status.code(), Some(1));
}
#[test]
fn req_loc_03_missing_translation_is_error_and_real_fluent_is_accepted() {
    let p = fixture();
    let a = pack(&p, "a", "");
    for lang in ["ko", "en"] {
        fs::write(a.join("localisation").join(lang).join("pack.ftl"),"-brand = OpenHOI4\npack-name = { -brand } { $count ->\n    [one] One\n   *[other] Many\n    }\n    .title = Title\n").unwrap();
    }
    let o = validate(&[&a], true);
    assert!(o.status.success(), "{}", stderr(&o));
    fs::write(a.join("localisation/en/pack.ftl"), "other = Missing\n").unwrap();
    let o = validate(&[&a], false);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("pack-name"));
    assert!(stderr(&o).contains("manifest.toml:2:"));
}

#[test]
fn req_mod_04_run_rejects_invalid_pack_before_simulation_or_save_creation() {
    let p = fixture();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &p,
    );
    let manifest = p.join("manifest.toml");
    let text = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        text.replace(
            "depends = []",
            "depends = [{ id = \"missing\", version = \"*\" }]",
        ),
    )
    .unwrap();
    for save in [false, true] {
        let destination = p.parent().unwrap().join(format!(
            "bad-{}.ohsave",
            p.file_name().unwrap().to_str().unwrap()
        ));
        let mut c = Command::new(env!("CARGO_BIN_EXE_oh_cli"));
        c.args([
            "run",
            "--pack",
            p.to_str().unwrap(),
            "--scenario",
            "m1",
            "--ticks",
            "0",
            "--seed",
            "1",
            "--hash-out",
        ]);
        if save {
            c.arg("--save-out").arg(&destination);
        }
        let out = c.output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
        assert!(stderr(&out).contains("missing dependency"));
        assert!(!destination.exists());
    }
    fs::write(&manifest, text).unwrap();
    fs::write(p.join("localisation/en/pack.ftl"), "other = Missing name\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_oh_cli"))
        .args([
            "run",
            "--pack",
            p.to_str().unwrap(),
            "--scenario",
            "m1",
            "--ticks",
            "0",
            "--seed",
            "1",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("testland_name"));
}

#[test]
fn p06_legacy_overlay_reaches_actual_cli_time_config_without_defaults() {
    let parent = fixture();
    let p = pack(&parent, "empty", "");
    fs::write(
        p.join("defines.toml"),
        "[time]\nspeed_ms_per_tick=[500,200,80,25,0]\ninitial_speed=1\n",
    )
    .unwrap();
    fs::create_dir_all(p.join("scenarios/empty")).unwrap();
    fs::write(
        p.join("scenarios/empty/scenario.toml"),
        "start_date='2000-01-01'\n",
    )
    .unwrap();
    let overlay = p.join("scenarios/empty/defines.toml");
    fs::write(
        &overlay,
        "[time]\nspeed_ms_per_tick=[100,90,80,70,60]\ninitial_speed=5\n",
    )
    .unwrap();
    let loaded = oh_cli::load_scenario(&parent, "empty").unwrap();
    let sim = loaded.simulation(1).unwrap();
    assert_eq!(sim.snapshot().speed(), 5);
    assert_eq!(sim.ms_per_tick(), 60);
    fs::remove_file(overlay).unwrap();
    let sim = oh_cli::load_scenario(&parent, "empty")
        .unwrap()
        .simulation(1)
        .unwrap();
    assert_eq!(sim.snapshot().speed(), 1);
    assert_eq!(sim.ms_per_tick(), 500);
}

#[test]
fn p06_cli_restore_force_rejects_invalid_active_fluent_before_decoding() {
    let root = fixture();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &root,
    );
    let context = oh_save::SaveContext::national(&root, "m1").unwrap();
    let bytes = oh_save::encode(&context.simulation(1).unwrap(), &context, 0, vec![]).unwrap();
    let file = root.join("state.ohsave");
    fs::write(&file, &bytes).unwrap();
    fs::write(root.join("localisation/en/pack.ftl"), "broken = {\n").unwrap();
    for force in [true, false] {
        let mut c = Command::new(env!("CARGO_BIN_EXE_oh_cli"));
        c.args([
            "resume",
            "--load",
            file.to_str().unwrap(),
            "--pack",
            root.to_str().unwrap(),
            "--ticks",
            "0",
        ]);
        if force {
            c.arg("--force");
        }
        let o = c.output().unwrap();
        assert_eq!(o.status.code(), Some(1), "{}", stdout(&o));
        assert!(stderr(&o).contains("invalid Fluent"), "{}", stderr(&o));
        assert_eq!(fs::read(&file).unwrap(), bytes);
    }
}
