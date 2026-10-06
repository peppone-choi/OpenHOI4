//! REQ-GEN-02 / AC-M0-03: real on-disk loader fixtures.
use oh_data::{DefineValue, ErrorKind, Fixed, Number, load_pack};
use std::path::PathBuf;
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
fn failure(name: &str, file: &str, kind: ErrorKind, line: usize, column: usize) {
    let err = load_pack(fixture(name)).expect_err(name);
    assert_eq!(err.path, fixture(name).join(file), "{err}");
    assert_eq!(err.kind, kind, "{err}");
    assert_eq!((err.line, err.column), (line, column), "{err}");
    assert!(
        err.to_string()
            .starts_with(&format!("{}:{line}:{column} — ", err.path.display()))
    );
    assert!(!err.message.is_empty());
}
#[test]
fn req_gen_02_manifest_and_data_defined_numbers() {
    let pack = load_pack(fixture("valid")).unwrap();
    assert_eq!(pack.manifest.id, "testland");
    assert_eq!(pack.manifest.version, "0.1.0");
    assert_eq!(pack.manifest.engine, ">=0.1, <0.2");
    assert_eq!(pack.manifest.name_key, "testland_name");
    assert_eq!(pack.manifest.depends[0].id, "base");
    assert_eq!(pack.manifest.depends[0].version, ">=0.1");
    assert_eq!(pack.manifest.conflicts, ["other_pack"]);
    assert_eq!(pack.manifest.load_after, ["base"]);
    for (key, number) in [
        ("count", Number::Integer(i64::MAX)),
        ("ratio", Number::Fixed("0.1".parse::<Fixed>().unwrap())),
        ("negative", Number::Fixed("-2.5".parse::<Fixed>().unwrap())),
        (
            "exponent",
            Number::Fixed("0.0125".parse::<Fixed>().unwrap()),
        ),
        (
            "separated",
            Number::Fixed("1000.25".parse::<Fixed>().unwrap()),
        ),
        ("hex_count", Number::Integer(16)),
    ] {
        assert_eq!(
            pack.defines.get(&format!("fixture.{key}")),
            Some(&DefineValue::Number(number))
        );
    }
    assert_eq!(
        pack.defines.get("fixture.steps"),
        Some(&DefineValue::Array(
            vec![500, 200, 80, 25, 0]
                .into_iter()
                .map(Number::Integer)
                .collect()
        ))
    );
    assert_eq!(
        pack.defines.get("fixture.mixed"),
        Some(&DefineValue::Array(vec![
            Number::Integer(1),
            Number::Fixed("0.5".parse().unwrap()),
            Number::Integer(-2)
        ]))
    );
    assert_eq!(pack.defines.get("fixture.missing"), None);
    assert_eq!(pack.defines.get("missing.count"), None);
    assert_eq!(pack.defines.get("count"), None);
}
#[test]
fn req_gen_02_no_gameplay_defaults_in_skeleton() {
    let pack = load_pack(fixture("empty")).unwrap();
    assert!(pack.defines.0.is_empty());
    assert!(pack.manifest.depends.is_empty());
    assert!(pack.manifest.conflicts.is_empty());
    assert!(pack.manifest.load_after.is_empty());
    let shipped =
        load_pack(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"))
            .unwrap();
    assert!(shipped.defines.0.is_empty());
    assert!(shipped.manifest.depends.is_empty());
}
#[test]
fn req_gen_02_repeat_load_and_order_are_stable() {
    let first = load_pack(fixture("valid")).unwrap();
    assert_eq!(first, load_pack(fixture("valid")).unwrap());
    let keys: Vec<_> = first.defines.0["fixture"]
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "count",
            "exponent",
            "hex_count",
            "mixed",
            "negative",
            "ratio",
            "separated",
            "steps"
        ]
    );
}
#[test]
fn ac_m0_03_manifest_schema_errors_have_source_locations() {
    for (name, line, column) in [
        ("bad_manifest_type", 3, 11),
        ("unknown_manifest", 5, 1),
        ("missing_manifest", 1, 1),
        ("bad_id", 1, 6),
        ("bad_version", 3, 11),
        ("bad_engine", 4, 10),
        ("bad_dependency", 5, 19),
        ("bad_dependency_version", 5, 37),
        ("bad_conflict", 5, 14),
        ("bad_load_after", 5, 15),
        ("bad_name", 2, 12),
    ] {
        failure(name, "manifest.toml", ErrorKind::Schema, line, column);
    }
}
#[test]
fn ac_m0_03_defines_schema_errors_have_source_locations() {
    for (name, line, column) in [
        ("bad_defines_bool", 2, 9),
        ("bad_defines_string", 2, 9),
        ("bad_defines_date", 2, 8),
        ("bad_defines_root", 1, 9),
        ("bad_defines_nested", 1, 1),
        ("bad_defines_array", 2, 13),
        ("bad_defines_nested_array", 2, 10),
        ("bad_defines_nan", 2, 9),
        ("bad_defines_inf", 2, 9),
        ("bad_defines_range", 2, 9),
        ("unicode_crlf", 3, 8),
    ] {
        failure(name, "defines.toml", ErrorKind::Schema, line, column);
    }
}
#[test]
fn ac_m0_03_toml_syntax_errors_have_source_locations() {
    failure("syntax_manifest", "manifest.toml", ErrorKind::Syntax, 5, 12);
    failure("syntax_defines", "defines.toml", ErrorKind::Syntax, 2, 12);
}
#[test]
fn ac_m0_03_missing_file_has_path_and_io_diagnostic() {
    failure("not_present", "manifest.toml", ErrorKind::Io, 1, 1);
}
#[test]
fn ac_m0_03_schema_snapshots_match_generated_types() {
    for (name, schema) in [
        ("manifest", oh_data::manifest_schema()),
        ("defines", oh_data::defines_schema()),
    ] {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("schema/{name}.schema.json"));
        let expected: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(schema).unwrap(), expected);
        assert_eq!(expected["type"], "object");
    }
}

