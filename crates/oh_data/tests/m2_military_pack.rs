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
#[test]
fn immutable_inherited_bytes_and_only_authorized_production_delta() {
    let r = fixture::root();
    let base = r.parent().unwrap().join("testland_m2_production");
    fn files(p: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut v = Vec::new();
        for e in std::fs::read_dir(p).unwrap() {
            let e = e.unwrap().path();
            if e.is_dir() {
                v.extend(files(&e))
            } else {
                v.push(e)
            }
        }
        v
    }
    assert_eq!(files(&r).len(), 32);
    for p in files(&base) {
        let relative = p.strip_prefix(&base).unwrap().to_str().unwrap();
        if matches!(
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
        ) {
            continue;
        }
        let mapped = relative.replace("scenarios/m2_production/", "scenarios/m2_military/");
        assert_eq!(
            std::fs::read(&p).unwrap(),
            std::fs::read(r.join(mapped)).unwrap(),
            "{relative}"
        );
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
        assert_eq!(std::fs::read_to_string(r.join(&path)).unwrap(), expected);
    }
    let scenario = std::fs::read_to_string(base.join("scenarios/m2_production/scenario.toml"))
        .unwrap()
        .replace(
            "# Synthetic economy and production; no military or victory rules.",
            "# Synthetic military fixture; no combat or victory rules.",
        );
    assert_eq!(
        std::fs::read_to_string(r.join("scenarios/m2_military/scenario.toml")).unwrap(),
        format!("military = \"initial\"\n{scenario}")
    );
    let mut expected =
        std::fs::read_to_string(base.join("common/production/initial.toml")).unwrap();
    for n in [3, 6] {
        expected=expected.replace(&format!("[nations.{n}]\nallowed_models = [\"m2_equipment_2\"]\nstock = {{ m2_equipment_2 = 0 }}"),&format!("[nations.{n}]\nallowed_models = [\"m2_equipment_1\", \"m2_equipment_2\"]\nstock = {{ m2_equipment_1 = 0, m2_equipment_2 = 0 }}"));
    }
    assert_eq!(
        std::fs::read_to_string(r.join("common/production/initial.toml")).unwrap(),
        format!(
            "# TEST FIXTURE DESIGN: common model1 permits the existing all-nation template loader.\n{expected}"
        )
    );
}
