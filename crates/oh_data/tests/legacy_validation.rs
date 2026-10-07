use std::{
    fs,
    path::{Path, PathBuf},
};
fn fixture() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/evidence/WP-24-P06/legacy-fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
        .join("empty");
    fs::create_dir_all(root.join("scenarios/empty")).unwrap();
    fs::write(
        root.join("manifest.toml"),
        "id='empty'\nname_key='pack-name'\nversion='0.1.0'\nengine='>=0.1, <0.2'\n",
    )
    .unwrap();
    fs::write(
        root.join("defines.toml"),
        "[time]\nspeed_ms_per_tick=[500,200,80,25,0]\ninitial_speed=1\n",
    )
    .unwrap();
    fs::write(
        root.join("scenarios/empty/scenario.toml"),
        "start_date='2000-01-01'\n",
    )
    .unwrap();
    for l in ["ko", "en"] {
        fs::create_dir_all(root.join("localisation").join(l)).unwrap();
        fs::write(
            root.join("localisation").join(l).join("pack.ftl"),
            "pack-name = Pack\n",
        )
        .unwrap();
    }
    root
}
#[test]
fn p06_registered_legacy_defines_is_parsed_and_applied() {
    let root = fixture();
    let path = root.join("scenarios/empty/defines.toml");
    fs::write(&path, "invalid [ syntax\n").unwrap();
    let report = oh_data::pack_validation::validate_packs(std::slice::from_ref(&root));
    assert!(report.failed(true), "{:?}", report);
    let e = &report.diagnostics[0].error;
    assert_eq!(e.path, path);
    assert_eq!(e.kind, oh_data::ErrorKind::Syntax);
    for (source, field) in [
        ("[time]\ninitial_speed=0\n", "initial_speed"),
        ("[time]\nspeed_ms_per_tick=[1,2,3,4]\n", "speed_ms_per_tick"),
        (
            "[time]\nspeed_ms_per_tick=[1,2,-3,4,0]\n",
            "speed_ms_per_tick",
        ),
        ("[time]\ninitial_speed=1.0\n", "initial_speed"),
        ("[time]\ninitial_speeed=1\n", "initial_speeed"),
        ("[time]\nspeed_ms_per_tick=[[1]]\n", "define value"),
        ("[network]\ndelta_ms=99\n", "delta_ms"),
    ] {
        fs::write(&path, source).unwrap();
        let r = oh_data::pack_validation::validate_packs(std::slice::from_ref(&root));
        assert!(r.failed(false), "{source}");
        assert!(
            r.diagnostics
                .iter()
                .any(|d| d.error.message.contains(field)),
            "{:?}",
            r.diagnostics
        );
    }
    fs::write(
        &path,
        "[time]\ninitial_speed=5\nspeed_ms_per_tick=[100,90,80,70,60]\n",
    )
    .unwrap();
    assert!(!oh_data::pack_validation::validate_packs(std::slice::from_ref(&root)).failed(true));
    let loaded = oh_data::m0::load_m0_scenario(root.parent().unwrap(), "empty").unwrap();
    assert_eq!(
        loaded.pack.defines.get("time.initial_speed"),
        Some(&oh_data::DefineValue::Number(oh_data::Number::Integer(5)))
    );
    fs::remove_file(path).unwrap();
    let loaded = oh_data::m0::load_m0_scenario(root.parent().unwrap(), "empty").unwrap();
    assert_eq!(
        loaded.pack.defines.get("time.initial_speed"),
        Some(&oh_data::DefineValue::Number(oh_data::Number::Integer(1)))
    );
}