#[test]
fn req_gen_02_decimal_precision_does_not_round_through_float() {
    let pack = load_pack(fixture("precision")).unwrap();
    // Just above a half-ULP boundary. Conversion through f64 would lose this bit.
    assert_eq!(
        pack.defines.get("fixture.ratio"),
        Some(&DefineValue::Number(Number::Fixed(Fixed::from_bits(
            (1_i64 << 32) + 1
        ))))
    );
}
#[test]
fn req_gen_02_values_change_with_data() {
    let original = load_pack(fixture("valid")).unwrap();
    let changed = load_pack(fixture("changed")).unwrap();
    assert_ne!(
        original.defines.get("fixture.ratio"),
        changed.defines.get("fixture.ratio")
    );
    assert_eq!(
        changed.defines.get("fixture.count"),
        Some(&DefineValue::Number(Number::Integer(-5)))
    );
    assert_eq!(
        changed.defines.get("fixture.ratio"),
        Some(&DefineValue::Number(Number::Fixed("0.2".parse().unwrap())))
    );
}
#[test]
fn ac_m0_03_additional_io_and_schema_cases() {
    failure("missing_defines", "defines.toml", ErrorKind::Io, 1, 1);
    failure(
        "missing_dependency_version",
        "manifest.toml",
        ErrorKind::Schema,
        5,
        12,
    );
    failure("bad_integer_range", "defines.toml", ErrorKind::Schema, 2, 9);
    failure(
        "bad_multiline_array",
        "defines.toml",
        ErrorKind::Schema,
        4,
        5,
    );
    failure("bad_duplicate", "defines.toml", ErrorKind::Syntax, 3, 1);
}

#[test]
fn ac_m0_03_crlf_unicode_error_location() {
    // Write an actual CRLF input under the ignored workspace target directory;
    // Git text normalization cannot silently turn this into an LF-only test.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wp03")
        .join(format!("crlf-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::copy(
        fixture("empty").join("manifest.toml"),
        root.join("manifest.toml"),
    )
    .unwrap();
    let text = std::fs::read_to_string(fixture("unicode_crlf").join("defines.toml")).unwrap();
    let text = text.replace("\r\n", "\n").replace('\n', "\r\n");
    std::fs::write(root.join("defines.toml"), text.as_bytes()).unwrap();
    assert!(text.as_bytes().windows(2).any(|pair| pair == b"\r\n"));
    let err = load_pack(&root).unwrap_err();
    assert_eq!(err.path, root.join("defines.toml"));
    assert_eq!(err.kind, ErrorKind::Schema);
    assert_eq!((err.line, err.column), (3, 8), "{err}");
}
