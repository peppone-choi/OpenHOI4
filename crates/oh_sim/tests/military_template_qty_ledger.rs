use oh_core::Qty;
use oh_sim::military_templates::{aggregate, aggregate_with_ledger};

const GOOD: &str = include_str!("../../oh_data/tests/fixtures/military_templates/normal.toml");
const FIELDS: [&str; 7] = [
    "strength",
    "soft_fire",
    "hard_fire",
    "defense",
    "breakthrough",
    "frontage",
    "supply_use",
];

#[test]
fn actual_qty_contributions_match_all_seven_normal_fields_without_narrowing() {
    let d = oh_data::military_templates::parse(GOOD).unwrap();
    let identity = d.identity().unwrap();
    let (normal, ledger) = aggregate_with_ledger(&d, "example").unwrap();
    let values = [
        normal.stats.strength,
        normal.stats.soft_fire,
        normal.stats.hard_fire,
        normal.stats.defense,
        normal.stats.breakthrough,
        normal.stats.frontage,
        normal.stats.supply_use,
    ];
    assert_eq!(ledger.fields.len(), 7);
    for ((field, expected_id), expected) in ledger.fields.iter().zip(FIELDS).zip(values) {
        assert_eq!(field.field, expected_id);
        assert_eq!(field.base, Qty::ZERO);
        assert_eq!(field.value.to_bits(), expected.to_bits());
        assert_eq!(field.entries.len(), 2);
        let mut raw = 0i64;
        for (index, row) in field.entries.iter().enumerate() {
            let component = if index == 0 { "line" } else { "support" };
            assert_eq!(row.component, component);
            assert_eq!(row.role, if index == 0 { "combat" } else { "support" });
            assert_eq!(row.position, 0);
            assert_eq!(row.id, format!("{}:{}:0", field.field, row.role));
            let stats = &d.components()[component].stats;
            let source = match field.field {
                "strength" => stats.strength,
                "soft_fire" => stats.soft_fire,
                "hard_fire" => stats.hard_fire,
                "defense" => stats.defense,
                "breakthrough" => stats.breakthrough,
                "frontage" => stats.frontage,
                "supply_use" => stats.supply_use,
                _ => unreachable!(),
            };
            assert_eq!(row.value.to_bits(), source.to_bits());
            raw = raw.checked_add(source.to_bits()).unwrap();
            assert_eq!(row.accumulated.to_bits(), raw);
        }
        assert_eq!(
            field.entries.last().unwrap().accumulated.to_bits(),
            expected.to_bits()
        );
    }
    assert_eq!(normal, aggregate(&d, "example").unwrap());
    assert_eq!(identity, d.identity().unwrap());
    // Normal still has exactly its historical four fields. Diagnostic records cannot
    // enter the stored division/job Normal or its serialized hash/save representation.
    let encoded = serde_json::to_value(&normal).unwrap();
    assert_eq!(
        encoded
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ["equipment", "manpower", "stats", "weighted_remainders"]
    );
    assert_eq!(
        serde_json::from_value::<oh_sim::military_templates::Normal>(encoded).unwrap(),
        normal
    );
}

#[test]
fn repeated_sorted_occurrences_keep_separate_ids_and_support_follows_combat() {
    let input = GOOD.replace("combat = [\"line\"]", "combat = [\"line\", \"line\"]");
    let d = oh_data::military_templates::parse(&input).unwrap();
    let (normal, ledger) = aggregate_with_ledger(&d, "example").unwrap();
    assert_eq!(normal.stats.strength, Qty::from_num(55));
    for field in ledger.fields {
        assert_eq!(
            field
                .entries
                .iter()
                .map(|e| (e.role, e.position, e.component.as_str()))
                .collect::<Vec<_>>(),
            [
                ("combat", 0, "line"),
                ("combat", 1, "line"),
                ("support", 0, "support")
            ]
        );
        assert_eq!(
            field
                .entries
                .iter()
                .map(|e| e.id.clone())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3
        );
    }
}

#[test]
fn large_qty_values_and_raw_fractional_bits_are_preserved() {
    let input = GOOD.replace("strength = \"25\"", "strength = \"1000000000000\"");
    let d = oh_data::military_templates::parse(&input).unwrap();
    let (_, ledger) = aggregate_with_ledger(&d, "example").unwrap();
    let strength = &ledger.fields[0];
    assert!(strength.entries[0].value.to_bits() > 9_007_199_254_740_991);
    assert_eq!(strength.value, Qty::from_num(1000000000005i64));
    let supply = &ledger.fields[6];
    assert_eq!(
        supply.value.to_bits(),
        "0.06".parse::<Qty>().unwrap().to_bits() + "0.02".parse::<Qty>().unwrap().to_bits()
    );
}

