const GOOD: &str = include_str!("../../oh_data/tests/fixtures/military_templates/normal.toml");
#[test]
fn normal_sum_weighted_min_and_counts() {
    let d = oh_data::military_templates::parse(GOOD).unwrap();
    let n = oh_sim::military_templates::aggregate(&d, "example").unwrap();
    assert_eq!(n.manpower, 1100);
    assert_eq!(n.stats.strength, oh_core::Qty::from_num(30));
    assert_eq!(
        n.stats.organization.to_bits(),
        (30i64 * 65536 * 1000 + 60i64 * 65536 * 100) / 1100
    );
    assert_eq!(n.stats.speed_kmh, oh_core::Fx::from_num(2));
    assert_eq!(n.equipment["test_model_1"], 110);
}
#[test]
fn all_sum_fields_fractional_precision_and_weighted_remainder() {
    let d = oh_data::military_templates::parse(GOOD).unwrap();
    let before = d.identity().unwrap();
    let n = oh_sim::military_templates::aggregate(&d, "example").unwrap();
    let q = |v: &str| v.parse::<oh_core::Qty>().unwrap();
    for (actual, expected) in [
        (n.stats.soft_fire, q("4")),
        (n.stats.hard_fire, q("1")),
        (n.stats.defense, q("20")),
        (n.stats.breakthrough, q("3")),
        (n.stats.frontage, q("2")),
        (n.stats.supply_use, q("0.06") + q("0.02")),
    ] {
        assert_eq!(actual, expected);
    }
    assert_eq!(
        n.stats.armor.to_bits(),
        (2 * 65536 * 1000 + 8 * 65536 * 100) / 1100
    );
    assert_eq!(
        n.stats.piercing.to_bits(),
        (4 * 65536 * 1000 + 10 * 65536 * 100) / 1100
    );
    assert_eq!(
        n.weighted_remainders["organization"],
        (30 * 65536 * 1000 + 60 * 65536 * 100) % 1100
    );
    assert_eq!(before, d.identity().unwrap());
}
#[test]
fn multiplicity_zero_personnel_and_empty_math_are_distinct_from_editor_rules() {
    let text = GOOD.replace("combat = [\"line\"]", "combat = [\"line\", \"line\"]");
    let d = oh_data::military_templates::parse(&text).unwrap();
    let n = oh_sim::military_templates::aggregate(&d, "example").unwrap();
    assert_eq!(n.manpower, 2100);
    assert_eq!(n.equipment["test_model_1"], 210);
    assert_eq!(n.stats.strength, oh_core::Qty::from_num(55));
    let text = GOOD.replace("manpower = 100\n", "manpower = 0\n");
    let d = oh_data::military_templates::parse(&text).unwrap();
    let n = oh_sim::military_templates::aggregate(&d, "example").unwrap();
    assert_eq!(n.stats.organization, oh_core::Qty::from_num(30));
    assert_eq!(n.stats.speed_kmh, oh_core::Fx::from_num(2));
    for text in [
        GOOD.replace("combat = [\"line\"]", "combat = []")
            .replace("support = [\"support\"]", "support = []"),
        GOOD.replace("manpower = 1000", "manpower = 0")
            .replace("manpower = 100\n", "manpower = 0\n"),
    ] {
        let d = oh_data::military_templates::parse(&text).unwrap();
        let before = d.identity().unwrap();
        assert!(
            oh_sim::military_templates::aggregate(&d, "example")
                .unwrap_err()
                .contains("UndefinedArithmetic")
        );
        assert_eq!(before, d.identity().unwrap());
    }
}
#[test]
fn wide_weighted_intermediate_and_checked_overflow() {
    let text = GOOD
        .replace("manpower = 1000", "manpower = 100000000000000")
        .replace("manpower = 100\n", "manpower = 100000000000000\n")
        .replace("organization = \"30\"", "organization = \"100000000000\"")
        .replace("organization = \"60\"", "organization = \"100000000002\"");
    let d = oh_data::military_templates::parse(&text).unwrap();
    let n = oh_sim::military_templates::aggregate(&d, "example").unwrap();
    assert_eq!(
        n.stats.organization,
        "100000000001".parse::<oh_core::Qty>().unwrap()
    );
    for text in [
        GOOD.replace("manpower = 1000", "manpower = 9223372036854775807"),
        GOOD.replace("items = 100", "items = 9223372036854775807"),
        GOOD.replace("strength = \"25\"", "strength = \"140737488355327\""),
    ] {
        let text = if text.contains("strength = \"140737488355327\"") {
            text.replace("combat = [\"line\"]", "combat = [\"line\", \"line\"]")
        } else {
            text
        };
        let d = oh_data::military_templates::parse(&text).unwrap();
        assert!(
            oh_sim::military_templates::aggregate(&d, "example")
                .unwrap_err()
                .contains("Overflow")
        );
    }
}
