use oh_proto::{ClientMessage, PROTOCOL_VERSION, ServerMessage, TimeState};
#[test]
fn req_net_01_messagepack_roundtrip() {
    let hello = ClientMessage::Hello {
        protocol_version: PROTOCOL_VERSION.into(),
    };
    assert_eq!(
        oh_proto::decode_client(&oh_proto::encode(&hello).unwrap()).unwrap(),
        hello
    );
    let snapshot = ServerMessage::Snapshot {
        state: TimeState {
            date: "2000-01-01".into(),
            hour: 0,
            tick: "0".into(),
            paused: true,
            speed: 1,
        },
    };
    assert_eq!(
        oh_proto::decode_server(&oh_proto::encode(&snapshot).unwrap()).unwrap(),
        snapshot
    );
}
#[test]
fn req_net_04_generated_types_are_current() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client/src/proto/protocol.ts");
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        oh_proto::typescript()
    );
}

#[test]
fn m1_actual_world_messagepack_preserves_applied_ledger_and_full_display_strings() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    loaded.scenario.state_modifiers.insert(
        1,
        vec![
            oh_data::national::ModifierInput {
                source: "fixture-add".into(),
                target_stat: "infrastructure".into(),
                operation: "add".into(),
                value: "0.5".into(),
                expires: Some(2),
            },
            oh_data::national::ModifierInput {
                source: "fixture-mul".into(),
                target_stat: "infrastructure".into(),
                operation: "mul".into(),
                value: "2".into(),
                expires: None,
            },
        ],
    );
    let mut sim = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        u64::MAX,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap();
    let view = oh_proto::WorldView::from_sim(&sim).unwrap();
    let ledger = &view.states[0].infrastructure;
    assert_eq!(
        (&ledger.base, &ledger.final_value),
        (&"1".to_owned(), &"3".to_owned())
    );
    assert_eq!(
        ledger
            .entries
            .iter()
            .map(|e| e.accumulated.as_str())
            .collect::<Vec<_>>(),
        vec!["1", "1.5", "3"]
    );
    assert_eq!(
        ledger
            .entries
            .iter()
            .map(|e| e.operation_key.as_str())
            .collect::<Vec<_>>(),
        vec!["ledger-base", "ledger-add", "ledger-multiply"]
    );
    let message = ServerMessage::WorldResult {
        request: u64::MAX.to_string(),
        supported: true,
        reason_key: None,
        world: Some(view),
    };
    assert_eq!(
        oh_proto::decode_server(&oh_proto::encode(&message).unwrap()).unwrap(),
        message
    );
    sim.step().unwrap();
    sim.step().unwrap();
    let view = oh_proto::WorldView::from_sim(&sim).unwrap();
    assert_eq!(view.states[0].infrastructure.final_value, "2");
    assert_eq!(view.states[0].infrastructure.tick, "2");
}
#[test]
fn req_time_04_trigger_query_retains_typed_end_and_exact_u64_text() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    loaded.scenario.end_conditions =
        Some(oh_data::trigger::parse_condition(r#"{"date_gte":"2000-01-01"}"#).unwrap());
    let sim = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        7,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap();
    let view = oh_proto::TriggerView::from_sim(&sim).unwrap();
    assert_eq!(view.ended.as_ref().unwrap().tick, "0");
    assert_eq!(
        view.ended.as_ref().unwrap().causes,
        vec![oh_proto::EndCauseView::Condition]
    );
    let message = oh_proto::ServerMessage::TriggerResult {
        request: "end".into(),
        supported: true,
        reason_key: None,
        trigger: Some(view),
    };
    assert_eq!(
        oh_proto::decode_server(&oh_proto::encode(&message).unwrap()).unwrap(),
        message
    );
}
