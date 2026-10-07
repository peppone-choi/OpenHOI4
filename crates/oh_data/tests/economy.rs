use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
static SERIAL: AtomicU64 = AtomicU64::new(0);
fn fixture(text: &str) -> PathBuf {
    fn copy(source: &Path, dest: &Path) {
        std::fs::create_dir_all(dest).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let p = entry.unwrap().path();
            let d = dest.join(p.file_name().unwrap());
            if p.is_dir() {
                copy(&p, &d);
            } else {
                std::fs::copy(p, d).unwrap();
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/evidence/WP-14-M2-r2/data")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::SeqCst)
        ));
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &root,
    );
    std::fs::create_dir_all(root.join("common/economy")).unwrap();
    std::fs::write(root.join("common/economy/synthetic.toml"), text).unwrap();
    let scenario = root.join("scenarios/m1/scenario.toml");
    let original = std::fs::read_to_string(&scenario).unwrap();
    std::fs::write(scenario, format!("economy = \"synthetic\"\n{original}")).unwrap();
    root
}
const VALID: &str = include_str!("fixtures/economy/valid.toml");
#[test]
fn req_nat_04_real_loader_accepts_political_conditions_and_validates_full_refs() {
    let root = fixture(VALID);
    let l = oh_data::national::load_scenario(&root, "m1").unwrap();
    assert!(l.economy.is_some());
    let report = oh_data::pack_validation::validate_for_purpose(
        &[root],
        oh_data::pack_validation::ValidationPurpose::Strict,
    );
    assert!(!report.failed(true), "{:?}", report.diagnostics);
}
#[test]
fn req_nat_04_zero_negative_cost_and_unsupported_zero_effect_are_rejected() {
    for cost in ["0", "-1"] {
        let root = fixture(&VALID.replace("cost = \"3\"", &format!("cost = \"{cost}\"")));
        let e = oh_data::national::load_scenario(&root, "m1").unwrap_err();
        assert!(
            e.message.contains("cost") || e.message.contains("negative"),
            "{e}"
        );
    }
}
#[test]
fn req_eco_01_02_04_06_07_nat_02_03_bad_values_references_and_presence_reject() {
    for (before, after) in [
        ("ic_per_level = \"10\"", "ic_per_level = \"NaN\""),
        (
            "allocation = [\"0.25\", \"0.25\", \"0.25\", \"0.25\"]",
            "allocation = [\"0.25\", \"0.25\", \"0.25\", \"0.5\"]",
        ),
        ("daily_cap = \"6\"", "daily_cap = \"-1\""),
        ("reserved = 10", "reserved = -1"),
        ("stability = \"0.5\"", "stability = \"1.1\""),
        ("political_capital = \"0\"", "political_capital = \"101\""),
        ("has_law = \"civil\"", "has_law = \"missing\""),
        (
            "condition = { stability = { gte = \"0.5\" } }",
            "condition = { at_war = false }",
        ),
        ("1 = 4", "1 = 0"),
    ] {
        let root = fixture(&VALID.replace(before, after));
        assert!(
            oh_data::national::load_scenario(&root, "m1").is_err(),
            "accepted {after}"
        );
    }
}
