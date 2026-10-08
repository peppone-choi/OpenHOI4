#[path = "support/production.rs"]
mod fixture;
#[test]
fn req_eco_03_strict_loader_and_schema_ref_tuning_controls() {
    let root = fixture::pack();
    let l = oh_data::national::load_scenario(&root, "m1").unwrap();
    assert_eq!(l.production.as_ref().unwrap().models.len(), 2);
    let r = oh_data::pack_validation::validate_for_purpose(
        &[root],
        oh_data::pack_validation::ValidationPurpose::Strict,
    );
    assert!(!r.failed(true), "{:?}", r.diagnostics);
}
#[test]
fn req_eco_03_rejects_missing_tuning_zero_cost_unknown_resource_and_null_presence() {
    for (file, from, to) in [
        (
            "defines.toml",
            "initial_efficiency = 0.25",
            "initial_efficiency = 0.0",
        ),
        (
            "common/production/synthetic.toml",
            "unit_cost = \"1\"",
            "unit_cost = \"0\"",
        ),
        (
            "common/production/synthetic.toml",
            "steel = \"1\"",
            "unknown = \"1\"",
        ),
        (
            "common/production/synthetic.toml",
            "test_model_1 = 0",
            "test_model_1 = -1",
        ),
    ] {
        let root = fixture::pack();
        let p = root.join(file);
        let s = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, s.replace(from, to)).unwrap();
        assert!(
            oh_data::national::load_scenario(&root, "m1").is_err(),
            "{file}:{to}"
        );
    }
    let root = fixture::pack();
    let l = oh_data::national::load_scenario(&root, "m1").unwrap();
    let mut raw = serde_json::to_value(&l.scenario).unwrap();
    raw["production"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<oh_data::national::Scenario>(raw).is_err());
    let root = fixture::pack();
    std::fs::write(root.join("common/production/orphan.toml"), "").unwrap();
    assert!(
        oh_data::pack_validation::validate_for_purpose(
            &[root],
            oh_data::pack_validation::ValidationPurpose::Strict
        )
        .failed(true)
    );
}
