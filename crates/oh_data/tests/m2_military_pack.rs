#[path = "support/m2_military_pack.rs"]
mod fixture;
use oh_data::pack_validation::{ValidationPurpose, validate_for_purpose};
#[test]
fn strict_six_nation_input_and_all_model_bindings() {
    let p = fixture::CopyPack::new();
    let l = oh_data::national::load_scenario(p.root(), fixture::SCENARIO).unwrap();
    assert_eq!(
        (l.nations.len(), l.map.provinces.len(), l.map.states.len()),
        (6, 120, 12)
    );
    let production = l.production.as_ref().unwrap();
    assert_eq!(production.models.len(), 2);
    let m = l.military.as_ref().unwrap();
    assert_eq!(m.templates().templates().len(), 1);
    assert_eq!(m.training_days.len(), 1);
    assert_eq!(m.training_days["m2_small"], 2);
    assert_eq!(m.armies.len(), 6);
    assert!(m.divisions.is_empty());
    for n in 1..=6 {
        let input = &production.nations[&n];
        assert!(input.allowed_models.contains("m2_equipment_1"));
        assert_eq!(
            input.allowed_models.iter().collect::<Vec<_>>(),
            input.stock.keys().collect::<Vec<_>>()
        );
        assert!(input.stock.values().all(|v| *v == 0));
        m.templates().validate_bindings(&l, n, "m2_small").unwrap();
        assert_eq!(
            (m.background[&n].committed, m.background[&n].reserved),
            (0, 0)
        );
        let a = &m.armies[&u64::from(n - 1)];
        assert_eq!((a.nation, a.capacity, a.priority), (n, 2, 0));
        assert_eq!(a.general, format!("synthetic_n{n:02}"));
    }
    let r = validate_for_purpose(&[p.root().to_owned()], ValidationPurpose::Strict);
    assert!(!r.failed(true), "{:?}", r.diagnostics);
}
#[test]
fn semantic_rejections_with_unmodified_valid_controls() {
    let military = "common/military/initial.toml";
    for (file, from, to, reason) in [
        (
            "common/production/initial.toml",
            "[nations.3]\nallowed_models = [\"m2_equipment_1\", \"m2_equipment_2\"]\nstock = { m2_equipment_1 = 0, m2_equipment_2 = 0 }",
            "[nations.3]\nallowed_models = [\"m2_equipment_2\"]\nstock = { m2_equipment_2 = 0 }",
            "DisallowedModel",
        ),
        (
            military,
            "6 = { committed = 0, reserved = 0 }",
            "",
            "InvalidMilitaryBackgroundNations",
        ),
        (military, "nation = 6", "nation = 7", "InvalidMilitaryArmy"),
        (
            military,
            "m2_small = 2",
            "m2_small = 0",
            "InvalidTrainingDays",
        ),
        (
            military,
            "m2_small = 2",
            "missing = 2",
            "MissingTrainingDays",
        ),
        (
            military,
            "m2_equipment_1",
            "missing_model",
            "BindingContextMismatch",
        ),
        (
            military,
            "divisions = []",
            "divisions = [{id=0,army=99,template='m2_small',province=1,manpower=0,equipment={}}]",
            "InvalidDivisionArmy",
        ),
        (
            "scenarios/m2_military/scenario.toml",
            "production = \"initial\"\n",
            "",
            "MilitaryRequiresEconomyAndProduction",
        ),
    ] {
        let p = fixture::CopyPack::new();
        assert!(oh_data::national::load_scenario(p.root(), fixture::SCENARIO).is_ok());
        p.alter(file, from, to);
        let e = oh_data::national::load_scenario(p.root(), fixture::SCENARIO)
            .unwrap_err()
            .to_string();
        assert!(e.contains(reason), "expected {reason}, got {e}");
    }
}
#[test]
fn strict_unused_input_and_missing_pack_name_are_rejected() {
    for unused in [true, false] {
        let p = fixture::CopyPack::new();
        assert!(
            !validate_for_purpose(&[p.root().to_owned()], ValidationPurpose::Strict).failed(true)
        );
        if unused {
            std::fs::write(p.root().join("common/military/unused.toml"), "").unwrap();
        } else {
            std::fs::write(p.root().join("localisation/en/pack.ftl"), "").unwrap();
        }
        assert!(
            validate_for_purpose(&[p.root().to_owned()], ValidationPurpose::Strict).failed(true)
        );
    }
}

