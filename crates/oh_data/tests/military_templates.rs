const GOOD: &str = include_str!("fixtures/military_templates/normal.toml");
use oh_data::military_templates::{MAX_BYTES, parse};
#[path = "support/production.rs"]
mod production;
#[test]
fn strict_definitions_have_valid_control() {
    let d = parse(GOOD).unwrap();
    assert_eq!(d.components().len(), 2);
    assert_eq!(d.templates()["example"].combat, ["line"]);
}
#[test]
fn missing_named_inputs_unknown_fields_and_numeric_domains() {
    for line in [
        "strength = \"25\"",
        "soft_fire = \"3\"",
        "hard_fire = \"0.5\"",
        "defense = \"18\"",
        "breakthrough = \"2\"",
        "frontage = \"2\"",
        "supply_use = \"0.06\"",
        "organization = \"30\"",
        "armor = \"2\"",
        "piercing = \"4\"",
        "speed_kmh = \"4\"",
    ] {
        assert!(
            parse(&GOOD.replace(&format!("{line}\n"), "")).is_err(),
            "{line}"
        );
    }
    for value in [
        "-1",
        "+1",
        "1e3",
        "NaN",
        "",
        "1.",
        ".5",
        "0x10",
        "99999999999999999999999999999999999",
    ] {
        assert!(
            parse(&GOOD.replace("strength = \"25\"", &format!("strength = \"{value}\""))).is_err(),
            "{value}"
        );
    }
    for change in [
        GOOD.replacen("version = 1", "version = 2", 1),
        GOOD.replacen("version = 1", "version = 1\nextra = 2", 1),
        GOOD.replace("manpower = 1000", "manpower = -1"),
        GOOD.replace("items = 100", "items = -1"),
        GOOD.replace("speed_kmh = \"4\"", "speed_kmh = \"0\""),
        GOOD.replace("speed_kmh = \"4\"", "speed_kmh = \"0.0000000000000001\""),
        GOOD.replace("strength = \"25\"", "strength = 25"),
        GOOD.replace("manpower = 1000", "manpower = 1.5"),
    ] {
        assert!(parse(&change).is_err());
    }
    assert!(parse(&" ".repeat(MAX_BYTES + 1)).is_err());
    assert!(parse(&GOOD.replace("strength = \"25\"", "strength = \"0\"")).is_ok());
}
#[test]
fn raw_duplicates_role_and_binding_context_are_rejected() {
    let component = GOOD.split("[[components]]").nth(1).unwrap();
    let template = GOOD.split("[[templates]]").nth(1).unwrap();
    for text in [format!("{GOOD}\n[[components]]{component}"),format!("{GOOD}\n[[templates]]{template}"),GOOD.replace("items = 100 }]","items = 100 }, { family = \"test_family\", model = \"test_model_1\", items = 1 }]"),GOOD.replace("bindings = [{ family = \"test_family\", model = \"test_model_1\" }]", "bindings = [{ family = \"test_family\", model = \"test_model_1\" }, { family = \"test_family\", model = \"test_model_1\" }]"),GOOD.replace("combat = [\"line\"]", "combat = [\"support\"]"),GOOD.replace("combat = [\"line\"]", "combat = [\"missing\"]"),GOOD.replace("bindings = [{ family = \"test_family\", model = \"test_model_1\" }]", "bindings = [{ family = \"test_family\", model = \"test_model_2\" }]")] {
        assert!(parse(&text).is_err(), "{text}");
    }
}
#[test]
fn approved_maxima_preserve_occurrences_without_editor_policy() {
    for (field, name, max) in [("combat", "line", 12), ("support", "support", 4)] {
        let existing = format!("{field} = [\"{name}\"]");
        let repeated = |n| format!("{field} = [{}]", vec![format!("\"{name}\""); n].join(","));
        let d = parse(&GOOD.replace(&existing, &repeated(max))).unwrap();
        assert_eq!(
            if field == "combat" {
                d.templates()["example"].combat.len()
            } else {
                d.templates()["example"].support.len()
            },
            max
        );
        assert!(parse(&GOOD.replace(&existing, &repeated(max + 1))).is_err());
        assert!(parse(&GOOD.replace(&existing, &repeated(0))).is_ok());
    }
    assert!(
        parse(
            &GOOD
                .replace("manpower = 1000", "manpower = 0")
                .replace("manpower = 100\n", "manpower = 0\n")
        )
        .is_ok()
    );
}
#[test]
fn normalized_identity_and_actual_production_bindings() {
    let d = parse(GOOD).unwrap();
    assert_eq!(
        d.identity().unwrap(),
        parse(&GOOD.replace("\"25\"", "\"25.0\""))
            .unwrap()
            .identity()
            .unwrap()
    );
    let pack = production::pack();
    let mut loaded = oh_data::national::load_scenario(&pack, "m1").unwrap();
    assert!(d.validate_bindings(&loaded, 1, "example").is_ok());
    assert!(d.validate_bindings(&loaded, 2, "example").is_ok());
    assert!(d.validate_bindings(&loaded, 3, "example").is_err());
    let p = loaded.production.as_mut().unwrap();
    p.models.get_mut("test_model_1").unwrap().family = "other".into();
    assert!(
        d.validate_bindings(&loaded, 1, "example")
            .unwrap_err()
            .contains("family")
    );
    loaded
        .production
        .as_mut()
        .unwrap()
        .models
        .get_mut("test_model_1")
        .unwrap()
        .family = "test_family".into();
    loaded
        .production
        .as_mut()
        .unwrap()
        .nations
        .get_mut(&1)
        .unwrap()
        .allowed_models
        .remove("test_model_1");
    assert!(
        d.validate_bindings(&loaded, 1, "example")
            .unwrap_err()
            .contains("DisallowedModel")
    );
    loaded.production = None;
    assert!(
        d.validate_bindings(&loaded, 1, "example")
            .unwrap_err()
            .contains("MissingContext")
    );
    std::fs::remove_dir_all(pack.parent().unwrap()).unwrap();
}
