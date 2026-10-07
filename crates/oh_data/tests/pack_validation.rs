use oh_data::pack_validation::{content_hash, resolve_packs, validate_packs};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
fn root() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let base = std::env::var_os("OH_WP24_EVIDENCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/evidence/WP-24")
        });
    let p = base.join("data-fixtures").join(format!(
        "{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&p).unwrap();
    p
}
fn pack(p: &Path, id: &str, extra: &str) -> PathBuf {
    let p = p.join(id);
    fs::create_dir_all(&p).unwrap();
    fs::write(p.join("manifest.toml"),format!("id = \"{id}\"\nname_key = \"pack-name\"\nversion = \"0.1.0\"\nengine = \">=0.1, <0.2\"\n{extra}\n")).unwrap();
    fs::write(p.join("defines.toml"), "").unwrap();
    for l in ["ko", "en"] {
        fs::create_dir_all(p.join("localisation").join(l)).unwrap();
        fs::write(
            p.join("localisation").join(l).join("pack.ftl"),
            "pack-name = Name\n",
        )
        .unwrap();
    }
    p
}
#[test]
fn graph_errors_point_to_original_manifest_values() {
    for (extra, message, column) in [
        (
            "depends = [{ id = \"missing\", version = \"*\" }]",
            "missing dependency",
            19,
        ),
        ("load_after = [\"missing\"]", "missing load_after", 15),
        (
            "depends = [{ id = \"a\", version = \"*\" }]",
            "self dependency",
            19,
        ),
        ("conflicts = [\"a\"]", "self conflicts", 14),
        ("load_after = [\"a\"]", "self load_after", 15),
        (
            "depends = [{ id = \"b\", version = \">=0.2\" }]",
            "does not match",
            34,
        ),
        (
            "depends = [{ id = \"b\", version = \"*\" }, { id = \"b\", version = \"*\" }]",
            "duplicate",
            48,
        ),
        ("conflicts = [\"absent\", \"absent\"]", "duplicate", 24),
        ("load_after = [\"b\", \"b\"]", "duplicate", 20),
    ] {
        let p = root();
        let a = pack(&p, "a", extra);
        let b = pack(&p, "b", "");
        let e = resolve_packs(&[a.clone(), b]).unwrap_err();
        assert!(e.message.contains(message), "{e}");
        assert_eq!(e.path, a.join("manifest.toml"));
        assert_eq!(e.line, 5);
        assert_eq!(e.column, column, "{e}");
    }
}
#[test]
fn cycles_conflicts_duplicates_and_engine_ranges() {
    let p = root();
    let a = pack(&p, "a", "depends = [{ id = \"b\", version = \"*\" }]");
    let b = pack(&p, "b", "load_after = [\"a\"]");
    assert!(
        resolve_packs(&[a.clone(), b.clone()])
            .unwrap_err()
            .message
            .contains("cycle")
    );
    let b = pack(&p, "b", "conflicts = [\"a\"]");
    let a = pack(&p, "a", "");
    let e = resolve_packs(&[a.clone(), b.clone()]).unwrap_err();
    assert_eq!(e.path, b.join("manifest.toml"));
    assert!(e.message.contains("conflicts"));
    assert!(
        resolve_packs(&[a.clone(), a.clone()])
            .unwrap_err()
            .message
            .contains("duplicate pack")
    );
    let manifest = a.join("manifest.toml");
    let s = fs::read_to_string(&manifest).unwrap();
    fs::write(&manifest, s.replace(">=0.1, <0.2", ">=0.2")).unwrap();
    let e = resolve_packs(&[a]).unwrap_err();
    assert_eq!((e.line, e.column), (4, 10));
    assert!(e.message.contains("incompatible"));
}
#[test]
fn lexical_ready_set_tie_and_redundant_edge() {
    let p = root();
    let z = pack(&p, "z", "");
    let a = pack(
        &p,
        "a",
        "depends = [{ id = \"z\", version = \"*\" }]\nload_after = [\"z\"]",
    );
    let b = pack(&p, "b", "conflicts = [\"absent\"]");
    let r = resolve_packs(&[a, b, z]).unwrap();
    assert_eq!(
        r.iter()
            .map(|p| p.data.manifest.id.as_str())
            .collect::<Vec<_>>(),
        ["b", "z", "a"]
    );
}
#[test]
fn hash_is_path_independent_and_changes_with_content() {
    let p = root();
    let a = pack(&p, "a", "");
    let b = p.join("copy");
    copy(&a, &b);
    assert_eq!(content_hash(&a).unwrap(), content_hash(&b).unwrap());
    fs::write(b.join("defines.toml"), "# bytes matter\n").unwrap();
    assert_ne!(content_hash(&a).unwrap(), content_hash(&b).unwrap());
}
fn copy(a: &Path, b: &Path) {
    fs::create_dir_all(b).unwrap();
    for e in fs::read_dir(a).unwrap() {
        let p = e.unwrap().path();
        let dest = b.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &dest)
        } else {
            fs::copy(p, dest).unwrap();
        }
    }
}
#[test]
fn actual_registered_schema_refs_and_ftl_are_not_manifest_only() {
    for (file, old, new, expected) in [
        (
            "maps/testland/provinces.csv",
            "plains",
            "missing",
            "unknown terrain",
        ),
        (
            "scenarios/m1/scenario.toml",
            "map = \"testland\"",
            "unknown = \"testland\"",
            "unknown field",
        ),
        (
            "maps/testland/states.toml",
            "population = 1000",
            "population = -1",
            "population",
        ),
        (
            "scenarios/m1/nations/NTH.toml",
            "capital = 10",
            "capital = 65535",
            "capital",
        ),
    ] {
        let p = root();
        copy(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
            &p,
        );
        let path = p.join(file);
        let s = fs::read_to_string(&path).unwrap();
        assert!(s.contains(old), "fixture premise: {file}");
        fs::write(&path, s.replace(old, new)).unwrap();
        let r = validate_packs(&[p]);
        assert!(r.failed(false));
        assert!(r.packs.is_empty());
        assert!(
            r.diagnostics
                .iter()
                .any(|d| d.error.message.contains(expected)),
            "{:?}",
            r.diagnostics
        );
    }
}
#[test]
fn fluent_syntax_refs_attributes_duplicates_cycles_and_parity() {
    for (text, expected) in [
        (
            "pack-name = { absent }\n",
            "missing Fluent reference absent",
        ),
        (
            "pack-name = { -absent }\n",
            "missing Fluent reference -absent",
        ),
        (
            "pack-name = { other.title }\nother = Yes\n",
            "missing Fluent reference other.title",
        ),
        ("pack-name = { other }\nother = { pack-name }\n", "cycle"),
        ("pack-name = One\npack-name = Two\n", "duplicate"),
        ("pack-name = {\n", "invalid Fluent"),
        (
            "pack-name = Name\n    .title = One\n    .title = Two\n",
            "duplicate",
        ),
        ("pack-name = { UNKNOWN() }\n", "function:UNKNOWN"),
    ] {
        let p = root();
        let a = pack(&p, "a", "");
        for l in ["ko", "en"] {
            fs::write(a.join("localisation").join(l).join("pack.ftl"), text).unwrap();
        }
        let r = validate_packs(&[a]);
        assert!(r.failed(false));
        assert!(
            r.diagnostics
                .iter()
                .any(|d| d.error.message.contains(expected)),
            "{:?}",
            r.diagnostics
        );
    }
    let p = root();
    let a = pack(&p, "a", "");
    fs::write(
        a.join("localisation/ko/pack.ftl"),
        "pack-name = Name\n    .title = Title\n",
    )
    .unwrap();
    let r = validate_packs(&[a]);
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.error.message.contains("missing pack-name.title in en"))
    );
}
#[test]
fn fluent_duplicate_across_files_and_unicode_crlf_location() {
    let p = root();
    let a = pack(&p, "a", "");
    fs::write(
        a.join("localisation/ko/second.ftl"),
        "pack-name = Duplicate\n",
    )
    .unwrap();
    let r = validate_packs(std::slice::from_ref(&a));
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.error.path.ends_with("second.ftl")
                && d.error.line == 1
                && d.error.message.contains("duplicate"))
    );
    fs::remove_file(a.join("localisation/ko/second.ftl")).unwrap();
    fs::write(
        a.join("localisation/ko/pack.ftl"),
        "# 한글\r\npack-name = { absent }\r\n",
    )
    .unwrap();
    let r = validate_packs(&[a]);
    let e = &r
        .diagnostics
        .iter()
        .find(|d| d.error.message.contains("reference absent"))
        .unwrap()
        .error;
    assert_eq!((e.line, e.column), (2, 15));
}