#[test]
fn exact_historical_errors_and_failure_precedence_remain_atomic() {
    let cases = [
        (GOOD.to_string(), "missing", "InvalidReference: template ID"),
        (
            GOOD.replace("manpower = 1000", "manpower = 9223372036854775807"),
            "example",
            "Overflow: manpower",
        ),
        (
            GOOD.replace("combat = [\"line\"]", "combat = []")
                .replace("support = [\"support\"]", "support = []"),
            "example",
            "UndefinedArithmetic: empty composition has no minimum speed",
        ),
        (
            GOOD.replace("manpower = 1000", "manpower = 0")
                .replace("manpower = 100\n", "manpower = 0\n"),
            "example",
            "UndefinedArithmetic: zero personnel weight",
        ),
        (
            GOOD.replace("items = 100", "items = 9223372036854775807")
                .replace("strength = \"25\"", "strength = \"140737488355327\""),
            "example",
            "Overflow: equipment count",
        ),
        (
            GOOD.replace("strength = \"25\"", "strength = \"140737488355327\""),
            "example",
            "Overflow: Qty sum",
        ),
    ];
    for (input, template, expected) in cases {
        let d = oh_data::military_templates::parse(&input).unwrap();
        let identity = d.identity().unwrap();
        assert_eq!(aggregate_with_ledger(&d, template).unwrap_err(), expected);
        assert_eq!(aggregate(&d, template).unwrap_err(), expected);
        assert_eq!(identity, d.identity().unwrap());
    }
}

#[test]
fn maximum_composition_is_bounded_and_deterministic() {
    let input = GOOD
        .replace(
            "combat = [\"line\"]",
            &format!("combat = [{}]", ["\"line\""; 12].join(",")),
        )
        .replace(
            "support = [\"support\"]",
            &format!("support = [{}]", ["\"support\""; 4].join(",")),
        );
    let d = oh_data::military_templates::parse(&input).unwrap();
    let first = aggregate_with_ledger(&d, "example").unwrap();
    assert_eq!(first, aggregate_with_ledger(&d, "example").unwrap());
    for f in first.1.fields {
        assert_eq!(f.entries.len(), 16);
        assert_eq!(f.entries[11].position, 11);
        assert_eq!((f.entries[12].role, f.entries[12].position), ("support", 0));
        assert_eq!(f.entries.last().unwrap().accumulated, f.value);
    }
}

#[test]
fn positions_follow_loaded_sorting_instead_of_authored_order() {
    let duplicate = GOOD
        .split("[[components]]")
        .nth(1)
        .unwrap()
        .replace("id = \"line\"", "id = \"a_line\"");
    let input = GOOD
        .replace(
            "[[templates]]",
            &format!("[[components]]{duplicate}[[templates]]"),
        )
        .replace(
            "combat = [\"line\"]",
            "combat = [\"line\", \"a_line\", \"line\"]",
        );
    let d = oh_data::military_templates::parse(&input).unwrap();
    let (_, ledger) = aggregate_with_ledger(&d, "example").unwrap();
    for f in ledger.fields {
        assert_eq!(
            f.entries
                .iter()
                .map(|e| (e.component.as_str(), e.position))
                .collect::<Vec<_>>(),
            [("a_line", 0), ("line", 1), ("line", 2), ("support", 0)]
        );
    }
}

#[test]
fn valid_registry_limit_can_contain_4096_maximum_compositions() {
    let stats = "strength = \"1\"\nsoft_fire = \"1\"\nhard_fire = \"1\"\ndefense = \"1\"\nbreakthrough = \"1\"\nfrontage = \"1\"\nsupply_use = \"1\"\norganization = \"1\"\narmor = \"1\"\npiercing = \"1\"\nspeed_kmh = \"1\"\n";
    let mut input = String::from("version = 1\n");
    for (id, role) in [("a", "combat"), ("b", "support")] {
        input.push_str(&format!("[[components]]\nid = \"{id}\"\nrole = \"{role}\"\nmanpower = 1\nequipment = []\n[components.stats]\n{stats}"));
    }
    for n in 0..4096 {
        input.push_str(&format!(
            "[[templates]]\nid = \"t{n:04}\"\ncombat = [{}]\nsupport = [{}]\nbindings = []\n",
            ["\"a\""; 12].join(","),
            ["\"b\""; 4].join(",")
        ));
    }
    assert!(input.len() < oh_data::military_templates::MAX_BYTES);
    let d = oh_data::military_templates::parse(&input).unwrap();
    assert_eq!(
        d.templates().len(),
        oh_data::military_templates::MAX_ENTRIES
    );
    let (_, ledger) = aggregate_with_ledger(&d, "t4095").unwrap();
    assert!(
        ledger
            .fields
            .iter()
            .all(|f| f.value == Qty::from_num(16) && f.entries.len() == 16)
    );
}
