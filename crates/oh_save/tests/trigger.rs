use oh_save::{SaveContext, decode, encode};
use std::path::Path;
fn copy(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for e in std::fs::read_dir(source).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copy(&e.path(), &target.join(e.file_name()));
        } else {
            std::fs::copy(e.path(), target.join(e.file_name())).unwrap();
        }
    }
}
fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        temp.path(),
    );
    std::fs::write(
        temp.path().join("scenarios/m1/scenario.toml"),
        include_str!("fixtures/trigger/scenario.toml"),
    )
    .unwrap();
    temp
}
#[test]
fn req_time_04_v4_codec_limits_trailing_full_state_and_future_queue() {
    let pack = fixture();
    let c = SaveContext::national(pack.path(), "m1").unwrap();
    let mut s = c.simulation(7).unwrap();
    s.enqueue(
        99,
        oh_core::NationId(1),
        1,
        oh_sim::Command::Effects {
            program: "prepare".into(),
        },
    )
    .unwrap();
    for _ in 0..48 {
        s.step().unwrap();
    }
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    assert_eq!(bytes[4], 4);
    let loaded = decode(&bytes, &c, false).unwrap();
    assert!(loaded.simulation.is_ended());
    assert_eq!(
        s.export_save_v4().unwrap(),
        loaded.simulation.export_save_v4().unwrap()
    );
    assert_eq!(encode(&loaded.simulation, &c, 0, vec![]).unwrap(), bytes);
    let limits = oh_save::Limits {
        queue_max_entries: 0,
        ..Default::default()
    };
    assert!(
        oh_save::decode_with_limits(&bytes, &c, false, &limits)
            .unwrap_err_string()
            .contains("Limit")
    );
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(decode(&trailing, &c, false).is_err());
    let mut future = bytes.clone();
    future[4] = 5;
    assert!(decode(&future, &c, false).is_err());
}
trait ErrText {
    fn unwrap_err_string(self) -> String;
}
impl ErrText for oh_save::Result<oh_save::LoadedSave> {
    fn unwrap_err_string(self) -> String {
        match self {
            Err(e) => e,
            Ok(_) => panic!("expected rejection"),
        }
    }
}
#[test]
fn req_time_04_legacy_none_local_some_and_force_cannot_drop_new_definitions() {
    let pack = fixture();
    let path = pack.path().join("scenarios/m1/scenario.toml");
    let new_source = std::fs::read(&path).unwrap();
    let original = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/packs/testland/scenarios/m1/scenario.toml");
    std::fs::write(&path, std::fs::read(original).unwrap()).unwrap();
    let old = SaveContext::national(pack.path(), "m1").unwrap();
    let s = old.simulation(7).unwrap();
    let old_bytes = encode(&s, &old, 0, vec![]).unwrap();
    std::fs::write(&path, &new_source).unwrap();
    let new = SaveContext::national(pack.path(), "m1").unwrap();
    for force in [false, true] {
        let error = decode(&old_bytes, &new, force).unwrap_err_string();
        assert!(error.contains("TriggerModeMismatch"), "{error}");
    }
    assert!(
        oh_sim::Simulation::from_save(s.export_save().unwrap(), new.restore_context())
            .unwrap_err()
            .contains("TriggerModeMismatch")
    );
    let mut changed = s.export_save().unwrap();
    changed.state.speed = 2;
    assert!(oh_sim::Simulation::from_save(changed, new.restore_context()).is_err());
    assert_eq!(new_source, std::fs::read(&path).unwrap());
}
