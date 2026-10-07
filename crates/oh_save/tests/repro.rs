use oh_save::{
    SaveContext, repro,
    repro_zip::{self, Policy},
};
use std::{collections::BTreeMap, path::Path};

#[test]
fn req_sav_05_archive_each_and_total_bounds_and_original_authority_preserved() {
    let members = BTreeMap::from([
        ("bundle.toml".into(), vec![42; 20]),
        ("commands.log".into(), vec![43; 20]),
    ]);
    let p = Policy::default();
    let bytes = repro_zip::encode(&members, &p).unwrap();
    for policy in [
        Policy {
            member_max_bytes: 19,
            ..p.clone()
        },
        Policy {
            total_max_bytes: 39,
            ..p.clone()
        },
        Policy {
            archive_max_bytes: bytes.len() as u64 - 1,
            ..p.clone()
        },
        Policy {
            members_max: 1,
            ..p.clone()
        },
    ] {
        assert!(repro_zip::decode(&bytes, &policy).is_err());
        assert!(repro_zip::encode(&members, &policy).is_err());
    }
    assert_eq!(repro_zip::decode(&bytes, &p).unwrap(), members);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let c = SaveContext::national(&root, "m1").unwrap();
    let mut sim = c.simulation(7).unwrap();
    sim.enqueue(100, oh_core::NationId(0), 1, oh_sim::Command::Pause(true))
        .unwrap();
    let before = repro::report(&sim).unwrap();
    assert!(repro::replay(&bytes, &c).is_err());
    assert_eq!(before, repro::report(&sim).unwrap());
    assert_eq!(sim.pending_commands().len(), 1);
}

#[test]
fn req_sav_05_optional_start_uses_exact_seed_context_and_full_report() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let c = SaveContext::national(&root, "m1").unwrap();
    let mut sim = c.simulation(7).unwrap();
    let mut recorder = repro::Recorder::new(&sim, &c).unwrap();
    recorder.input(&mut sim, repro::Input::Step {}).unwrap();
    let bytes = recorder.finish(&sim, &c).unwrap();
    let mut members = repro_zip::decode(&bytes, &Policy::default()).unwrap();
    let mut bundle = repro::inspect(&bytes).unwrap();
    bundle.start_save = false;
    members.remove("start.ohsave");
    members.insert(
        "bundle.toml".into(),
        toml::to_string(&bundle).unwrap().into_bytes(),
    );
    let bytes = repro_zip::encode(&members, &Policy::default()).unwrap();
    let restored = repro::replay(&bytes, &c).unwrap();
    assert_eq!(
        repro::report(&sim).unwrap(),
        repro::report(&restored).unwrap()
    );
    bundle.seed = 8;
    members.insert(
        "bundle.toml".into(),
        toml::to_string(&bundle).unwrap().into_bytes(),
    );
    assert!(
        repro::replay(
            &repro_zip::encode(&members, &Policy::default()).unwrap(),
            &c
        )
        .is_err()
    );
}

#[test]
fn req_sav_05_machine_human_reagreement_cannot_forge_actual_pump_results() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let c = SaveContext::national(&root, "m1").unwrap();
    let mut sim = c.simulation(7).unwrap();
    let mut recorder = repro::Recorder::new(&sim, &c).unwrap();
    recorder.input(&mut sim, repro::Input::Step {}).unwrap();
    let bytes = recorder.finish(&sim, &c).unwrap();
    let mut members = repro_zip::decode(&bytes, &Policy::default()).unwrap();
    let mut events: Vec<repro::Event> = serde_json::from_slice(&members["commands.log"]).unwrap();
    events[0].outcome.advanced = false;
    members.insert("commands.log".into(), serde_json::to_vec(&events).unwrap());
    members.insert("commands.txt".into(), repro::human(&events).unwrap());
    let error = repro::replay(
        &repro_zip::encode(&members, &Policy::default()).unwrap(),
        &c,
    )
    .unwrap_err();
    assert!(error.contains("event 0 mismatch"), "{error}");
    assert_eq!(sim.snapshot().tick(), 1);
}

#[test]
fn req_sav_05_untrusted_array_limits_apply_before_allocating_overflow_entries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let c = SaveContext::national(&root, "m1").unwrap();
    let mut sim = c.simulation(7).unwrap();
    let mut recorder = repro::Recorder::new(&sim, &c).unwrap();
    recorder.input(&mut sim, repro::Input::Step {}).unwrap();
    let bytes = recorder.finish(&sim, &c).unwrap();
    let original = repro_zip::decode(&bytes, &Policy::default()).unwrap();
    for phase_overflow in [false, true] {
        let mut members = original.clone();
        let mut events: Vec<repro::Event> =
            serde_json::from_slice(&members["commands.log"]).unwrap();
        if phase_overflow {
            events[0].outcome.phases =
                vec![String::new(); Policy::default().phases_per_event_max + 1];
        } else {
            events.push(events[0].clone());
        }
        members.insert("commands.log".into(), serde_json::to_vec(&events).unwrap());
        let error = repro::replay(
            &repro_zip::encode(&members, &Policy::default()).unwrap(),
            &c,
        )
        .unwrap_err();
        assert!(error.contains("array limit"), "{error}");
    }
    assert_eq!(sim.snapshot().tick(), 1);
}
