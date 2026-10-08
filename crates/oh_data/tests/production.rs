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

#[test]
fn req_eco_03_allowed_model_duplicates_rejected_with_canonical_valid_control() {
    let schema = serde_json::to_value(oh_data::production::schema()).unwrap();
    assert_eq!(
        schema["$defs"]["NationInput"]["properties"]["allowed_models"]["uniqueItems"],
        true
    );
    let root = fixture::pack();
    let original = oh_data::national::load_scenario(&root, "m1")
        .unwrap()
        .production
        .unwrap();
    let path = root.join("common/production/synthetic.toml");
    let source = std::fs::read_to_string(&path).unwrap();
    let from = "allowed_models = [\"test_model_1\", \"test_model_2\"]";
    assert!(source.contains(from));
    // Unique unsorted source is valid and must retain the same sorted identity.
    std::fs::write(
        &path,
        source.replace(
            from,
            "allowed_models = [\"test_model_2\", \"test_model_1\"]",
        ),
    )
    .unwrap();
    let reordered = oh_data::national::load_scenario(&root, "m1")
        .unwrap()
        .production
        .unwrap();
    assert_eq!(original, reordered);
    assert_eq!(original.identity().unwrap(), reordered.identity().unwrap());
    let valid = oh_data::pack_validation::validate_for_purpose(
        std::slice::from_ref(&root),
        oh_data::pack_validation::ValidationPurpose::Strict,
    );
    assert!(!valid.failed(true), "{:?}", valid.diagnostics);
    for duplicate in [
        "allowed_models = [\"test_model_1\", \"test_model_1\", \"test_model_2\"]",
        "allowed_models = [\"test_model_1\", \"test_model_2\", \"test_model_1\"]",
    ] {
        std::fs::write(&path, source.replace(from, duplicate)).unwrap();
        assert!(
            oh_data::national::load_scenario(&root, "m1").is_err(),
            "{duplicate}"
        );
        let invalid = oh_data::pack_validation::validate_for_purpose(
            std::slice::from_ref(&root),
            oh_data::pack_validation::ValidationPurpose::Strict,
        );
        assert!(
            invalid.failed(true),
            "{duplicate}: {:?}",
            invalid.diagnostics
        );
    }
    let valid = serde_json::json!({"allowed_models":["test_model_2","test_model_1"],"stock":{"test_model_1":0,"test_model_2":0}});
    let parsed: oh_data::production::NationInput = serde_json::from_value(valid.clone()).unwrap();
    assert_eq!(
        parsed
            .allowed_models
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["test_model_1", "test_model_2"]
    );
    let mut duplicate = valid;
    duplicate["allowed_models"] =
        serde_json::json!(["test_model_1", "test_model_2", "test_model_1"]);
    assert!(serde_json::from_value::<oh_data::production::NationInput>(duplicate).is_err());
}
