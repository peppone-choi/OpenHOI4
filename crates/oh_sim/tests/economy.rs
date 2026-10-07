use oh_core::{Fx, NationId, Qty};
use oh_sim::{Command, Date, Simulation, TimeConfig, world::World};
fn loaded() -> oh_data::national::LoadedNational {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut l = oh_data::national::load_scenario(&root, "m1").unwrap();
    l.scenario.economy = Some("synthetic".into());
    l.economy = Some(toml_definition());
    l
}
fn toml_definition() -> oh_data::economy::Definition {
    use oh_data::economy::*;
    let building = Building {
        name_key: "industry-name".into(),
        costs: vec!["3".into(), "20".into(), "40".into()],
        slots: vec![1, 1, 1],
        daily_cap: "6".into(),
        ic_per_level: "10".into(),
        infrastructure_factors: vec![
            InfrastructureFactor {
                infrastructure: "0".into(),
                factor: "1".into(),
            },
            InfrastructureFactor {
                infrastructure: "1".into(),
                factor: "1".into(),
            },
            InfrastructureFactor {
                infrastructure: "1.125".into(),
                factor: "1".into(),
            },
        ],
        infrastructure_levels: None,
    };
    let law = Law {
        name_key: "law-name".into(),
        category: "economy".into(),
        step: 0,
        cost: "3".into(),
        condition: oh_data::trigger::Condition::DateGte("1900-01-01".into()),
        ic_multiplier: "1".into(),
        conscription_ratio: "0.125".into(),
        consumer_base: "0".into(),
        instability_slope: "0".into(),
    };
    let mut second = law.clone();
    second.step = 1;
    second.consumer_base = "0.5".into();
    let nation = NationInput {
        laws: std::collections::BTreeMap::from([("economy".into(), "civil".into())]),
        political_capital: "0".into(),
        political_capital_cap: "100".into(),
        political_capital_daily: "3".into(),
        stability: "0.5".into(),
        mobilization: "0.5".into(),
        allocation: std::array::from_fn(|_| "0.25".into()),
        ic_multiplier: "1".into(),
        committed: 60,
        reserved: 10,
    };
    Definition {
        buildings: std::collections::BTreeMap::from([("industry".into(), building)]),
        laws: std::collections::BTreeMap::from([("civil".into(), law), ("war".into(), second)]),
        economy_category: "economy".into(),
        conscription_category: "economy".into(),
        nations: std::collections::BTreeMap::from([(1, nation.clone()), (2, nation)]),
        state_slots: std::collections::BTreeMap::from([(1, 4), (2, 4)]),
    }
}
fn sim(l: &oh_data::national::LoadedNational) -> Simulation {
    Simulation::with_world(
        "m1".into(),
        Date::new(2000, 2, 28).unwrap(),
        1,
        TimeConfig::from_defines(&l.pack.defines).unwrap(),
        World::from_loaded(l).unwrap(),
    )
    .unwrap()
}
#[test]
fn req_eco_01_04_owned_state_contributions_resources_and_population() {
    let l = loaded();
    let mut s = sim(&l);
    let ledger = s.economy().unwrap().nation(NationId(1)).unwrap().ledger();
    assert_eq!(ledger.total_ic, Qty::from_num(10));
    assert_eq!(ledger.population, 1000);
    assert_eq!(ledger.resources["steel"], 2);
    assert_eq!(ledger.contributions.len(), 1);
    for _ in 0..24 {
        s.step().unwrap();
    }
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .ledger()
            .tick,
        24
    );
}
#[test]
fn req_nat_02_boundary_command_does_not_spend_future_pc_and_pause_does_not_pay() {
    let mut s = sim(&loaded());
    // cost3 cannot be previewed while PC0. Future income is unavailable at enqueue.
    assert!(
        s.enqueue(
            23,
            NationId(1),
            1,
            Command::Economy(oh_sim::economy::Action::ChangeLaw { law: "war".into() })
        )
        .is_err()
    );
    for _ in 0..23 {
        s.step().unwrap();
    }
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .political_capital(),
        Qty::ZERO
    );
    s.step().unwrap();
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .political_capital(),
        Qty::from_num(3)
    );
    s.enqueue(24, NationId(1), 2, Command::Pause(true)).unwrap();
    s.step().unwrap();
    s.step().unwrap();
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .political_capital(),
        Qty::from_num(3)
    );
}
#[test]
fn req_nat_03_04_law_transaction_minimum_and_pc_are_atomic() {
    let mut s = sim(&loaded());
    for _ in 0..24 {
        s.step().unwrap();
    }
    let before = oh_core::canonical_bytes(&s).unwrap();
    assert!(
        s.enqueue(
            24,
            NationId(1),
            1,
            Command::Economy(oh_sim::economy::Action::ChangeLaw { law: "war".into() })
        )
        .is_err()
    );
    assert_eq!(before, oh_core::canonical_bytes(&s).unwrap());
    s.enqueue(
        24,
        NationId(1),
        2,
        Command::Economy(oh_sim::economy::Action::Allocate {
            ratios: [Fx::from_num(0.5), Fx::from_num(0.5), Fx::ZERO, Fx::ZERO],
        }),
    )
    .unwrap();
    s.step().unwrap();
    s.enqueue(
        25,
        NationId(1),
        3,
        Command::Economy(oh_sim::economy::Action::ChangeLaw { law: "war".into() }),
    )
    .unwrap();
    s.step().unwrap();
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .political_capital(),
        Qty::ZERO
    );
    assert!(
        s.enqueue(
            26,
            NationId(1),
            4,
            Command::Economy(oh_sim::economy::Action::ChangeLaw { law: "war".into() })
        )
        .is_err()
    );
}
#[test]
fn req_eco_06_queue_stage_reservation_cancel_and_completion_not_retroactive() {
    let mut s = sim(&loaded());
    let action = oh_sim::economy::Action::Construct {
        project: 7,
        state: 1,
        building: "industry".into(),
    };
    s.enqueue(0, NationId(1), 1, Command::Economy(action.clone()))
        .unwrap();
    s.step().unwrap();
    assert!(
        s.enqueue(
            1,
            NationId(1),
            2,
            Command::Economy(oh_sim::economy::Action::Construct {
                project: 8,
                state: 1,
                building: "industry".into()
            })
        )
        .is_err()
    );
    s.enqueue(
        1,
        NationId(1),
        3,
        Command::Economy(oh_sim::economy::Action::Cancel { project: 7 }),
    )
    .unwrap();
    s.step().unwrap();
    s.enqueue(2, NationId(1), 4, Command::Economy(action))
        .unwrap();
    s.step().unwrap();
    for _ in 3..24 {
        s.step().unwrap();
    }
    assert_eq!(
        s.economy().unwrap().nation(NationId(1)).unwrap().projects()[0].progress,
        Qty::from_num(2.5)
    );
}
#[test]
fn req_eco_07_capacity_reserved_lifetime_and_no_daily_duplication() {
    let mut s = sim(&loaded());
    let n = s.economy().unwrap().nation(NationId(1)).unwrap();
    assert_eq!(
        (n.capacity(), n.available(), n.overcommitted()),
        (125, 55, 0)
    );
    for _ in 0..48 {
        s.step().unwrap();
    }
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .available(),
        55
    );
}
