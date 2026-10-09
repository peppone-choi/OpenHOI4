use oh_proto::{ClientMessage, MilitaryCommand, ServerMessage};
#[test]
fn req_mil_03_wire_exact_ids_explicit_deploy_and_strict_integer_rejections() {
    let controls = [
        MilitaryCommand::Train {
            template: "normal".into(),
        },
        MilitaryCommand::Cancel {
            job: u64::MAX.to_string(),
        },
        MilitaryCommand::Deploy {
            job: "0".into(),
            army: u64::MAX.to_string(),
            province: u16::MAX,
            allow_understrength: true,
        },
        MilitaryCommand::SetPriority {
            army: "0".into(),
            priority: u16::MAX,
        },
    ];
    for c in controls {
        let action = c.clone().into_action().unwrap();
        assert_eq!(MilitaryCommand::from_action(&action), c);
        let packet = ClientMessage::MilitaryCommand {
            sequence: u64::MAX.to_string(),
            command: c,
        };
        assert_eq!(
            oh_proto::decode_client(&oh_proto::encode(&packet).unwrap()).unwrap(),
            packet
        );
    }
    for id in [
        "",
        "-9223372036854775808",
        "-1",
        "+1",
        "01",
        " 1",
        "1 ",
        "18446744073709551616",
        "1.0",
    ] {
        assert!(
            MilitaryCommand::Cancel { job: id.into() }
                .into_action()
                .is_err(),
            "{id}"
        );
        assert!(
            MilitaryCommand::Deploy {
                job: "0".into(),
                army: id.into(),
                province: 1,
                allow_understrength: false
            }
            .into_action()
            .is_err(),
            "{id}"
        );
    }
    for raw in [
        r#"{"type":"Deploy","job":"0","army":"0","province":1}"#,
        r#"{"type":"Deploy","job":"0","army":"0","province":1,"allow_understrength":true,"nation":2}"#,
        r#"{"type":"Deploy","job":"0","army":"0","province":-32768,"allow_understrength":true}"#,
        r#"{"type":"SetPriority","army":"0","priority":65536}"#,
        r#"{"type":"Train","template":"normal","reserved_manpower":"100"}"#,
    ] {
        assert!(
            serde_json::from_str::<MilitaryCommand>(raw).is_err(),
            "{raw}"
        );
    }
}
#[test]
fn req_mil_query_none_is_capability_failure_not_empty_authority() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    let s = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap();
    assert!(oh_proto::MilitaryView::from_sim(&s).is_none());
    let result = ServerMessage::MilitaryResult {
        request: u64::MAX.to_string(),
        supported: false,
        reason_key: Some("unsupported-query".into()),
        military: None,
    };
    assert_eq!(
        oh_proto::decode_server(&oh_proto::encode(&result).unwrap()).unwrap(),
        result
    );
}

#[path = "../../oh_save/tests/support/military.rs"]
mod fixture;
#[test]
fn req_mil_query_actual_ownership_normal_counts_ready_and_wire_are_equal() {
    let root = fixture::pack();
    let l = oh_data::national::load_scenario(&root, "m1").unwrap();
    let mut s = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&l.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&l).unwrap(),
    )
    .unwrap();
    s.enqueue(
        0,
        oh_core::NationId(1),
        1,
        oh_sim::Command::Military(oh_sim::military::Action::Train {
            template: "example".into(),
        }),
    )
    .unwrap();
    for _ in 0..72 {
        s.step().unwrap();
    }
    let v = oh_proto::MilitaryView::from_sim(&s).unwrap();
    assert_eq!(v.jobs[0].status, oh_proto::MilitaryJobStatus::Ready);
    assert_eq!(v.jobs[0].reserved_manpower, "8");
    assert_eq!(v.jobs[0].progress_days, 2);
    assert_eq!(v.jobs[0].start_tick.as_deref(), Some("24"));
    assert_eq!(v.templates[0].normal.manpower, "8");
    assert_eq!(v.templates[0].normal.equipment[0].count, "8");
    assert!(v.divisions.is_empty());
    assert_eq!(v.background[0].reserved, "10");
    assert_eq!(v.armies[0].nation, 1);
    assert_eq!(v.armies[1].nation, 2);
    let m = ServerMessage::MilitaryResult {
        request: "exact".into(),
        supported: true,
        reason_key: None,
        military: Some(v),
    };
    assert_eq!(
        oh_proto::decode_server(&oh_proto::encode(&m).unwrap()).unwrap(),
        m
    );
}
