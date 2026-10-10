#[path = "../../oh_data/tests/support/m2_military_pack.rs"]
mod fixture;
#[path = "support/m2_military.rs"]
mod runtime;
use oh_core::{NationId, canonical_bytes};
use oh_sim::{
    Command,
    military::{Action, JobStatus, MilitaryError},
};
#[test]
fn observed_production_training_cancel_conservation_and_v7_resume() {
    let p = fixture::CopyPack::new();
    let c = oh_save::SaveContext::national(p.root(), fixture::SCENARIO).unwrap();
    let (mut s, summary) = runtime::checkpoint(&c);
    // Regression values come from the preserved first actual production observation.
    assert_eq!(summary["days"], serde_json::json!([5, 5, 5, 5, 5, 5]));
    assert_eq!(summary["train_enqueued_tick"], "120");
    assert_eq!(summary["tick"], "144");
    for n in summary["nations"].as_array().unwrap() {
        assert_eq!(n["held"], "6");
        assert_eq!(n["stock"], "2");
    }
    let bytes = oh_save::encode(&s, &c, 0, vec![1, 2, 3, 4, 5, 6]).unwrap();
    assert_eq!(
        oh_save::inspect_header(&bytes, &Default::default())
            .unwrap()
            .format_version,
        7
    );
    if let Some(out) = std::env::var_os("OH_M2_MILITARY_OUTPUT_DIR") {
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        assert!(!out.join("paused-training.ohsave").exists());
        std::fs::write(out.join("paused-training.ohsave"), &bytes).unwrap();
        std::fs::write(
            out.join("expected.json"),
            serde_json::to_vec_pretty(&summary).unwrap(),
        )
        .unwrap();
    }
    let mut resumed = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(
        canonical_bytes(&s).unwrap(),
        canonical_bytes(&resumed).unwrap()
    );
    for n in 1..=6 {
        let job = u64::from(n - 1);
        let e = s.economy().unwrap().nation(NationId(n)).unwrap().clone();
        let held = s.military().unwrap().jobs()[&job].equipment()["m2_equipment_1"];
        let stock = runtime::stock(&s, n);
        let production = s.production().unwrap().lines().clone();
        let next = s.military().unwrap().next_job_id();
        s.enqueue(
            s.snapshot().tick(),
            NationId(n),
            100 + u64::from(n),
            Command::Military(Action::Cancel { job }),
        )
        .unwrap();
        assert!(s.step().unwrap().commands[0].result.is_ok());
        let after = s.economy().unwrap().nation(NationId(n)).unwrap();
        assert_eq!(
            (after.reserved(), after.available(), after.committed()),
            (e.reserved() - 8, e.available() + 8, e.committed())
        );
        assert_eq!(runtime::stock(&s, n), stock + held);
        assert_eq!(s.production().unwrap().lines(), &production);
        let j = &s.military().unwrap().jobs()[&job];
        assert_eq!(j.status(), JobStatus::Cancelled);
        assert_eq!(
            (j.reserved_manpower(), j.progress_days(), j.start_tick()),
            (0, 0, None)
        );
        assert!(j.equipment().is_empty());
        assert_eq!(s.military().unwrap().next_job_id(), next);
        assert!(s.military().unwrap().divisions().is_empty());
        assert!(s.pending_commands().is_empty());
        let before = canonical_bytes(&s).unwrap();
        let hash = s.state_hash().unwrap();
        assert_eq!(
            s.enqueue(
                s.snapshot().tick(),
                NationId(n % 6 + 1),
                900,
                Command::Military(Action::Cancel { job })
            ),
            Err(oh_sim::Error::Military(MilitaryError::NotOwner))
        );
        assert_eq!(canonical_bytes(&s).unwrap(), before);
        assert_eq!(s.state_hash().unwrap(), hash);
        s.enqueue(
            s.snapshot().tick(),
            NationId(n),
            901,
            Command::Military(Action::Cancel { job }),
        )
        .unwrap();
        assert_eq!(
            s.step().unwrap().commands[0].result,
            Err(oh_sim::Error::Military(MilitaryError::Terminal))
        );
        assert_eq!(canonical_bytes(&s).unwrap(), before);
        assert_eq!(
            s.enqueue(
                s.snapshot().tick(),
                NationId(n),
                902,
                Command::Military(Action::Train {
                    template: "missing".into()
                })
            ),
            Err(oh_sim::Error::Military(MilitaryError::InvalidReference))
        );
        assert_eq!(canonical_bytes(&s).unwrap(), before);
    }
    runtime::cancel_tail(&mut resumed);
    assert_eq!(
        canonical_bytes(&s).unwrap(),
        canonical_bytes(&resumed).unwrap()
    );
    assert_eq!(
        oh_save::encode(&s, &c, 0, vec![]).unwrap(),
        oh_save::encode(&resumed, &c, 0, vec![]).unwrap()
    );
    let (mut again, summary2) = runtime::checkpoint(&c);
    assert_eq!(summary, summary2);
    runtime::cancel_tail(&mut again);
    assert_eq!(
        canonical_bytes(&s).unwrap(),
        canonical_bytes(&again).unwrap()
    );
    assert_eq!(s.state_hash().unwrap(), again.state_hash().unwrap());
}
#[test]
fn nonzero_background_without_ownership_is_accounting_mismatch() {
    let p = fixture::CopyPack::new();
    let valid = oh_save::SaveContext::national(p.root(), fixture::SCENARIO).unwrap();
    assert!(valid.simulation(1).is_ok());
    p.alter(
        "common/military/initial.toml",
        "1 = { committed = 0",
        "1 = { committed = 1",
    );
    let loaded = oh_data::national::load_scenario(p.root(), fixture::SCENARIO).unwrap();
    let result = oh_sim::Simulation::with_world(
        fixture::SCENARIO.into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    );
    assert_eq!(
        result.unwrap_err(),
        oh_sim::Error::Military(MilitaryError::AccountingMismatch)
    );
    // SaveContext deliberately exposes String, so assert that public boundary separately.
    let c = oh_save::SaveContext::national(p.root(), fixture::SCENARIO).unwrap();
    assert_eq!(
        c.simulation(1).unwrap_err(),
        "military command or phase failed"
    );
}
