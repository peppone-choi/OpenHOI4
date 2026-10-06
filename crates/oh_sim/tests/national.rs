use oh_core::{Fx, ProvinceId, StateId};
use oh_sim::{Date, Simulation, TimeConfig};
fn sim() -> Simulation {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    Simulation::with_world(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap()
}
#[test]
fn req_map_08_owner_and_controller_remain_distinct() {
    let sim = sim();
    let world = sim.world().unwrap();
    let p = world.province(ProvinceId(20)).unwrap();
    assert_ne!(p.owner(), p.controller());
    assert_eq!(world.nation(p.owner().unwrap()).unwrap().tag(), "NTH");
    assert_eq!(world.nation(p.controller().unwrap()).unwrap().tag(), "STH");
}
#[test]
fn ac_m1_03_real_infrastructure_ledger_applied_and_recomputed() {
    let mut sim = sim();
    for _ in 0..25 {
        sim.step().unwrap();
    }
    let s = sim.world().unwrap().state(StateId(1)).unwrap();
    assert_eq!(s.infrastructure(), Fx::ONE);
    assert_eq!(s.ledger().tick(), 25);
    s.ledger().verify_applied_value(s.infrastructure()).unwrap();
    assert_eq!(s.ledger().entries()[0].value, Fx::ONE);
}

#[test]
fn m1_generic_canonical_hash_includes_world() {
    let sim = sim();
    assert_eq!(
        oh_core::state_hash(&sim).unwrap(),
        sim.state_hash().unwrap(),
        "every canonical caller must include M1"
    );
}

fn loaded() -> oh_data::national::LoadedNational {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    oh_data::national::load_scenario(&root, "m1").unwrap()
}
fn from_data(data: &oh_data::national::LoadedNational) -> Simulation {
    Simulation::with_world(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&data.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(data).unwrap(),
    )
    .unwrap()
}
#[test]
fn m1_independent_fnv_oracle_and_every_initial_input_changes_hash() {
    let original = loaded();
    let expected = from_data(&original).state_hash().unwrap();
    let sim = from_data(&original);
    // Independently assemble the canonical field order and implement FNV directly.
    let bytes = oh_core::canonical_bytes(&(
        sim.snapshot(),
        sim.config(),
        sim.pending_commands(),
        sim.world(),
    ))
    .unwrap();
    let oracle = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    assert_eq!(oracle, expected);
    type Mutation = fn(&mut oh_data::national::LoadedNational);
    let cases: &[(&str, Mutation)] = &[
        ("tag", |d| {
            d.nations[0].tag = "ALT".into();
            d.scenario.ownership.insert(1, "ALT".into());
        }),
        ("name", |d| d.nations[0].name_key.push_str("-changed")),
        ("color", |d| d.nations[0].color[0] += 1),
        ("capital", |d| d.nations[0].capital = 20),
        ("government", |d| {
            d.nations[0].government_key.push_str("-changed")
        }),
        ("support", |d| {
            d.nations[0]
                .ideology_support
                .insert("ideology-test-civic".into(), "0.5".into());
            d.nations[0]
                .ideology_support
                .insert("ideology-test-local".into(), "0.5".into());
        }),
        ("owner", |d| {
            d.scenario.ownership.insert(1, "STH".into());
        }),
        ("controller", |d| {
            d.scenario.control_overrides.insert(10, "STH".into());
        }),
        ("population", |d| d.map.states[0].population += 1),
        ("resource", |d| {
            d.map.states[0].resources.insert("steel".into(), 3);
        }),
        ("building", |d| {
            d.map.states[0].buildings.insert("industry".into(), 2);
        }),
        ("base", |d| d.map.states[0].infrastructure += 1),
        ("modifier", |d| {
            d.scenario.state_modifiers.insert(
                1,
                vec![oh_data::national::ModifierInput {
                    source: "fixture-add".into(),
                    target_stat: "infrastructure".into(),
                    operation: "add".into(),
                    value: "0.5".into(),
                    expires: Some(2),
                }],
            );
        }),
    ];
    for (name, change) in cases {
        let mut d = original.clone();
        change(&mut d);
        assert_ne!(from_data(&d).state_hash().unwrap(), expected, "{name}");
    }
    let mut ordered = original.clone();
    ordered.nations.reverse();
    ordered.map.states.reverse();
    assert_eq!(from_data(&ordered).state_hash().unwrap(), expected);
    let mut sim = from_data(&original);
    sim.enqueue(10, oh_core::NationId(1), 2, oh_sim::Command::Pause(true))
        .unwrap();
    assert_ne!(sim.state_hash().unwrap(), expected);
    let mut other = from_data(&original);
    other
        .enqueue(10, oh_core::NationId(1), 2, oh_sim::Command::Pause(true))
        .unwrap();
    assert_eq!(other.state_hash().unwrap(), sim.state_hash().unwrap());
}
#[test]
fn m1_expiry_input_order_and_atomic_overflow_are_authoritative() {
    use oh_data::national::ModifierInput;
    let mut d = loaded();
    let add = ModifierInput {
        source: "fixture-add".into(),
        target_stat: "infrastructure".into(),
        operation: "add".into(),
        value: "0.5".into(),
        expires: Some(2),
    };
    let mul = ModifierInput {
        source: "fixture-mul".into(),
        operation: "mul".into(),
        value: "2".into(),
        ..add.clone()
    };
    d.scenario
        .state_modifiers
        .insert(1, vec![mul.clone(), add.clone()]);
    let mut sim = from_data(&d);
    assert_eq!(
        sim.world()
            .unwrap()
            .state(StateId(1))
            .unwrap()
            .infrastructure(),
        Fx::from_num(3)
    );
    let hash = sim.state_hash().unwrap();
    d.scenario.state_modifiers.get_mut(&1).unwrap().reverse();
    assert_eq!(from_data(&d).state_hash().unwrap(), hash);
    d.scenario.state_modifiers.get_mut(&1).unwrap()[0].expires = Some(3);
    assert_ne!(from_data(&d).state_hash().unwrap(), hash);
    sim.step().unwrap();
    sim.step().unwrap();
    let s = sim.world().unwrap().state(StateId(1)).unwrap();
    assert_eq!(s.infrastructure(), Fx::ONE);
    assert_eq!(s.ledger().entries().len(), 1);
    // Removing an expiring negative contribution causes overflow only at tick 2.
    let mut d = loaded();
    d.map.states[0].infrastructure = 1_500_000_000;
    d.scenario.state_modifiers.insert(
        1,
        vec![
            ModifierInput {
                value: "-1000000000".into(),
                ..add
            },
            ModifierInput {
                expires: None,
                ..mul
            },
        ],
    );
    let mut sim = from_data(&d);
    sim.step().unwrap();
    sim.enqueue(1, oh_core::NationId(1), 1, oh_sim::Command::SetSpeed(5))
        .unwrap();
    let before = sim.state_hash().unwrap();
    assert!(sim.step().is_err());
    assert_eq!(sim.state_hash().unwrap(), before);
    assert_eq!(sim.pending_commands().len(), 1);
}