#[test]
fn p06_legacy_missing_fields_types_ranges_and_source_context() {
    let root = fixture();
    let p = root.join("scenarios/empty/defines.toml");
    for text in [
        "[time]\ninitial_speed=6\n",
        "[time]\ninitial_speed='1'\n",
        "[time]\nspeed_ms_per_tick=[1,2,3,4,5,6]\n",
        "[time]\nspeed_ms_per_tick=[1,2,3,4,5.0]\n",
        "[time]\ninitial_speed=9223372036854775808\n",
        "[network]\ncommand_capacity=[]\n",
        "[network]\nunknown=1\n",
    ] {
        fs::write(&p, text).unwrap();
        let before = oh_data::pack_validation::content_hash(&root).unwrap();
        let r = oh_data::pack_validation::validate_packs(std::slice::from_ref(&root));
        assert!(r.failed(false), "{text}");
        assert_eq!(r.diagnostics[0].error.path, p);
        assert_eq!(r.diagnostics[0].error.line, 2);
        assert_eq!(
            before,
            oh_data::pack_validation::content_hash(&root).unwrap()
        );
    }
    fs::write(&p, "[time]\ninitial_speed=5\n").unwrap();
    fs::write(root.join("defines.toml"), "").unwrap();
    let r = oh_data::pack_validation::validate_packs(std::slice::from_ref(&root));
    assert!(r.failed(false));
    assert!(r.diagnostics[0].error.message.contains("speed_ms_per_tick"));
    fs::write(
        root.join("defines.toml"),
        "[time]\nspeed_ms_per_tick=[0,0,0,0,0]\ninitial_speed=1\n",
    )
    .unwrap();
    assert!(!oh_data::pack_validation::validate_packs(std::slice::from_ref(&root)).failed(true));
    fs::write(
        root.join("scenarios/empty/scenario.toml"),
        "start_date='2000-01-01'\nnation='missing'\n",
    )
    .unwrap();
    let r = oh_data::pack_validation::validate_packs(&[root]);
    assert!(r.failed(false));
    assert!(r.diagnostics[0].error.message.contains("unknown field"));
}

#[test]
fn p06_compatibility_is_bound_to_original_bytes_and_purpose() {
    use oh_data::pack_validation::{ValidationPurpose, content_hash, validate_for_purpose};
    let original =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/examples/m0/testland");
    assert_eq!(content_hash(&original).unwrap(), 8_747_082_838_669_556_868);
    assert!(
        validate_for_purpose(std::slice::from_ref(&original), ValidationPurpose::Strict)
            .failed(false)
    );
    let purpose = ValidationPurpose::ServerStartup {
        scenario_id: "testland".into(),
        kind: oh_data::host_policy::ScenarioKind::Empty,
    };
    let active = validate_for_purpose(std::slice::from_ref(&original), purpose.clone());
    assert!(!active.failed(false));
    assert!(active.failed(true));
    assert_eq!(active.diagnostics.len(), 2);
    let p = fixture();
    fs::write(
        p.join("defines.toml"),
        fs::read(original.join("defines.toml")).unwrap(),
    )
    .unwrap();
    fs::write(
        p.join("manifest.toml"),
        fs::read(original.join("manifest.toml")).unwrap(),
    )
    .unwrap();
    for l in ["ko", "en"] {
        fs::remove_file(p.join("localisation").join(l).join("pack.ftl")).unwrap();
    }
    // Same ID/version/key and valid data do not qualify when any bytes differ.
    assert!(validate_for_purpose(std::slice::from_ref(&p), purpose).failed(false));
    let frozen = Path::new(env!("CARGO_MANIFEST_DIR")).join("../oh_save/tests/fixtures/m1-pack-v1");
    assert!(
        validate_for_purpose(std::slice::from_ref(&frozen), ValidationPurpose::Active)
            .failed(false)
    );
    assert!(validate_for_purpose(&[frozen], ValidationPurpose::RestoreV1).failed(false));
}

#[test]
fn p06_missing_manifest_diagnostic_is_structured_not_a_text_filter() {
    use oh_data::pack_validation::{DiagnosticCode, validate_packs};
    let original =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/examples/m0/testland");
    let r = validate_packs(&[original]);
    assert!(r.failed(false));
    assert_eq!(r.diagnostics.len(), 2);
    let expected = ["ko", "en"];
    for (d, locale) in r.diagnostics.iter().zip(expected) {
        assert_eq!(
            d.code,
            DiagnosticCode::MissingMessageValue {
                locale: locale.into(),
                key: "testland_name".into(),
                source_field: Some("manifest.name_key".into())
            }
        );
        assert_eq!((d.error.line, d.error.column), (3, 12));
    }
    let root = fixture();
    fs::write(root.join("localisation/en/pack.ftl"), "broken = {\n").unwrap();
    let r = validate_packs(&[root]);
    assert!(r.failed(false));
    assert_eq!(r.diagnostics[0].code, DiagnosticCode::Data);
}
