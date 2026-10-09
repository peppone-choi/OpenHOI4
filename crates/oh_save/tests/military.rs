#[path = "support/military.rs"]
mod fixture;
use oh_core::{NationId, ProvinceId};
use oh_sim::{
    Command,
    military::{Action, JobStatus},
};
fn command(s: &mut oh_sim::Simulation, sequence: u64, a: Action) {
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        sequence,
        Command::Military(a),
    )
    .unwrap();
    assert!(s.step().unwrap().commands.iter().all(|c| c.result.is_ok()));
}
#[test]
fn req_mil_v7_real_codec_pending_training_ready_deployed_cancelled_and_resume() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    command(
        &mut s,
        1,
        Action::Train {
            template: "example".into(),
        },
    );
    for target in [1, 24, 48, 72] {
        while s.snapshot().tick() < target {
            s.step().unwrap();
        }
        s.enqueue(
            target + 10,
            NationId(1),
            100 + target,
            Command::Military(Action::SetPriority {
                army: 0,
                priority: 2,
            }),
        )
        .unwrap();
        let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
        assert_eq!(
            oh_save::inspect_header(&bytes, &Default::default())
                .unwrap()
                .format_version,
            7
        );
        let mut restored = oh_save::decode(&bytes, &c, false).unwrap().simulation;
        assert_eq!(
            oh_core::canonical_bytes(&s).unwrap(),
            oh_core::canonical_bytes(&restored).unwrap()
        );
        assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
        let mut continuous = s.clone();
        for _ in 0..12 {
            continuous.step().unwrap();
            restored.step().unwrap();
        }
        assert_eq!(
            oh_core::canonical_bytes(&continuous).unwrap(),
            oh_core::canonical_bytes(&restored).unwrap()
        );
    }
    assert_eq!(s.military().unwrap().jobs()[&0].status(), JobStatus::Ready);
    command(
        &mut s,
        500,
        Action::Deploy {
            job: 0,
            army: 0,
            province: ProvinceId(10),
            allow_understrength: true,
        },
    );
    command(
        &mut s,
        501,
        Action::Train {
            template: "example".into(),
        },
    );
    command(&mut s, 502, Action::Cancel { job: 1 });
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    let restored = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(
        restored.military().unwrap().jobs()[&0].status(),
        JobStatus::Deployed
    );
    assert_eq!(
        restored.military().unwrap().jobs()[&1].status(),
        JobStatus::Cancelled
    );
    assert_eq!(restored.military().unwrap().divisions()[&0].manpower(), 8);
    assert!(s.export_save_v6().is_err());
    let dto = s.export_save_v7().unwrap();
    assert!(dto.base.queue.is_empty());
    assert!(dto.base.base.queue.is_empty());
}
#[test]
fn req_mil_v7_untrusted_bounds_immutable_definitions_and_owned_accounting_rejections() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    command(
        &mut s,
        1,
        Action::Train {
            template: "example".into(),
        },
    );
    while s.snapshot().tick() < 24 {
        s.step().unwrap();
    }
    let bytes = oh_save::encode(&s, &c, 0, vec![]).unwrap();
    for n in [0, 1, 9, bytes.len() - 1] {
        assert!(oh_save::decode(&bytes[..n], &c, false).is_err());
    }
    for limits in [
        oh_save::Limits {
            queue_max_entries: 0,
            ..Default::default()
        },
        oh_save::Limits {
            allocation_budget_bytes: 1,
            ..Default::default()
        },
        oh_save::Limits {
            body_max_bytes: 1,
            ..Default::default()
        },
    ] {
        assert!(oh_save::decode_with_limits(&bytes, &c, false, &limits).is_err());
    }
    let dto = s.export_save_v7().unwrap();
    let raw = serde_json::to_value(&dto).unwrap();
    for (pointer, value) in [
        ("/military/definitions_hash", serde_json::json!(0)),
        ("/military/jobs/0/reserved_manpower", serde_json::json!(7)),
        ("/military/jobs/0/progress_days", serde_json::json!(1)),
        ("/military/jobs/0/normal/manpower", serde_json::json!(9)),
        ("/military/jobs/0/nation", serde_json::json!(2)),
        ("/military/background/1/reserved", serde_json::json!(11)),
        ("/military/armies/0/general", serde_json::json!("changed")),
    ] {
        let mut bad = raw.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        let bad = serde_json::from_value::<oh_sim::military_save::SimulationSaveV7>(bad).unwrap();
        assert!(
            oh_sim::Simulation::from_save_v7(bad, c.restore_context()).is_err(),
            "{pointer}"
        );
    }
    let old = oh_save::SaveContext::national(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap();
    for force in [false, true] {
        assert!(oh_save::decode(&bytes, &old, force).is_err());
    }
}
