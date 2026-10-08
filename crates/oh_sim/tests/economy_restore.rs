use oh_core::NationId;
use oh_sim::{Command, Date, Simulation, TimeConfig, save_state::RestoreContext, world::World};
use std::path::{Path, PathBuf};

fn loaded() -> oh_data::national::LoadedNational {
    fn copy(source: &Path, target: &Path) {
        std::fs::create_dir_all(target).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let source = entry.unwrap().path();
            let target = target.join(source.file_name().unwrap());
            if source.is_dir() {
                copy(&source, &target);
            } else {
                std::fs::copy(source, target).unwrap();
            }
        }
    }
    static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let target = root
        .join("target/evidence/WP-14-M2-r3-P06/sim-inputs")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
    copy(
        &root.join("tests/repro/WP-14-M2-r3-restore-stage/packs/testland"),
        &target,
    );
    std::fs::write(
        target.join("common/buildings.toml"),
        "[[building]]\nid = \"industry\"\n[[building]]\nid = \"unmodeled\"\n[[building]]\nid = \"works\"\n",
    )
    .unwrap();
    oh_data::national::load_scenario(&target, "m1").unwrap()
}
fn initial(l: &oh_data::national::LoadedNational) -> (Simulation, RestoreContext) {
    let world = World::from_loaded(l).unwrap();
    let date = Date::new(2000, 1, 1).unwrap();
    let sim = Simulation::with_world(
        "m1".into(),
        date,
        1000,
        TimeConfig::from_defines(&l.pack.defines).unwrap(),
        world.clone(),
    )
    .unwrap();
    (
        sim,
        RestoreContext {
            scenario: "m1".into(),
            start_date: date,
            world: Some(world),
        },
    )
}
#[test]
fn req_eco_06_req_sav_02_current_authority_building_bounds_not_historical_ledger() {
    let mut failures = Vec::new();
    for (name, level, limit, slot_costs, accepted) in [
        ("zero", 0, 4, vec![1, 1, 1], true),
        ("stage-and-slot-exact", 3, 3, vec![1, 1, 1], true),
        ("stage-overflow", 4, 4, vec![1, 1, 1], false),
        ("negative", -1, 4, vec![1, 1, 1], false),
        ("slot-overflow", 3, 2, vec![1, 1, 1], false),
        (
            "slot-add-overflow",
            2,
            i64::MAX,
            vec![1, i64::MAX, 1],
            false,
        ),
    ] {
        let mut l = loaded();
        let d = l.economy.as_mut().unwrap();
        d.state_slots.insert(1, limit);
        d.buildings.get_mut("industry").unwrap().slots = slot_costs;
        let (mut sim, context) = initial(&l);
        sim.enqueue(99, NationId(1), 42, Command::SetSpeed(2))
            .unwrap();
        let before = oh_core::canonical_bytes(&sim).unwrap();
        let context_before = oh_core::canonical_bytes(context.world.as_ref().unwrap()).unwrap();
        let mut dto = sim.export_save_v5().unwrap();
        dto.base.base.base.world.as_mut().unwrap().inputs.states[0].buildings[0].1 = level;
        let result = Simulation::from_save_v5(dto.clone(), &context);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: accepted={}", result.is_ok()));
        }
        if accepted && result.is_ok() {
            let restored = Simulation::from_save_v5(dto.clone(), &context).unwrap();
            assert_eq!(dto, restored.export_save_v5().unwrap());
            assert_eq!(restored.pending_commands().len(), 1);
            assert_eq!(
                restored
                    .economy()
                    .unwrap()
                    .nation(NationId(1))
                    .unwrap()
                    .ledger()
                    .contributions[0]
                    .levels,
                1
            );
        }
        assert_eq!(oh_core::canonical_bytes(&sim).unwrap(), before);
        assert_eq!(
            oh_core::canonical_bytes(context.world.as_ref().unwrap()).unwrap(),
            context_before
        );
    }
    assert!(failures.is_empty(), "{}", failures.join(", "));
}
#[test]
fn req_eco_06_current_known_refs_include_zero_levels_and_missing_state() {
    let (sim, context) = initial(&loaded());
    for building in ["unregistered", "unmodeled"] {
        let mut dto = sim.export_save_v5().unwrap();
        dto.base.base.base.world.as_mut().unwrap().inputs.states[0].buildings =
            vec![(building.into(), 0)];
        assert!(
            Simulation::from_save_v5(dto, &context).is_err(),
            "accepted {building} even at zero"
        );
    }
    let mut dto = sim.export_save_v5().unwrap();
    dto.base
        .base
        .base
        .world
        .as_mut()
        .unwrap()
        .inputs
        .states
        .remove(0);
    assert!(Simulation::from_save_v5(dto, &context).is_err());
}

#[test]
fn req_eco_06_current_slots_sum_across_buildings_and_check_overflow() {
    let mut failures = Vec::new();
    for (name, industry_level, works_slots, limit, accepted) in [
        ("two-buildings-exact", 3, 1, 4, true),
        ("two-buildings-over", 3, 1, 3, false),
        ("two-buildings-overflow", 1, i64::MAX, i64::MAX, false),
    ] {
        let mut l = loaded();
        let d = l.economy.as_mut().unwrap();
        let mut works = d.buildings["industry"].clone();
        works.slots = vec![works_slots; 3];
        d.buildings.insert("works".into(), works);
        d.state_slots.insert(1, limit);
        let (sim, context) = initial(&l);
        let mut dto = sim.export_save_v5().unwrap();
        dto.base.base.base.world.as_mut().unwrap().inputs.states[0].buildings =
            vec![("industry".into(), industry_level), ("works".into(), 1)];
        let result = Simulation::from_save_v5(dto, &context);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: accepted={}", result.is_ok()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join(", "));
}

#[test]
fn req_sav_02_none_and_explicit_empty_have_no_invented_economic_stage_cap() {
    let mut l = loaded();
    l.scenario.economy = None;
    l.economy = None;
    let (sim, context) = initial(&l);
    let mut legacy = sim.export_save().unwrap();
    legacy.world.as_mut().unwrap().inputs.states[0].buildings[0].1 = 4;
    let restored = Simulation::from_save(legacy.clone(), &context).unwrap();
    assert_eq!(restored.export_save().unwrap(), legacy);

    l.scenario.economy = Some("empty".into());
    l.economy = Some(oh_data::economy::Definition {
        buildings: Default::default(),
        laws: Default::default(),
        nations: Default::default(),
        state_slots: Default::default(),
        economy_category: String::new(),
        conscription_category: String::new(),
    });
    let (sim, context) = initial(&l);
    let mut empty = sim.export_save_v5().unwrap();
    empty.base.base.base.world.as_mut().unwrap().inputs.states[0].buildings[0].1 = 4;
    let restored = Simulation::from_save_v5(empty.clone(), &context).unwrap();
    assert_eq!(restored.export_save_v5().unwrap(), empty);
    assert!(restored.economy().unwrap().nation(NationId(1)).is_none());
}
