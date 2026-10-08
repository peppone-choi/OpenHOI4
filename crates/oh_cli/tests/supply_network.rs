use std::{path::Path, process::Command};
#[test]
fn native_inspect_command_reaches_initial_world_missing_context_without_defaults() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let o = Command::new(env!("CARGO_BIN_EXE_oh_cli"))
        .current_dir(&p)
        .args([
            "supply-network",
            "inspect",
            "--pack",
            "data/packs/testland",
            "--scenario",
            "m1",
            "--nation",
            "1",
            "--config",
            "crates/oh_data/tests/fixtures/supply_network/capital.toml",
        ])
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("MissingContext"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}
use std::{
    fs,
    path::PathBuf,
    process::Output,
    sync::atomic::{AtomicU64, Ordering},
};
const GOOD: &str = include_str!("../../oh_data/tests/fixtures/supply_network/capital.toml");
struct Fixture {
    root: PathBuf,
    pack: PathBuf,
    config: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "oh-wp19-config-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let pack = root.join("pack");
        fn copy(from: &Path, to: &Path) {
            fs::create_dir_all(to).unwrap();
            for e in fs::read_dir(from).unwrap() {
                let e = e.unwrap();
                if e.file_type().unwrap().is_dir() {
                    copy(&e.path(), &to.join(e.file_name()));
                } else {
                    fs::copy(e.path(), to.join(e.file_name())).unwrap();
                }
            }
        }
        copy(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
            &pack,
        );
        let defines = pack.join("scenarios/m1/defines.toml");
        let mut d = fs::read_to_string(&defines).unwrap();
        d.push_str("\n[movement]\nterrain_plains = 1\nterrain_hills = 2\nriver_normal = 1\nriver_small = 1\nriver_large = 1\n");
        fs::write(defines, d).unwrap();
        let config = root.join("network.toml");
        fs::write(&config, GOOD).unwrap();
        Self { root, pack, config }
    }
    fn invoke(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_oh_cli"))
            .args(["supply-network", "inspect", "--pack"])
            .arg(&self.pack)
            .args(["--scenario", "m1", "--nation", "1", "--config"])
            .arg(&self.config)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn value(o: Output) -> serde_json::Value {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    serde_json::from_slice(&o.stdout).unwrap()
}
#[test]
fn native_success_provenance_asymmetry_and_two_fresh_processes() {
    let f = Fixture::new();
    let a = f.invoke();
    let b = f.invoke();
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert_eq!(a.stdout, b.stdout);
    println!(
        "NETWORK_DIAGNOSTIC_FNV64={:016x}",
        oh_core::fnv1a64(&a.stdout)
    );
    let v: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(v["scope"], "initial_world_network");
    assert_eq!(v["nation"], 1);
    assert_eq!(v["capital"], 10);
    assert_eq!(
        v["provenance"]["config_identity_scope"],
        "normalized_diagnostic_only"
    );
    assert_eq!(
        v["provenance"]["rails"],
        "explicit_external_static_metadata_not_construction_state"
    );
    assert_eq!(v["sources"][0]["capacity_qty_bits"], 20i64 << 16);
    assert_eq!(v["sources"][0]["feeder"], serde_json::json!([10]));
    let rail = &v["rails"][0];
    assert_eq!(rail["shared_capacity_qty_bits"], 20i64 << 16);
    assert_ne!(rail["a_to_b_hours_fx_bits"], rail["b_to_a_hours_fx_bits"]);
    let nodes = v["nodes"].as_array().unwrap();
    let p = nodes.iter().find(|p| p["province"] == 20).unwrap();
    assert_eq!(p["owner"], 1);
    assert_eq!(p["controller"], 2);
    let sea = nodes.iter().find(|p| p["province"] == 50).unwrap();
    assert_eq!(sea["land"], false);
    assert!(sea["controller"].is_null());
    for key in [
        "state_hash",
        "daily_supply",
        "demand",
        "delivered",
        "ratio",
        "effects",
        "divisions",
        "tick",
    ] {
        assert!(v.get(key).is_none());
    }
}
#[test]
fn equivalent_config_bytes_same_identity_and_changed_coefficients_change_costs() {
    let f = Fixture::new();
    let original = value(f.invoke());
    fs::write(
        &f.config,
        GOOD.replace("\"4\"", "\"4.0\"")
            .replace("\"20\"", "\"20.00\""),
    )
    .unwrap();
    let equivalent = value(f.invoke());
    assert_eq!(original, equivalent);
    fs::write(&f.config, GOOD.replace("\"1.5\"", "\"1.75\"")).unwrap();
    let changed = value(f.invoke());
    assert_ne!(
        original["provenance"]["config_identity"],
        changed["provenance"]["config_identity"]
    );
    assert_ne!(original["land"], changed["land"]);
}
#[test]
fn native_initial_modifier_needs_exact_level_then_uses_nondefault_factor() {
    let f = Fixture::new();
    let original = value(f.invoke());
    let scenario = f.pack.join("scenarios/m1/scenario.toml");
    let mut text = fs::read_to_string(&scenario).unwrap();
    text.push_str("\n[[state_modifiers.1]]\nsource=\"network-diagnostic\"\ntarget_stat=\"infrastructure\"\noperation=\"add\"\nvalue=\"0.125\"\nexpires=1\n");
    fs::write(scenario, text).unwrap();
    let bad = f.invoke();
    assert_eq!(bad.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&bad.stderr).contains("MissingContext"));
    assert!(bad.stdout.is_empty());
    fs::write(
        &f.config,
        GOOD.replace(
            "{ level = \"1\", factor = \"1.5\" }",
            "{ level = \"1\", factor = \"1.5\" }, { level = \"1.125\", factor = \"0.5\" }",
        ),
    )
    .unwrap();
    let current = value(f.invoke());
    assert_eq!(
        current["states"][0]["current_infrastructure_fx_bits"],
        (1i64 << 32) + (1i64 << 29)
    );
    assert_ne!(original["land"], current["land"]);
    assert_eq!(current["scope"], "initial_world_network");
}
#[test]
fn native_empty_explicit_metadata_remains_network_only() {
    let f = Fixture::new();
    let original = value(f.invoke());
    fs::write(
        &f.config,
        GOOD.replace(
            "[{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }]",
            "[]",
        )
        .replace("[{ a = 10, b = 20, level = 2 }]", "[]"),
    )
    .unwrap();
    let v = value(f.invoke());
    assert_eq!(v["sources"], serde_json::json!([]));
    assert_eq!(v["rails"], serde_json::json!([]));
    assert!(!v["land"].as_array().unwrap().is_empty());
    assert_ne!(
        original["provenance"]["config_identity"],
        v["provenance"]["config_identity"]
    );
    assert!(v.get("ratio").is_none());
}
#[test]
fn native_refs_unknown_and_missing_configs_fail_with_no_output() {
    let f = Fixture::new();
    assert!(f.invoke().status.success());
    for (text, expected) in [
        (
            GOOD.replace("version = 1", "version = 1\nunknown=true"),
            "InvalidConfig",
        ),
        (
            GOOD.replace("province = 10", "province = 999"),
            "InvalidReference",
        ),
        (
            GOOD.replace(
                "kind = \"capital\", province = 10",
                "kind = \"port\", province = 20",
            ),
            "UnsupportedBuildingInstance",
        ),
        (
            GOOD.replace(
                "kind = \"capital\", province = 10",
                "kind = \"hub\", province = 20",
            ),
            "UnsupportedBuildingInstance",
        ),
        (GOOD.replace("\"1.5\"", "\"0\""), "InvalidValue"),
    ] {
        fs::write(&f.config, text).unwrap();
        let o = f.invoke();
        assert_eq!(o.status.code(), Some(1));
        assert!(o.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&o.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
    fs::remove_file(&f.config).unwrap();
    let o = f.invoke();
    assert_eq!(o.status.code(), Some(1));
    assert!(o.stdout.is_empty());
}
#[test]
fn native_options_require_config_and_reject_saved_world_and_noncanonical_ids() {
    let base = [
        "supply-network",
        "inspect",
        "--pack",
        "unused",
        "--scenario",
        "m1",
        "--nation",
        "1",
        "--config",
        "unused.toml",
    ];
    let valid = base.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
    assert!(oh_cli::supply_network::Options::parse(&valid).is_ok());
    let mut cases = Vec::new();
    cases.push(
        base[..8]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>(),
    );
    for extra in [
        ["--load", "save.bin"],
        ["--days", "1"],
        ["--config", "other.toml"],
        ["--nation", "2"],
    ] {
        let mut v = valid.clone();
        v.extend(extra.map(str::to_string));
        cases.push(v);
    }
    for id in ["01", "-1", "65536", "1.0", "+1"] {
        let mut v = valid.clone();
        v[7] = id.into();
        cases.push(v);
    }
    for args in cases {
        assert!(oh_cli::supply_network::Options::parse(&args).is_err());
        let o = Command::new(env!("CARGO_BIN_EXE_oh_cli"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(1));
        assert!(o.stdout.is_empty());
    }
    let mut zero = valid;
    zero[7] = "0".into();
    assert_eq!(
        oh_cli::supply_network::Options::parse(&zero)
            .unwrap()
            .nation,
        oh_core::NationId(0)
    );
}
#[test]
fn native_actual_nation_zero_and_captured_capital_keep_explicit_null_connectivity() {
    let f = Fixture::new();
    let definition = f.pack.join("scenarios/m1/nations/NTH.toml");
    let text = fs::read_to_string(&definition)
        .unwrap()
        .replace("id = 1", "id = 0");
    fs::write(definition, text).unwrap();
    fs::write(
        &f.config,
        GOOD.replace("id = 1\nsources", "id = 0\nsources"),
    )
    .unwrap();
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_oh_cli"))
            .args(["supply-network", "inspect", "--pack"])
            .arg(&f.pack)
            .args(["--scenario", "m1", "--nation", "0", "--config"])
            .arg(&f.config)
            .output()
            .unwrap()
    };
    let original = value(invoke());
    assert_eq!(original["nation"], 0);
    assert_eq!(original["sources"][0]["feeder"], serde_json::json!([10]));
    assert_eq!(original["nodes"][0]["controller"], 0);
    let scenario = f.pack.join("scenarios/m1/scenario.toml");
    let text = fs::read_to_string(&scenario)
        .unwrap()
        .replace("[control_overrides]", "[control_overrides]\n10 = \"STH\"");
    fs::write(scenario, text).unwrap();
    let captured = value(invoke());
    assert_eq!(captured["nodes"][0]["owner"], 0);
    assert_eq!(captured["nodes"][0]["controller"], 2);
    assert!(captured["sources"][0]["feeder"].is_null());
    assert!(captured.get("ratio").is_none());
}
