use oh_data::supply_network::parse;
const GOOD: &str = include_str!("fixtures/supply_network/capital.toml");
#[test]
fn explicit_capital_config_loads_real_refs() {
    let c = parse(GOOD).expect("explicit valid config");
    let l = oh_data::national::load_scenario(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap();
    c.validate_references(&l).unwrap();
}
#[test]
fn normalized_decimal_identity_is_diagnostic_only() {
    assert_eq!(
        parse(GOOD).unwrap().identity().unwrap(),
        parse(&GOOD.replace("\"4\"", "\"4.0\""))
            .unwrap()
            .identity()
            .unwrap()
    );
}
#[test]
fn strict_unknown_negative_and_duplicate_level_rejected_with_valid_control() {
    assert!(parse(GOOD).is_ok());
    for bad in [
        GOOD.replace("version = 1", "version = 1\nunknown = true"),
        GOOD.replace("\"4\"", "\"-4\""),
        GOOD.replace("level = \"1\"", "level = \"0.0\""),
    ] {
        assert!(parse(&bad).is_err());
    }
}
fn loaded() -> oh_data::national::LoadedNational {
    oh_data::national::load_scenario(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap()
}
#[test]
fn strict_nested_missing_null_and_numeric_controls() {
    assert!(parse(GOOD).is_ok());
    let cases = [
        GOOD.replace("version = 1", "version = 2"),
        GOOD.replace("reference_speed_kmh = \"4\"\n", ""),
        GOOD.replace("reference_speed_kmh = \"4\"", "reference_speed_kmh = null"),
        GOOD.replace("reference_speed_kmh = \"4\"", "reference_speed_kmh = 4"),
        GOOD.replace("reference_speed_kmh = \"4\"", "reference_speed_kmh = \"0\""),
        GOOD.replace(
            "reference_speed_kmh = \"4\"",
            "reference_speed_kmh = \"2147483648\"",
        ),
        GOOD.replace(
            "reference_speed_kmh = \"4\"",
            "reference_speed_kmh = \"0.0000000000000000001\"",
        ),
        GOOD.replace("\"0.0078125\"", "\"-0.0078125\""),
        GOOD.replace("factor = \"1.5\"", "factor = \"0\""),
        GOOD.replace("capacity = \"20\"", "capacity = \"0\""),
        GOOD.replace("capacity = \"20\"", "capacity = \"140737488355328\""),
        GOOD.replace("capacity = \"20\"", "capacity = \"NaN\""),
        GOOD.replace("capacity = \"20\"", "capacity = \"2e1\""),
        GOOD.replace("capacity = \"20\"", "capacity = \"+20\""),
        GOOD.replace("capacity = \"20\"", "capacity = \" 20\""),
        GOOD.replace("capacity = \"20\"", "capacity = \"20\", bad = 0"),
        GOOD.replace("a = 10, b = 20", "a = 20, b = 10"),
        GOOD.replace("level = 2 }", "level = 3 }"),
        GOOD.replace("kind = \"capital\"", "kind = \"Capital\""),
        GOOD.replace("province = 10", "province = 65536"),
    ];
    for (i, bad) in cases.iter().enumerate() {
        assert!(parse(bad).is_err(), "case {i}");
    }
    // Qty has a wider integer range than Fx. Never narrow vehicle/capacity quantities.
    assert!(parse(&GOOD.replace("capacity = \"20\"", "capacity = \"2147483648\"")).is_ok());
    assert!(parse(&GOOD.replace("\"0.0078125\"", "\"0\"")).is_ok());
}
#[test]
fn duplicates_rejected_before_lossy_maps_including_quantized_level_collisions() {
    assert!(parse(GOOD).is_ok());
    let source = "{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }";
    let rail = "{ a = 10, b = 20, level = 2 }";
    for bad in [
        GOOD.replace(source, &format!("{source}, {source}")),
        GOOD.replace(
            source,
            &format!("{source}, {}", source.replace("id = 0", "id = 1")),
        ),
        GOOD.replace(rail, &format!("{rail}, {rail}")),
        GOOD.replace(
            "level = 2, capacity = \"20\"",
            "level = 1, capacity = \"20\"",
        ),
        GOOD.replace("level = \"1\"", "level = \"0.0000000000000000001\""),
        format!("{GOOD}\n[[nations]]\nid=1\nsources=[]\nrails=[]\n"),
    ] {
        assert!(parse(&bad).unwrap_err().contains("Duplicate"));
    }
}
#[test]
fn reference_capital_land_state_coast_edge_and_nation_controls() {
    let c = parse(GOOD).unwrap();
    let l = loaded();
    c.validate_references(&l).unwrap();
    for bad in [
        GOOD.replace("id = 1\nsources", "id = 99\nsources"),
        GOOD.replace("province = 10", "province = 20"),
        GOOD.replace("b = 20", "b = 999"),
    ] {
        assert!(parse(&bad).unwrap().validate_references(&l).is_err());
    }
    let mut l = loaded();
    l.map.states[0].provinces.clear();
    assert!(c.validate_references(&l).is_err());
    let mut l = loaded();
    l.map
        .provinces
        .iter_mut()
        .find(|p| p.id == 10)
        .unwrap()
        .kind = oh_data::map::ProvinceKind::Sea;
    assert!(c.validate_references(&l).is_err());
    for kind in [
        oh_data::map::EdgeKind::Strait,
        oh_data::map::EdgeKind::Impassable,
    ] {
        let mut l = loaded();
        l.map
            .edges
            .iter_mut()
            .find(|e| e.a == 10 && e.b == 20)
            .unwrap()
            .kind = kind;
        assert!(c.validate_references(&l).is_err());
    }
    let port = GOOD.replace(
        "kind = \"capital\", province = 10",
        "kind = \"port\", province = 20",
    );
    let c = parse(&port).unwrap();
    let mut l = loaded();
    l.map
        .provinces
        .iter_mut()
        .find(|p| p.id == 20)
        .unwrap()
        .coastal = false;
    assert!(c.validate_references(&l).is_err());
    l.map
        .provinces
        .iter_mut()
        .find(|p| p.id == 20)
        .unwrap()
        .coastal = true;
    c.validate_references(&l).unwrap(); // Still no instance authority; sim adapter rejects separately.
}
#[test]
fn explicit_empty_metadata_has_distinct_identity_and_no_implicit_source() {
    let empty = GOOD
        .replace(
            "[{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }]",
            "[]",
        )
        .replace("[{ a = 10, b = 20, level = 2 }]", "[]");
    let c = parse(&empty).unwrap();
    c.validate_references(&loaded()).unwrap();
    assert!(c.nations()[&1].sources.is_empty());
    assert!(c.nations()[&1].rails.is_empty());
    assert_ne!(
        c.identity().unwrap(),
        parse(GOOD).unwrap().identity().unwrap()
    );
    assert!(parse("").is_err());
    assert!(
        parse(&empty.replace(
            "[{ level = \"0\", factor = \"2\" }, { level = \"1\", factor = \"1.5\" }]",
            "[]"
        ))
        .is_err()
    );
}
#[test]
fn canonical_order_and_typed_decimal_identity() {
    let two = format!("{GOOD}\n[[nations]]\nid=2\nsources=[]\nrails=[]\n");
    let alternate = two
        .replace(
            "{ level = \"0\", factor = \"2\" }, { level = \"1\", factor = \"1.5\" }",
            "{ level = \"1.0\", factor = \"1.50\" }, { level = \"0.0\", factor = \"2.0\" }",
        )
        .replace(
            "{ level = 1, capacity = \"10\" }, { level = 2, capacity = \"20\" }",
            "{ level = 2, capacity = \"20.0\" }, { level = 1, capacity = \"10.0\" }",
        );
    assert_ne!(two.as_bytes(), alternate.as_bytes());
    assert_eq!(
        parse(&two).unwrap().identity(),
        parse(&alternate).unwrap().identity()
    );
    assert_ne!(
        parse(GOOD).unwrap().identity(),
        parse(&GOOD.replace("\"1.5\"", "\"1.75\""))
            .unwrap()
            .identity()
    );
}
#[test]
fn bounded_document_tables_strings_and_file_reads() {
    use oh_data::supply_network::{MAX_BYTES, MAX_ENTRIES, MAX_NATIONS, read_file};
    assert!(
        parse(&" ".repeat(MAX_BYTES + 1))
            .unwrap_err()
            .contains("LimitExceeded")
    );
    assert!(parse(&GOOD.replace("\"4\"", &format!("\"{}\"", "0".repeat(65)))).is_err());
    let many = (0..=MAX_ENTRIES)
        .map(|i| format!("{{level=\"{i}\",factor=\"1\"}}"))
        .collect::<Vec<_>>()
        .join(",");
    let p = GOOD.replace(
        "[{ level = \"0\", factor = \"2\" }, { level = \"1\", factor = \"1.5\" }]",
        &format!("[{many}]"),
    );
    assert!(parse(&p).unwrap_err().contains("LimitExceeded"));
    let many = (2..=MAX_NATIONS + 1)
        .map(|i| format!("\n[[nations]]\nid={i}\nsources=[]\nrails=[]\n"))
        .collect::<String>();
    assert!(
        parse(&format!("{GOOD}{many}"))
            .unwrap_err()
            .contains("LimitExceeded")
    );
    let path = std::env::temp_dir().join(format!("oh-supply-data-{}", std::process::id()));
    std::fs::write(&path, GOOD).unwrap();
    assert!(read_file(&path).is_ok());
    std::fs::write(&path, vec![b' '; MAX_BYTES + 1]).unwrap();
    assert!(read_file(&path).is_err());
    std::fs::write(&path, [255u8]).unwrap();
    assert!(read_file(&path).is_err());
    std::fs::remove_file(&path).unwrap();
    assert!(read_file(&std::env::temp_dir()).is_err());
}
#[test]
fn per_nation_and_total_metadata_bounds_are_checked_before_normalization() {
    use oh_data::supply_network::MAX_ENTRIES;
    let source = "{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }";
    let too_many = std::iter::repeat_n(source, MAX_ENTRIES + 1)
        .collect::<Vec<_>>()
        .join(",");
    let text = GOOD.replace(source, &too_many);
    assert!(parse(&text).unwrap_err().contains("LimitExceeded: sources"));
    let head = GOOD.split("[[nations]]").next().unwrap();
    let rails = (0..MAX_ENTRIES)
        .map(|i| format!("{{a={i},b={},level=1}}", i + 1))
        .collect::<Vec<_>>()
        .join(",");
    let text = format!(
        "{head}{}",
        (1..=5)
            .map(|id| format!("\n[[nations]]\nid={id}\nsources=[]\nrails=[{rails}]\n"))
            .collect::<String>()
    );
    assert!(text.len() < oh_data::supply_network::MAX_BYTES);
    assert!(
        parse(&text)
            .unwrap_err()
            .contains("LimitExceeded: total metadata")
    );
}
#[test]
fn source_and_rail_order_identity_and_nation_scoped_source_ids() {
    let text=format!("{}\n[[nations]]\nid=2\nsources=[{{id=0,kind=\"capital\",province=30,capacity=\"8\"}}]\nrails=[]\n",GOOD.replace("{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }","{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }, { id = 2, kind = \"hub\", province = 20, capacity = \"10\" }").replace("{ a = 10, b = 20, level = 2 }","{ a = 10, b = 20, level = 2 }, { a = 10, b = 30, level = 1 }"));
    let c = parse(&text).unwrap();
    c.validate_references(&loaded()).unwrap();
    let alternate=text.replace("{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }, { id = 2, kind = \"hub\", province = 20, capacity = \"10\" }","{ id = 2, kind = \"hub\", province = 20, capacity = \"10\" }, { id = 0, kind = \"capital\", province = 10, capacity = \"20\" }").replace("{ a = 10, b = 20, level = 2 }, { a = 10, b = 30, level = 1 }","{ a = 10, b = 30, level = 1 }, { a = 10, b = 20, level = 2 }");
    assert_eq!(
        c.identity().unwrap(),
        parse(&alternate).unwrap().identity().unwrap()
    );
}
