#[path = "../../oh_data/tests/support/production.rs"]
mod fixture;
use oh_core::{NationId, Qty};
use oh_sim::{Command, production::Action};
#[test]
fn req_eco_mil_v6_actual_codec_fractional_carry_future_queue_and_resume() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    s.enqueue(
        0,
        NationId(1),
        1,
        Command::Production(Action::Create {
            model: "test_model_1".into(),
            requested_ic: Qty::from_num(1),
        }),
    )
    .unwrap();
    for _ in 0..24 {
        s.step().unwrap();
    }
    s.enqueue(
        60,
        NationId(1),
        2,
        Command::Production(Action::Switch {
            line: 0,
            model: "test_model_2".into(),
        }),
    )
    .unwrap();
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    assert_eq!(
        oh_save::inspect_header(&bytes, &Default::default())
            .unwrap()
            .format_version,
        6
    );
    let mut restored = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(
        oh_core::canonical_bytes(&s).unwrap(),
        oh_core::canonical_bytes(&restored).unwrap()
    );
    for _ in 0..240 {
        s.step().unwrap();
        restored.step().unwrap();
    }
    assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
    assert_eq!(
        oh_save::repro::report(&s).unwrap(),
        oh_save::repro::report(&restored).unwrap()
    );
    let dto = s.export_save_v6().unwrap();
    assert!(dto.base.queue.is_empty());
}
#[test]
fn req_eco_03_v6_bounds_reject_truncation_and_corrupt_presence() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let s = c.simulation(1).unwrap();
    let b = oh_save::encode(&s, &c, 0, vec![]).unwrap();
    for n in [0, 1, 5, 9, b.len() - 1] {
        assert!(oh_save::decode(&b[..n], &c, false).is_err());
    }
    for limits in [
        oh_save::Limits {
            body_max_bytes: 1,
            ..Default::default()
        },
        oh_save::Limits {
            map_entries_max: 0,
            ..Default::default()
        },
        oh_save::Limits {
            allocation_budget_bytes: 1,
            ..Default::default()
        },
    ] {
        assert!(oh_save::decode_with_limits(&b, &c, false, &limits).is_err());
    }
    let mut dto = s.export_save_v6().unwrap();
    dto.base.queue.push(oh_sim::economy_save::PendingV5 {
        tick: 0,
        nation: 1,
        sequence: 1,
        command: oh_sim::economy_save::CommandV5::Pause(true),
    });
    assert!(
        oh_sim::Simulation::from_save_v6(
            dto,
            &oh_sim::save_state::RestoreContext {
                scenario: "m1".into(),
                start_date: c.simulation(1).unwrap().snapshot().date(),
                world: s.world().cloned()
            }
        )
        .is_err()
    );
}

#[test]
fn req_eco_03_v6_force_never_inserts_production_into_legacy_mode() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let s = c.simulation(1).unwrap();
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    let old = oh_save::SaveContext::national(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap();
    let oldbytes = oh_save::encode(&old.simulation(1).unwrap(), &old, 0, vec![]).unwrap();
    for force in [false, true] {
        assert!(oh_save::decode(&bytes, &old, force).is_err());
        assert!(oh_save::decode(&oldbytes, &c, force).is_err());
    }
    let path = root.join("defines.toml");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("\n# source-only content hash change\n");
    std::fs::write(path, text).unwrap();
    let changed = oh_save::SaveContext::national(&root, "m1").unwrap();
    assert!(oh_save::decode(&bytes, &changed, false).is_err());
    let forced = oh_save::decode(&bytes, &changed, true).unwrap();
    assert!(!forced.warnings.is_empty());
    assert_eq!(
        forced.simulation.state_hash().unwrap(),
        s.state_hash().unwrap()
    );
}

#[test]
fn req_eco_03_v6_preserves_future_intent_after_line_cancellation() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    s.enqueue(
        0,
        NationId(1),
        1,
        Command::Production(Action::Create {
            model: "test_model_1".into(),
            requested_ic: Qty::ONE,
        }),
    )
    .unwrap();
    s.step().unwrap();
    s.enqueue(
        3,
        NationId(1),
        2,
        Command::Production(Action::Pause {
            line: 0,
            paused: true,
        }),
    )
    .unwrap();
    s.enqueue(
        1,
        NationId(1),
        3,
        Command::Production(Action::Cancel { line: 0 }),
    )
    .unwrap();
    s.step().unwrap();
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    let mut restored = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    s.step().unwrap();
    restored.step().unwrap();
    let a = s.step().unwrap();
    let b = restored.step().unwrap();
    assert_eq!(a.commands, b.commands);
    assert!(a.commands[0].result.is_err());
    assert!(restored.pending_commands().is_empty());
    assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
}

#[test]
fn req_eco_03_v6_rejects_forged_authority_day_line_stock_and_queue() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    s.enqueue(
        0,
        NationId(1),
        1,
        Command::Production(Action::Create {
            model: "test_model_1".into(),
            requested_ic: Qty::ONE,
        }),
    )
    .unwrap();
    for _ in 0..24 {
        s.step().unwrap();
    }
    let context = oh_sim::save_state::RestoreContext {
        scenario: "m1".into(),
        start_date: c.simulation(1).unwrap().snapshot().date(),
        world: s.world().cloned(),
    };
    let dto = s.export_save_v6().unwrap();
    let raw = serde_json::to_value(&dto).unwrap();
    for pointer in [
        "/production/definitions_hash",
        "/production/lines/0/carry",
        "/production/lines/0/efficiency",
        "/production/stock/1/test_model_1",
        "/production/day/nations/1/lines/0/output",
        "/production/day/nations/1/lines/0/starting/nation",
    ] {
        let mut bad = raw.clone();
        let field = bad.pointer_mut(pointer).unwrap();
        *field = match pointer {
            "/production/definitions_hash" => serde_json::json!(0),
            "/production/lines/0/carry" => serde_json::to_value(Qty::ONE).unwrap(),
            "/production/lines/0/efficiency" => {
                serde_json::to_value(oh_core::Fx::from_num(2)).unwrap()
            }
            "/production/stock/1/test_model_1" => serde_json::json!(-1),
            "/production/day/nations/1/lines/0/starting/nation" => serde_json::json!(2),
            _ => serde_json::json!(100),
        };
        let bad: oh_sim::production_save::SimulationSaveV6 = serde_json::from_value(bad).unwrap();
        assert!(
            oh_sim::Simulation::from_save_v6(bad, &context).is_err(),
            "{pointer}"
        );
    }
    let mut bad = dto.clone();
    bad.queue.push(oh_sim::production_save::PendingV6 {
        tick: 25,
        nation: 1,
        sequence: 9,
        command: oh_sim::production_save::CommandV6::Production(Action::Cancel { line: 1 }),
    });
    assert!(oh_sim::Simulation::from_save_v6(bad, &context).is_err());
    assert!(oh_sim::Simulation::from_save_v6(dto, &context).is_ok());
}
