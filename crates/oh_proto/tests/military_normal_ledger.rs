#[path = "../../oh_save/tests/support/military.rs"]
mod fixture;
#[path = "support/military_ledger.rs"]
mod ledger_fixture;

#[test]
fn selected_template_authority_is_bound_to_base_bits_and_does_not_mutate_simulation() {
    let root = fixture::pack();
    let loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    let sim = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap();
    let before = sim.state_hash().unwrap();
    let ledger = oh_proto::MilitaryNormalLedgerView::from_sim(&sim, "example").unwrap();
    let base = oh_proto::MilitaryView::from_sim(&sim).unwrap();
    assert_eq!(ledger.definitions_hash, base.definitions_hash);
    assert_eq!(ledger.state_hash, base.state_hash);
    assert_eq!(ledger.tick, "0");
    assert_eq!(ledger.template, "example");
    assert_eq!(ledger.fields.len(), 7);
    let n = &base.templates[0].normal;
    let values = [
        &n.strength,
        &n.soft_fire,
        &n.hard_fire,
        &n.defense,
        &n.breakthrough,
        &n.frontage,
        &n.supply_use,
    ];
    for (field, value) in ledger.fields.iter().zip(values) {
        assert_eq!(&field.value, value);
        assert_eq!(field.base.bits, "0");
        assert_eq!(field.entries.last().unwrap().accumulated, *value);
        for entry in &field.entries {
            assert_eq!(entry.value.fractional_bits, 16);
            assert_eq!(
                entry.id,
                format!("{}:{}:{}", field.field, entry.role, entry.position)
            );
        }
    }
    assert_eq!(sim.state_hash().unwrap(), before);
    assert_eq!(
        oh_proto::MilitaryNormalLedgerView::from_sim(&sim, "missing"),
        Err("unknown-template")
    );
    let packet = oh_proto::ServerMessage::MilitaryNormalLedgerResult {
        request: "military-ledger:18446744073709551615".into(),
        supported: true,
        reason_key: None,
        ledger: Some(ledger),
    };
    let bytes = oh_proto::encode(&packet).unwrap();
    assert!(bytes.len() < 65536);
    assert_eq!(oh_proto::decode_server(&bytes).unwrap(), packet);
    let old = oh_proto::ServerMessage::MilitaryResult {
        request: "military:1".into(),
        supported: true,
        reason_key: None,
        military: Some(base),
    };
    let json = serde_json::to_value(&old).unwrap();
    assert_eq!(json["military"].as_object().unwrap().len(), 10);
    assert_eq!(
        json["military"]["templates"][0].as_object().unwrap().len(),
        3
    );
}

#[test]
fn maximum_occurrences_long_ids_and_large_raw_qty_fit_selected_response() {
    let (root, template) = ledger_fixture::maximum_pack(fixture::pack());
    let loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    let sim = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap();
    let ledger = oh_proto::MilitaryNormalLedgerView::from_sim(&sim, &template).unwrap();
    assert!(
        ledger
            .fields
            .iter()
            .all(|f| f.entries.len() == 16 && f.value.bits == "8388608000000000000")
    );
    let packet = oh_proto::ServerMessage::MilitaryNormalLedgerResult {
        request: format!("military-ledger:{}", u64::MAX),
        supported: true,
        reason_key: None,
        ledger: Some(ledger),
    };
    let bytes = oh_proto::encode(&packet).unwrap();
    assert!(
        bytes.len() < 65536,
        "selected packet bytes: {}",
        bytes.len()
    );
    assert_eq!(oh_proto::decode_server(&bytes).unwrap(), packet);
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/wp22");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("ledger-worst.json"),
        serde_json::to_vec_pretty(&packet).unwrap(),
    )
    .unwrap();
    std::fs::write(
        out.join("ledger-worst-size.json"),
        format!("{{\"bytes\":{}}}\n", bytes.len()),
    )
    .unwrap();
}
