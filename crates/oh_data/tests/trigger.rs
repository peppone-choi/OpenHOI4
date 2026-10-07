use oh_data::trigger::{Condition, Effect};

#[test]
fn req_agd_05_single_key_argument_and_depth_validation() {
    for source in [
        r#"{}"#,
        r#"{"all":[]}"#,
        r#"{"date_gte":null}"#,
        r#"{"nation_is":"NTH","at_war":false}"#,
        r#"{"stability":{"gte":"0.5","lte":"1"}}"#,
        r#"{"unknown":true}"#,
        r#"{"controls_province":65536}"#,
    ] {
        assert!(
            oh_data::trigger::parse_condition(source).is_err(),
            "{source}"
        );
    }
    let mut source = r#"{"date_gte":"2000-01-01"}"#.to_owned();
    for _ in 1..16 {
        source = format!("{{\"not\":{source}}}");
    }
    assert!(oh_data::trigger::parse_condition(&source).is_ok());
    source = format!("{{\"not\":{source}}}");
    assert!(oh_data::trigger::parse_condition(&source).is_err());
    let _: Condition =
        oh_data::trigger::parse_condition(r#"{"any":[{"at_war":false},{"nation_is":"NTH"}]}"#)
            .unwrap();
    let _: Vec<Effect> =
        oh_data::trigger::parse_effects(r#"[{"set_flag":"x"},{"end_scenario":"test_end"}]"#)
            .unwrap();
    assert!(oh_data::trigger::parse_effects(r#"[{"set_flag":"x","clear_flag":"x"}]"#).is_err());
}
fn loaded() -> oh_data::national::LoadedNational {
    oh_data::national::load_scenario(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap()
}
#[test]
fn req_agd_05_registry_every_initial_argument_schema_and_malformed() {
    for item in oh_data::trigger::registry() {
        let json = format!("{{\"{}\":{}}}", item.key, item.example);
        if item.kind == "condition" {
            oh_data::trigger::parse_condition(&json).unwrap();
        } else {
            oh_data::trigger::parse_effects(&format!("[{json}]")).unwrap();
        }
        assert!(item.argument_schema.is_object());
        assert!(!item.description.is_empty());
    }
    for c in [
        r#"{"date_gte":"1900-02-29"}"#,
        r#"{"stability":{"gte":"-0.1"}}"#,
        r#"{"mobilization":{"eq":"1.1"}}"#,
        r#"{"chance":"2"}"#,
        r#"{"date_gte":"2000-01-01","date_gte":"2000-01-01"}"#,
    ] {
        assert!(oh_data::trigger::parse_condition(c).is_err());
    }
    for e in [
        r#"[{"add_stability":1}]"#,
        r#"[{"add_equipment":{"equipment":"x","amount":0}}]"#,
        r#"[{"add_building":{"state":1,"building":"x","levels":-1}}]"#,
        r#"[{"scope":{"target":"state:-1","effects":[]}}]"#,
        r#"[{"if":{"condition":{"at_war":true},"then":[],"else":[],"extra":1}}]"#,
    ] {
        assert!(oh_data::trigger::parse_effects(e).is_err());
    }
}
#[test]
fn req_time_04_null_schema_refs_initial_keys_and_capabilities() {
    assert!(
        serde_json::from_str::<oh_data::trigger::EffectProgram>(r#"{"root":null,"effects":[]}"#)
            .is_err()
    );
    let l = loaded();
    let base = serde_json::to_value(&l.scenario).unwrap();
    for key in [
        "end_date",
        "end_conditions",
        "end_root",
        "flag_keys",
        "initial_flags",
        "effect_programs",
        "score_weights",
    ] {
        let mut v = base.clone();
        v[key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<oh_data::national::Scenario>(v).is_err(),
            "{key}"
        );
        let schema = serde_json::to_value(oh_data::national::scenario_schema()).unwrap();
        assert!(
            !schema["properties"][key].to_string().contains("\"null\""),
            "{key}"
        );
    }
    let mut l = loaded();
    l.scenario.end_date = Some("1999-12-31".into());
    assert!(
        oh_data::trigger::definition(&l)
            .unwrap_err()
            .contains("end_date")
    );
    l.scenario.end_date = None;
    l.scenario.flag_keys = Some(vec!["x".into(), "x".into()]);
    assert!(oh_data::trigger::definition(&l).is_err());
    l.scenario.flag_keys = Some(vec!["x".into()]);
    l.scenario.initial_flags = Some(std::collections::BTreeMap::from([(
        "NTH".into(),
        vec!["x".into(), "x".into()],
    )]));
    assert!(oh_data::trigger::definition(&l).is_err());
    l.scenario.initial_flags = None;
    for c in [
        r#"{"has_flag":"x"}"#,
        r#"{"at_war":false}"#,
        r#"{"chance":"0.5"}"#,
        r#"{"owns_state":65535}"#,
        r#"{"controls_province":65535}"#,
    ] {
        l.scenario.end_conditions = Some(oh_data::trigger::parse_condition(c).unwrap());
        assert!(oh_data::trigger::definition(&l).is_err());
        l.scenario.end_root = Some("NTH".into());
        if !c.contains("has_flag") {
            assert!(oh_data::trigger::definition(&l).is_err());
        }
        l.scenario.end_root = None;
    }
    l.scenario.end_conditions = None;
    for effect in [
        r#"[{"add_manpower":3}]"#,
        r#"[{"scope":{"target":"state:1","effects":[{"set_flag":"x"}]}}]"#,
        r#"[{"scope":{"target":"faction_leader","effects":[]}}]"#,
    ] {
        l.scenario.effect_programs = Some(std::collections::BTreeMap::from([(
            "test".into(),
            oh_data::trigger::EffectProgram {
                root: Some("NTH".into()),
                effects: oh_data::trigger::parse_effects(effect).unwrap(),
            },
        )]));
        assert!(
            oh_data::trigger::definition(&l)
                .unwrap_err()
                .contains("capability")
        );
    }
    l.scenario.effect_programs = None;
    l.scenario.score_weights = Some(oh_data::trigger::ScoreWeights {
        victory_points: "1".into(),
        industrial_capacity: "0".into(),
        survival: "0".into(),
        faction_victory: "0".into(),
    });
    assert!(
        oh_data::trigger::definition(&l)
            .unwrap_err()
            .contains("capability")
    );
}
#[test]
fn req_agd_05_actual_registered_loader_rejects_capability_with_source() {
    use std::path::Path;
    fn copy(source: &Path, target: &Path) {
        std::fs::create_dir_all(target).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target.join(entry.file_name()));
            } else {
                std::fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
            }
        }
    }
    let original = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let source = std::fs::read_to_string(original.join("scenarios/m1/scenario.toml")).unwrap();
    for (name, extra, field) in [
        (
            "war",
            "end_root = 'NTH'\nend_conditions = {at_war = false}\n",
            "end_conditions",
        ),
        (
            "effect",
            "effect_programs = {bad = {root = 'NTH', effects = [{add_manpower = 1}]}}\n",
            "effect_programs",
        ),
        (
            "flag_scope",
            "flag_keys = ['x']\neffect_programs = {bad = {root = 'NTH', effects = [{scope = {target = 'state:1', effects = [{set_flag = 'x'}]}}]}}\n",
            "effect_programs",
        ),
        ("date", "end_date = '1999-12-31'\n", "end_date"),
    ] {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/evidence/WP-13/data-rejections/{}-{name}",
            std::process::id()
        ));
        copy(&original, &root);
        let path = root.join("scenarios/m1/scenario.toml");
        std::fs::write(&path, format!("{extra}{source}")).unwrap();
        let report = oh_data::pack_validation::validate_for_purpose(
            std::slice::from_ref(&root),
            oh_data::pack_validation::ValidationPurpose::Active,
        );
        assert!(report.failed(false));
        let diagnostic = report
            .diagnostics
            .iter()
            .find(|d| d.error.path == path)
            .unwrap();
        assert_eq!(
            diagnostic.code,
            oh_data::pack_validation::DiagnosticCode::Data
        );
        assert!(diagnostic.error.message.contains(field));
        assert!(diagnostic.error.line > 0 && diagnostic.error.column > 0);
        assert!(oh_data::national::load_scenario(&root, "m1").is_err());
        std::fs::write(
            root.join("../")
                .join(format!("{name}-diagnostic-{}.txt", std::process::id())),
            diagnostic.error.to_string(),
        )
        .unwrap();
    }
}
