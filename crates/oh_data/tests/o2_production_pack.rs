#[path = "support/o2_production_pack.rs"]
mod fixture;
use oh_data::pack_validation::{ValidationPurpose, validate_for_purpose};

#[test]
fn six_nation_self_contained_pack_has_explicit_production_and_bilingual_strict_validation() {
    let pack = fixture::CopyPack::new();
    let loaded = oh_data::national::load_scenario(pack.root(), fixture::SCENARIO).unwrap();
    assert_eq!(loaded.nations.len(), 6);
    assert_eq!(loaded.map.provinces.len(), 120);
    assert_eq!(loaded.map.states.len(), 12);
    let production = loaded.production.as_ref().unwrap();
    assert_eq!(production.nations.len(), 6);
    assert_eq!(production.models.len(), 2);
    assert!(production.tuning.is_some());
    for input in production.nations.values() {
        assert!(!input.allowed_models.is_empty());
        assert_eq!(
            input.allowed_models.iter().collect::<Vec<_>>(),
            input.stock.keys().collect::<Vec<_>>()
        );
        assert!(input.stock.values().all(|stock| *stock == 0));
    }
    let report = validate_for_purpose(&[pack.root().to_path_buf()], ValidationPurpose::Strict);
    assert!(!report.failed(true), "{:?}", report.diagnostics);
}

#[test]
fn dedicated_pack_rejects_unknown_fields_duplicate_allowlist_unknown_resource_and_missing_tuning() {
    for (file, from, to) in [
        (
            "common/production/initial.toml",
            "unit_cost = \"1\"",
            "unit_cost = \"1\"\nunknown = 1",
        ),
        (
            "common/production/initial.toml",
            "allowed_models = [\"m2_equipment_1\"]",
            "allowed_models = [\"m2_equipment_1\", \"m2_equipment_1\"]",
        ),
        (
            "common/production/initial.toml",
            "steel = \"1\"",
            "missing_resource = \"1\"",
        ),
        ("defines.toml", "daily_efficiency_growth = 0.0078125", ""),
    ] {
        let pack = fixture::CopyPack::new();
        let path = pack.root().join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(source.contains(from));
        std::fs::write(path, source.replace(from, to)).unwrap();
        assert!(
            oh_data::national::load_scenario(pack.root(), fixture::SCENARIO).is_err(),
            "{file}:{to}"
        );
        assert!(
            validate_for_purpose(&[pack.root().to_path_buf()], ValidationPurpose::Strict)
                .failed(true)
        );
    }
}

#[test]
fn missing_localisation_and_unused_producer_are_rejected_with_valid_controls() {
    for mutation in ["localisation", "unused"] {
        let pack = fixture::CopyPack::new();
        let before = validate_for_purpose(&[pack.root().to_path_buf()], ValidationPurpose::Strict);
        assert!(!before.failed(true), "{:?}", before.diagnostics);
        if mutation == "localisation" {
            let path = pack.root().join("localisation/en/production.ftl");
            let source = std::fs::read_to_string(&path).unwrap();
            let reduced = source
                .lines()
                .filter(|line| !line.starts_with("o2-equipment-2 ="))
                .collect::<Vec<_>>()
                .join("\n");
            assert_ne!(source.trim(), reduced.trim());
            std::fs::write(path, reduced).unwrap();
        } else {
            std::fs::write(pack.root().join("common/production/unused.toml"), "").unwrap();
        }
        assert!(
            validate_for_purpose(&[pack.root().to_path_buf()], ValidationPurpose::Strict)
                .failed(true)
        );
    }
    let schema = serde_json::to_value(oh_data::production::schema()).unwrap();
    assert_eq!(
        schema["$defs"]["NationInput"]["properties"]["allowed_models"]["uniqueItems"],
        true
    );
}