#[test]
fn missing_model_in_both_contexts_and_duplicate_general_same_nation() {
    for duplicate in [false, true] {
        let p = fixture::CopyPack::new();
        let path = p.root().join("common/military/initial.toml");
        let s = std::fs::read_to_string(&path).unwrap();
        let bad = if duplicate {
            s.replace("nation = 2", "nation = 1")
                .replace("synthetic_n02", "synthetic_n01")
        } else {
            s.replace("m2_equipment_1", "missing_model")
        };
        std::fs::write(path, bad).unwrap();
        let e = oh_data::national::load_scenario(p.root(), fixture::SCENARIO)
            .unwrap_err()
            .to_string();
        assert!(
            e.contains(if duplicate {
                "InvalidMilitaryArmy"
            } else {
                "InvalidReference"
            }),
            "{e}"
        );
    }
}
// Only fixture relative names use this key; no product loader policy changes.
fn relative_key(relative: &str) -> Result<String, &'static str> {
    let key = relative.replace('\\', "/");
    if key
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == ".." || part.contains(':'))
    {
        return Err("non-canonical relative pack path");
    }
    Ok(key)
}
fn has_authorized_delta(relative: &str) -> bool {
    matches!(
        relative,
        "manifest.toml"
            | "README.md"
            | "SOURCES.md"
            | "common/production/initial.toml"
            | "localisation/en/pack.ftl"
            | "localisation/ko/pack.ftl"
            | "localisation/en/production.ftl"
            | "localisation/ko/production.ftl"
            | "scenarios/m2_production/scenario.toml"
    )
}
fn mapped_relative(relative: &str) -> String {
    if let Some(tail) = relative.strip_prefix("scenarios/m2_production/") {
        format!("scenarios/m2_military/{tail}")
    } else {
        relative.into()
    }
}
fn pack_files(p: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut v = Vec::new();
    for entry in std::fs::read_dir(p).unwrap() {
        let entry = entry.unwrap().path();
        if entry.is_dir() {
            v.extend(pack_files(&entry));
        } else {
            v.push(entry);
        }
    }
    v
}
fn inherited_pack_audit(
    base: &std::path::Path,
    r: &std::path::Path,
    display_path: fn(&std::path::Path) -> String,
) -> Result<(), String> {
    if pack_files(r).len() != 32 {
        return Err("file count".into());
    }
    for p in pack_files(base) {
        let relative = relative_key(&display_path(p.strip_prefix(base).unwrap()))?;
        if has_authorized_delta(&relative) {
            continue;
        }
        if std::fs::read(&p).unwrap() != std::fs::read(r.join(mapped_relative(&relative))).unwrap()
        {
            return Err(relative);
        }
    }
    for language in ["en", "ko"] {
        let path = format!("localisation/{language}/production.ftl");
        let inherited = std::fs::read_to_string(base.join(&path)).unwrap();
        let expected = inherited
            .lines()
            .filter(|line| !line.starts_with("o2-pack-name ="))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        if std::fs::read_to_string(r.join(&path)).unwrap() != expected {
            return Err(path);
        }
    }
    let scenario = std::fs::read_to_string(base.join("scenarios/m2_production/scenario.toml"))
        .unwrap()
        .replace(
            "# Synthetic economy and production; no military or victory rules.",
            "# Synthetic military fixture; no combat or victory rules.",
        );
    if std::fs::read_to_string(r.join("scenarios/m2_military/scenario.toml")).unwrap()
        != format!("military = \"initial\"\n{scenario}")
    {
        return Err("scenarios/m2_military/scenario.toml".into());
    }
    let mut expected =
        std::fs::read_to_string(base.join("common/production/initial.toml")).unwrap();
    for n in [3, 6] {
        expected=expected.replace(&format!("[nations.{n}]\nallowed_models = [\"m2_equipment_2\"]\nstock = {{ m2_equipment_2 = 0 }}"),&format!("[nations.{n}]\nallowed_models = [\"m2_equipment_1\", \"m2_equipment_2\"]\nstock = {{ m2_equipment_1 = 0, m2_equipment_2 = 0 }}"));
    }
    if std::fs::read_to_string(r.join("common/production/initial.toml")).unwrap()
        != format!(
            "# TEST FIXTURE DESIGN: common model1 permits the existing all-nation template loader.\n{expected}"
        )
    {
        return Err("common/production/initial.toml".into());
    }
    Ok(())
}
fn native_path(p: &std::path::Path) -> String {
    p.to_str().unwrap().into()
}
fn linux_path(p: &std::path::Path) -> String {
    p.to_str().unwrap().replace('\\', "/")
}
fn windows_path(p: &std::path::Path) -> String {
    linux_path(p).replace('/', "\\")
}
#[test]
fn immutable_inherited_bytes_and_only_authorized_production_delta() {
    let r = fixture::root();
    let base = r.parent().unwrap().join("testland_m2_production");
    assert_eq!(inherited_pack_audit(&base, &r, native_path), Ok(()));
}
#[test]
fn linux_and_windows_relative_keys_allow_only_exact_authorized_paths() {
    for key in [
        "manifest.toml",
        "README.md",
        "SOURCES.md",
        "common/production/initial.toml",
        "localisation/en/pack.ftl",
        "localisation/ko/pack.ftl",
        "localisation/en/production.ftl",
        "localisation/ko/production.ftl",
        "scenarios/m2_production/scenario.toml",
    ] {
        for representation in [key.to_owned(), key.replace('/', "\\")] {
            let normalized = relative_key(&representation).unwrap();
            assert_eq!(normalized, key);
            assert!(has_authorized_delta(&normalized), "{representation}");
        }
    }
    for key in [
        "other/manifest.toml",
        "common/production/other.toml",
        "common/economy/initial.toml",
        "localisation/en/nested/production.ftl",
        "localisation/fr/production.ftl",
        "scenarios/m2_production/nations/N01.toml",
        "scenarios/m2_production/defines.toml",
        "maps/testland_m2/provinces.png",
    ] {
        for representation in [key.to_owned(), key.replace('/', "\\")] {
            let normalized = relative_key(&representation).unwrap();
            assert_eq!(normalized, key);
            assert!(!has_authorized_delta(&normalized), "{representation}");
        }
    }
    for key in [
        "/manifest.toml",
        "\\manifest.toml",
        "./manifest.toml",
        "..\\manifest.toml",
        "C:\\manifest.toml",
        "common//production/initial.toml",
        "common\\\\production\\initial.toml",
    ] {
        assert!(relative_key(key).is_err(), "{key}");
    }
    for key in [
        "scenarios/m2_production/nations/N01.toml",
        "scenarios/m2_production/defines.toml",
    ] {
        assert_eq!(
            mapped_relative(&relative_key(&key.replace('/', "\\")).unwrap()),
            key.replace("scenarios/m2_production/", "scenarios/m2_military/")
        );
    }
    assert_eq!(
        mapped_relative("other/scenarios/m2_production/defines.toml"),
        "other/scenarios/m2_production/defines.toml"
    );
}
#[test]
fn both_path_representations_still_reject_unauthorized_actual_content_changes() {
    let base = fixture::root()
        .parent()
        .unwrap()
        .join("testland_m2_production");
    for formatter in [linux_path as fn(&std::path::Path) -> String, windows_path] {
        for file in [
            "common/economy/initial.toml",
            "maps/testland_m2/provinces.png",
            "common/production/initial.toml",
            "localisation/en/production.ftl",
            "scenarios/m2_military/scenario.toml",
        ] {
            let candidate = fixture::CopyPack::new();
            assert_eq!(
                inherited_pack_audit(&base, candidate.root(), formatter),
                Ok(())
            );
            let path = candidate.root().join(file);
            let mut bytes = std::fs::read(&path).unwrap();
            bytes.push(b'!');
            std::fs::write(path, bytes).unwrap();
            assert_eq!(
                inherited_pack_audit(&base, candidate.root(), formatter),
                Err(file.into()),
                "{file}"
            );
        }
    }
}
