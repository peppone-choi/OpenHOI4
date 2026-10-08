use std::process::Command;
#[test]
fn native_command_exposes_specific_grammar() {
    let o = Command::new(env!("CARGO_BIN_EXE_oh_cli"))
        .args(["military-template", "inspect"])
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("--definitions"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}
use std::path::PathBuf;
#[path = "../../oh_data/tests/support/production.rs"]
mod production;
const GOOD: &str = include_str!("../../oh_data/tests/fixtures/military_templates/normal.toml");
struct Fixture {
    pack: PathBuf,
    definitions: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let pack = production::pack();
        let definitions = pack.parent().unwrap().join("normal.toml");
        std::fs::write(&definitions, GOOD).unwrap();
        Self { pack, definitions }
    }
    fn invoke(&self, nation: &str) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_oh_cli"))
            .args(["military-template", "inspect", "--pack"])
            .arg(&self.pack)
            .args(["--scenario", "m1", "--nation", nation, "--definitions"])
            .arg(&self.definitions)
            .args(["--template", "example"])
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.pack.parent().unwrap()).unwrap();
    }
}
#[test]
fn real_native_initial_production_binding_and_two_fresh_processes() {
    let f = Fixture::new();
    let a = f.invoke("1");
    let b = f.invoke("1");
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert!(b.status.success());
    assert_eq!(a.stdout, b.stdout);
    let v: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(v["scope"], "normal_template_stats");
    assert_eq!(v["manpower"], 1100);
    assert_eq!(v["equipment_requirements"]["test_model_1"], 110);
    assert_eq!(
        v["stats"]["organization_qty_bits"],
        (30i64 * 65536 * 1000 + 60i64 * 65536 * 100) / 1100
    );
    assert_eq!(
        v["provenance"]["stats"],
        "component_declared_resolved_normal_inputs"
    );
    assert_eq!(v["provenance"]["game_editor_legality"], "not_evaluated");
    assert!(v.get("division").is_none());
    assert!(v.get("demand").is_none());
    assert!(v.get("stock").is_none());
    std::fs::write(&f.definitions, GOOD.replace("\"25\"", "\"25.0\"")).unwrap();
    assert_eq!(a.stdout, f.invoke("1").stdout);
}
#[test]
fn actual_model_unknown_wrong_family_disallowed_and_no_production_controls() {
    let f = Fixture::new();
    for (text, expected) in [
        (
            GOOD.replace("test_model_1", "missing_model"),
            "production model",
        ),
        (GOOD.replace("test_family", "wrong_family"), "family"),
        (
            GOOD.replace("combat = [\"line\"]", "combat = []")
                .replace("support = [\"support\"]", "support = []"),
            "UndefinedArithmetic",
        ),
    ] {
        std::fs::write(&f.definitions, text).unwrap();
        let o = f.invoke("1");
        assert!(!o.status.success());
        assert!(
            String::from_utf8_lossy(&o.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
    std::fs::write(&f.definitions, GOOD).unwrap();
    assert!(f.invoke("2").status.success());
    assert!(!f.invoke("01").status.success());
    assert!(!f.invoke("3").status.success());
    let production = f.pack.join("common/production/synthetic.toml");
    let original = std::fs::read_to_string(&production).unwrap();
    let disallowed = original
        .replacen(
            "allowed_models = [\"test_model_1\", \"test_model_2\"]",
            "allowed_models = [\"test_model_2\"]",
            1,
        )
        .replacen(
            "stock = { test_model_1 = 0, test_model_2 = 0 }",
            "stock = { test_model_2 = 0 }",
            1,
        );
    std::fs::write(&production, disallowed).unwrap();
    let o = f.invoke("1");
    assert!(!o.status.success());
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("DisallowedModel"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(f.invoke("2").status.success());
    std::fs::write(&production, original).unwrap();
    let scenario = f.pack.join("scenarios/m1/scenario.toml");
    let s = std::fs::read_to_string(&scenario).unwrap();
    std::fs::write(&scenario, s.replace("production = \"synthetic\"\n", "")).unwrap();
    std::fs::remove_file(&production).unwrap(); // own temp pack: no unregistered production input
    let o = f.invoke("1");
    assert!(!o.status.success());
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("MissingContext"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}
#[test]
fn cli_grammar_never_steps_loads_or_defaults() {
    let good = [
        "military-template",
        "inspect",
        "--pack",
        "pack",
        "--scenario",
        "m1",
        "--nation",
        "0",
        "--definitions",
        "defs.toml",
        "--template",
        "example",
    ]
    .map(str::to_owned)
    .to_vec();
    assert!(oh_cli::military_templates::Options::parse(&good).is_ok());
    for flag in ["--load", "--days", "--ticks", "--save-out", "--force"] {
        let mut args = good.clone();
        args.extend([flag.into(), "1".into()]);
        assert!(oh_cli::military_templates::Options::parse(&args).is_err());
    }
    for index in [2, 4, 6, 8, 10] {
        let mut args = good.clone();
        args.drain(index..index + 2);
        assert!(oh_cli::military_templates::Options::parse(&args).is_err());
        let mut args = good.clone();
        args.extend([good[index].clone(), good[index + 1].clone()]);
        assert!(oh_cli::military_templates::Options::parse(&args).is_err());
    }
}