#[test]
fn map_warning_provenance_and_failure_policy_survive_validation() {
    let p = root();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &p,
    );
    let map = oh_data::map::load_map(&p, "testland").unwrap();
    let mut index = map.index.clone();
    index[47] = 0;
    let bytes: Vec<_> = index
        .iter()
        .flat_map(|&i| map.provinces[i as usize].rgb)
        .collect();
    let file = fs::File::create(p.join("maps/testland/provinces.png")).unwrap();
    let mut encoder = png::Encoder::new(file, 8, 6);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&bytes)
        .unwrap();
    let before = content_hash(&p).unwrap();
    let report = validate_packs(std::slice::from_ref(&p));
    assert!(!report.failed(false), "{:?}", report.diagnostics);
    assert!(report.failed(true));
    let e = &report
        .diagnostics
        .iter()
        .find(|d| d.error.message.contains("2 disconnected components"))
        .unwrap()
        .error;
    assert_eq!((e.line, e.column), (2, 1));
    assert!(e.path.ends_with("provinces.csv"));
    assert!(e.message.contains("province 10"));
    assert_eq!(before, content_hash(&p).unwrap());
}

#[test]
fn unsupported_adapters_are_visible_and_empty_selection_is_not_success() {
    assert!(resolve_packs(&[]).is_err());
    let p = root();
    let a = pack(&p, "a", "");
    fs::create_dir_all(a.join("common")).unwrap();
    fs::write(a.join("common/equipment.toml"), "invalid future syntax").unwrap();
    let report = validate_packs(&[a]);
    assert!(!report.failed(false));
    assert!(report.failed(true));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.error.path.ends_with("equipment.toml")
                && d.error.message.contains("not validated"))
    );
}

