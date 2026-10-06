use std::path::Path;
#[test]
fn req_nat_01_validated_data_and_references() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    assert_eq!(loaded.nations.len(), 2);
    assert_eq!(loaded.nations[0].tag, "NTH");
    assert_eq!(loaded.nations[0].capital, 10);
    assert_eq!(loaded.nations[0].color, [40, 100, 180]);
    assert_eq!(loaded.scenario.ownership[&1], "NTH");
    assert_eq!(loaded.scenario.control_overrides[&20], "STH");
}

fn fixture() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wp09/fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    fn copy(source: &Path, target: &Path) {
        std::fs::create_dir_all(target).unwrap();
        for e in std::fs::read_dir(source).unwrap() {
            let p = e.unwrap().path();
            let t = target.join(p.file_name().unwrap());
            if p.is_dir() {
                copy(&p, &t)
            } else {
                std::fs::copy(&p, &t).unwrap();
            }
        }
    }
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &root,
    );
    root
}
#[test]
fn req_nat_01_schema_ranges_and_reference_rejections_are_atomic() {
    let cases = &[
        (
            "scenarios/m1/nations/NTH.toml",
            "capital = 10",
            "capital = 65535",
        ),
        (
            "scenarios/m1/nations/NTH.toml",
            "color = [40, 100, 180]",
            "color = [256, 100, 180]",
        ),
        (
            "scenarios/m1/nations/NTH.toml",
            "color = [40, 100, 180]",
            "color = [40, 100]",
        ),
        (
            "scenarios/m1/nations/NTH.toml",
            "government_key =",
            "unknown =",
        ),
        ("scenarios/m1/nations/STH.toml", "id = 2", "id = 1"),
        (
            "scenarios/m1/nations/NTH.toml",
            "tag = \"NTH\"",
            "tag = \"BAD\"",
        ),
        ("scenarios/m1/nations/NTH.toml", "\"0.75\"", "\"1.25\""),
        ("scenarios/m1/nations/NTH.toml", "\"0.75\"", "\"0.5\""),
        ("scenarios/m1/nations/NTH.toml", "\"0.75\"", "\"nan\""),
        ("scenarios/m1/scenario.toml", "1 = \"NTH\"", "1 = \"BAD\""),
        ("scenarios/m1/scenario.toml", "2 = \"STH\"", "3 = \"STH\""),
        ("scenarios/m1/scenario.toml", "20 = \"STH\"", "50 = \"STH\""),
        ("scenarios/m1/scenario.toml", "20 = \"STH\"", "20 = \"BAD\""),
        (
            "scenarios/m1/scenario.toml",
            "[\"NTH\", \"STH\"]",
            "[\"../NTH\", \"STH\"]",
        ),
        (
            "maps/testland/visuals.toml",
            "1 = [140, 95, 170]",
            "3 = [140, 95, 170]",
        ),
    ];
    for (file, from, to) in cases {
        let root = fixture();
        let path = root.join(file);
        let original = std::fs::read_to_string(&path).unwrap();
        assert!(original.contains(from), "{from}");
        std::fs::write(&path, original.replace(from, to)).unwrap();
        let error = oh_data::national::load_scenario(&root, "m1").unwrap_err();
        assert!(error.path.ends_with(file), "{error}");
        assert!(error.line > 0 && error.column > 0);
    }
    assert!(oh_data::national::load_scenario(&fixture(), "../m1").is_err());
}
#[test]
fn req_nat_01_generated_schemas_are_current() {
    for (name, schema) in [
        ("nation", oh_data::national::nation_schema()),
        ("scenario", oh_data::national::scenario_schema()),
        ("visuals", oh_data::national::visuals_schema()),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("schema")
            .join(format!("{name}.schema.json"));
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            format!("{}\n", serde_json::to_string_pretty(&schema).unwrap())
        );
    }
}
