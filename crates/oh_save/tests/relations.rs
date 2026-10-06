#[path = "support/copy.rs"]
mod support;
use oh_core::{NationId, canonical_bytes};
use oh_save::{SaveContext, decode, encode, pack_hash};
use oh_sim::{Command, Simulation, save_state::*};
use std::{fs, path::Path};
fn change(root: &Path, file: &str, from: &str, to: &str) {
    let p = root.join(file);
    let text = fs::read_to_string(&p).unwrap();
    assert!(text.contains(from), "{file}: {from}");
    fs::write(p, text.replace(from, to)).unwrap();
}
#[test]
fn req_sav_01_noncontiguous_zero_and_max_ids_survive_restore() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("pack");
    support::copy_pack(&root);
    change(&root, "scenarios/m1/nations/NTH.toml", "id = 1", "id = 0");
    change(
        &root,
        "scenarios/m1/nations/NTH.toml",
        "capital = 10",
        "capital = 0",
    );
    change(&root, "scenarios/m1/nations/STH.toml", "id = 2", "id = 42");
    change(
        &root,
        "scenarios/m1/nations/STH.toml",
        "capital = 30",
        "capital = 65535",
    );
    let file = "maps/testland/provinces.csv";
    for (from, to) in [
        ("10,200", "0,200"),
        ("20,40,200", "10,40,200"),
        ("30,40,40", "65535,40,40"),
        ("40,200,200", "30,200,200"),
    ] {
        change(&root, file, from, to);
    }
    let file = "maps/testland/states.toml";
    for (from, to) in [
        ("id = 1", "id = 3"),
        ("id = 2", "id = 65535"),
        ("[10, 20]", "[0, 10]"),
        ("[30, 40]", "[65535, 30]"),
        ("province = 10", "province = 0"),
        ("province = 30", "province = 65535"),
    ] {
        change(&root, file, from, to);
    }
    change(
        &root,
        "scenarios/m1/scenario.toml",
        "1 = \"NTH\"",
        "3 = \"NTH\"",
    );
    change(
        &root,
        "scenarios/m1/scenario.toml",
        "2 = \"STH\"",
        "65535 = \"STH\"",
    );
    change(
        &root,
        "scenarios/m1/scenario.toml",
        "20 = \"STH\"",
        "10 = \"STH\"",
    );
    change(&root, "maps/testland/visuals.toml", "1 = [140", "3 = [140");
    change(
        &root,
        "maps/testland/visuals.toml",
        "2 = [185",
        "65535 = [185",
    );
    fs::write(
        root.join("maps/testland/adjacency_overrides.csv"),
        "a,b,kind\n0,10,river_small\n30,65535,river_large\n0,30,strait\n30,50,impassable\n",
    )
    .unwrap();
    let c = SaveContext::national(&root, "m1").unwrap();
    let sim = c.simulation(0).unwrap();
    let loaded = decode(&encode(&sim, &c, 0, vec![0, 42]).unwrap(), &c, false).unwrap();
    let world = loaded.simulation.world().unwrap();
    let p = world.province(oh_core::ProvinceId(0)).unwrap();
    assert_eq!(p.owner(), Some(NationId(0)));
    assert_eq!(p.state(), Some(oh_core::StateId(3)));
    assert_eq!(
        world
            .province(oh_core::ProvinceId(10))
            .unwrap()
            .controller(),
        Some(NationId(42))
    );
    assert!(world.state(oh_core::StateId(65535)).is_some());
    assert!(world.province(oh_core::ProvinceId(65535)).is_some());
    assert_eq!(
        world.province(oh_core::ProvinceId(50)).unwrap().owner(),
        None
    );
    assert_eq!(
        loaded.simulation.state_hash().unwrap(),
        sim.state_hash().unwrap()
    );
}
#[test]
fn req_sav_01_modifiers_overflow_duplicates_expired_and_floor() {
    let c = SaveContext::national(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap();
    let original = c.simulation(7).unwrap();
    let dto = original.export_save().unwrap();
    let before = canonical_bytes(&original).unwrap();
    let make = |value, expires| ModifierV1 {
        source: "a".into(),
        target_stat: "infrastructure".into(),
        op: ModifierOpV1::Add,
        value,
        expires,
    };
    let mut expired = dto.clone();
    expired.world.as_mut().unwrap().inputs.states[0].modifiers =
        vec![make(1, Some(0)), make(1, Some(0))];
    let valid = Simulation::from_save(expired, c.restore_context()).unwrap();
    let saved = encode(&valid, &c, 0, vec![]).unwrap();
    assert_eq!(
        decode(&saved, &c, false)
            .unwrap()
            .simulation
            .export_save()
            .unwrap()
            .world
            .unwrap()
            .inputs
            .states[0]
            .modifiers
            .len(),
        2
    );
    for modifiers in [
        vec![make(1, None), make(1, None)],
        vec![make(i64::MAX, None)],
        vec![ModifierV1 {
            source: String::new(),
            ..make(1, Some(0))
        }],
        vec![ModifierV1 {
            target_stat: "other".into(),
            ..make(1, Some(0))
        }],
    ] {
        let mut bad = dto.clone();
        bad.world.as_mut().unwrap().inputs.states[0].modifiers = modifiers;
        assert!(Simulation::from_save(bad, c.restore_context()).is_err());
        assert_eq!(canonical_bytes(&original).unwrap(), before);
    }
    let mut time = dto.clone();
    time.state.date = DateV1 {
        year: 1900,
        month: 2,
        day: 29,
    };
    assert!(Simulation::from_save(time, c.restore_context()).is_err());
    let mut past = original.clone();
    assert!(past.step().unwrap().advanced);
    let mut queue = past.export_save().unwrap();
    queue.queue.push(PendingV1 {
        tick: 0,
        nation: 0,
        sequence: 0,
        command: CommandV1::Pause(false),
    });
    assert!(Simulation::from_save(queue, c.restore_context()).is_err());
}
#[test]
fn req_sav_01_whole_path_sort_and_save_location() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a")).unwrap();
    fs::write(dir.path().join("a/x"), b"x").unwrap();
    fs::write(dir.path().join("a.txt"), b"text").unwrap();
    let expected =
        oh_core::state_hash(&vec![("a.txt", b"text".to_vec()), ("a/x", b"x".to_vec())]).unwrap();
    assert_eq!(pack_hash(dir.path()).unwrap(), expected);
    let c = SaveContext::national(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap();
    assert!(oh_save::write_atomic(&c.root().join("bad.ohsave"), b"bad", &c).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(dir.path().join("a.txt"), dir.path().join("link")).unwrap();
        assert!(pack_hash(dir.path()).is_err());
    }
}
#[cfg(unix)]
#[test]
fn req_sav_01_literal_backslash_is_not_a_directory_separator() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a")).unwrap();
    fs::write(dir.path().join("a/x"), b"nested").unwrap();
    fs::write(dir.path().join("a\\x"), b"literal").unwrap();
    let expected = oh_core::state_hash(&vec![
        ("a/x", b"nested".to_vec()),
        ("a\\x", b"literal".to_vec()),
    ])
    .unwrap();
    assert_eq!(pack_hash(dir.path()).unwrap(), expected);
}
#[test]
fn req_sav_02_paused_failure_keeps_input_file_and_live_queue() {
    let c = SaveContext::national(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap();
    let mut sim = c.simulation(7).unwrap();
    sim.enqueue(0, NationId(0), 0, Command::Pause(true))
        .unwrap();
    assert!(!sim.step().unwrap().advanced);
    sim.enqueue(5, NationId(0), 1, Command::SetSpeed(5))
        .unwrap();
    let bytes = encode(&sim, &c, 0, vec![]).unwrap();
    assert_eq!(
        decode(&bytes, &c, false).unwrap().simulation.snapshot(),
        sim.snapshot()
    );
    assert_eq!(sim.pending_commands().len(), 1);
}