#[test]
fn semver_prerelease_and_cross_locale_value_boundary() {
    let p = root();
    let a = pack(&p, "a", "depends = [{ id = \"b\", version = \"*\" }]");
    let b = pack(&p, "b", "");
    let manifest = b.join("manifest.toml");
    let text = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        text.replace("version = \"0.1.0\"", "version = \"0.1.0-alpha.1\""),
    )
    .unwrap();
    assert!(
        resolve_packs(&[a.clone(), b.clone()])
            .unwrap_err()
            .message
            .contains("does not match")
    );
    let manifest = a.join("manifest.toml");
    let text = fs::read_to_string(&manifest).unwrap();
    fs::write(
        manifest,
        text.replace("version = \"*\"", "version = \">=0.1.0-alpha.1\""),
    )
    .unwrap();
    assert!(resolve_packs(&[a.clone(), b]).is_ok());
    fs::write(
        a.join("localisation/en/pack.ftl"),
        "pack-name =\n    .title = Attribute without value\n",
    )
    .unwrap();
    // Single pack below intentionally removes its dependency, isolating locale checks.
    let a = pack(&p, "a", "");
    fs::write(
        a.join("localisation/en/pack.ftl"),
        "pack-name =\n    .title = Attribute without value\n",
    )
    .unwrap();
    let r = validate_packs(&[a]);
    assert!(r.diagnostics.iter().any(|d| {
        d.error
            .message
            .contains("missing en message value pack-name")
    }));
}
