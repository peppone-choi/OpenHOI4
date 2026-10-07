use oh_data::pack_validation::validate_packs;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

fn fixture() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/evidence/WP-24-P06-current-comparer/data-fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
    fn copy(a: &Path, b: &Path) {
        fs::create_dir_all(b).unwrap();
        for e in fs::read_dir(a).unwrap() {
            let p = e.unwrap().path();
            let t = b.join(p.file_name().unwrap());
            if p.is_dir() {
                copy(&p, &t);
            } else {
                fs::copy(p, t).unwrap();
            }
        }
    }
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &root,
    );
    fs::create_dir_all(root.join("scenarios/legacy/nations")).unwrap();
    fs::write(
        root.join("scenarios/legacy/scenario.toml"),
        "start_date='2000-01-01'\n",
    )
    .unwrap();
    fs::write(
        root.join("scenarios/legacy/defines.toml"),
        "[time]\ninitial_speed=1\nspeed_ms_per_tick=[500,200,80,25,0]\n",
    )
    .unwrap();
    root
}
#[test]
fn registered_legacy_nation_syntax_is_not_an_uninspected_success() {
    let root = fixture();
    let path = root.join("scenarios/legacy/nations/NTH.toml");
    fs::write(&path, "# 한글\r\ninvalid [ syntax\r\n").unwrap();
    let before = fs::read(&path).unwrap();
    let r = validate_packs(std::slice::from_ref(&root));
    assert!(r.failed(false), "{r:?}");
    assert!(r.packs.is_empty());
    let e = r.diagnostics.iter().find(|d| d.error.path == path).unwrap();
    assert_eq!(e.error.kind, oh_data::ErrorKind::Syntax);
    assert_eq!(e.error.line, 2);
    assert!(e.error.column > 0);
    assert_eq!(fs::read(path).unwrap(), before);
}
#[test]
fn legacy_nation_type_intrinsics_and_unselected_context_are_errors() {
    let root = fixture();
    let path = root.join("scenarios/legacy/nations/NTH.toml");
    let valid = fs::read_to_string(root.join("scenarios/m1/nations/NTH.toml")).unwrap();
    for (old, new, cause) in [
        ("id = 1", "id = '1'", "invalid type"),
        ("id = 1", "id = 65536", "u16"),
        ("color = [40, 100, 180]", "color = [40, 256, 180]", "u8"),
        ("capital = 10", "capital = -1", "u16"),
        ("capital = 10", "unknown = 10", "unknown field"),
        ("tag = \"NTH\"", "tag = \"STH\"", "tag"),
        (
            "name_key = \"nation-testland-north\"",
            "name_key = ''",
            "name_key",
        ),
        (
            "government_key = \"government-test-republic\"",
            "government_key = ''",
            "government_key",
        ),
        ("\"0.75\"", "\"-0.75\"", "ideology"),
        ("\"0.75\"", "\"0.5\"", "ideology"),
        ("\"0.75\"", "\"invalid\"", "ideology"),
        ("capital = 10", "capital = 65535", "not selected"),
        ("id = 1", "id = 1", "not selected"),
    ] {
        fs::write(&path, valid.replace(old, new)).unwrap();
        let before = fs::read(&path).unwrap();
        let r = validate_packs(std::slice::from_ref(&root));
        assert!(r.failed(false), "{old}->{new}: {r:?}");
        let e = r.diagnostics.iter().find(|d| d.error.path == path).unwrap();
        assert!(e.error.message.contains(cause), "{cause}: {:?}", e.error);
        assert!(e.error.line > 0 && e.error.column > 0);
        assert_eq!(fs::read(&path).unwrap(), before);
    }
}
#[test]
fn normal_legacy_and_structured_nation_references_are_preserved() {
    let root = fixture();
    assert!(!validate_packs(std::slice::from_ref(&root)).failed(true));
    let file = root.join("scenarios/m1/nations/ZZZ.toml");
    fs::write(&file, "invalid [ syntax\n").unwrap();
    let r = validate_packs(std::slice::from_ref(&root));
    assert!(r.failed(false));
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.error.path == file && d.error.kind == oh_data::ErrorKind::Syntax)
    );
    fs::remove_file(&file).unwrap();
    let path = root.join("scenarios/m1/nations/NTH.toml");
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, original.replace("capital = 10", "capital = 65535")).unwrap();
    let r = validate_packs(&[root]);
    assert!(r.failed(false));
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.error.path == path && d.error.message.contains("capital"))
    );
}

#[test]
fn selected_nation_multiple_errors_keep_original_priority() {
    let root = fixture();
    let path = root.join("scenarios/m1/nations/STH.toml");
    let source = fs::read_to_string(&path).unwrap();
    let bad_ideology = source.replace("\"0.75\"", "\"bad\"");
    for (text, cause) in [
        (
            bad_ideology
                .replace("tag = \"STH\"", "tag = \"NTH\"")
                .replace("id = 2", "id = 1"),
            "tag",
        ),
        (
            bad_ideology
                .replace("id = 2", "id = 1")
                .replace("name_key = \"nation-testland-south\"", "name_key = ''"),
            "duplicate nation ID",
        ),
        (
            bad_ideology
                .replace("name_key = \"nation-testland-south\"", "name_key = ''")
                .replace("capital = 30", "capital = 65535"),
            "name_key",
        ),
        (
            bad_ideology
                .replace(
                    "government_key = \"government-test-council\"",
                    "government_key = ''",
                )
                .replace("capital = 30", "capital = 65535"),
            "government_key",
        ),
        (
            bad_ideology.replace("capital = 30", "capital = 65535"),
            "capital",
        ),
        (bad_ideology, "ideology"),
    ] {
        fs::write(&path, text).unwrap();
        let e = oh_data::national::load_scenario(&root, "m1").unwrap_err();
        assert_eq!(e.path, path);
        assert!(e.message.contains(cause), "{cause}: {e:?}");
        assert!(e.line > 0 && e.column > 0);
    }
}
