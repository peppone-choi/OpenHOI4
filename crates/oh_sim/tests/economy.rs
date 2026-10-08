use oh_core::{Fx, NationId, Qty};
use oh_sim::{Command, Date, Simulation, TimeConfig, world::World};
#[path = "support/economy_fixture.rs"]
mod fixture;
use fixture::loaded;
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
// Separate authoritative save: optional trigger/movement and the entire economy.
#[test]
fn req_eco_nat_v5_full_state_future_queue_and_legacy_mode_are_preserved() {
    let l = loaded();
    let mut s = sim(&l);
    s.enqueue(
        0,
        NationId(1),
        1,
        Command::Economy(oh_sim::economy::Action::Construct {
            project: 91,
            state: 1,
            building: "industry".into(),
        }),
    )
    .unwrap();
    s.step().unwrap();
    for _ in 1..24 {
        s.step().unwrap();
    }
    s.enqueue(
        99,
        NationId(1),
        2,
        Command::Economy(oh_sim::economy::Action::Cancel { project: 91 }),
    )
    .unwrap();
    let dto = s.export_save_v5().unwrap();
    let context = oh_sim::save_state::RestoreContext {
        scenario: "m1".into(),
        start_date: Date::new(2000, 2, 28).unwrap(),
        world: Some(World::from_loaded(&l).unwrap()),
    };
    let mut restored = Simulation::from_save_v5(dto.clone(), &context).unwrap();
    assert_eq!(dto, restored.export_save_v5().unwrap());
    assert_eq!(
        oh_core::canonical_bytes(&s).unwrap(),
        oh_core::canonical_bytes(&restored).unwrap()
    );
    for _ in 0..48 {
        s.step().unwrap();
        restored.step().unwrap();
    }
    assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
    assert!(s.export_save().is_err());
    assert!(s.export_save_v4().is_err());
    let mut missing = context.clone();
    missing.world = Some(
        World::from_loaded(
            &oh_data::national::load_scenario(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
                "m1",
            )
            .unwrap(),
        )
        .unwrap(),
    );
    assert!(Simulation::from_save_v5(dto, &missing).is_err());
}
#[test]
fn req_eco_07_internal_manpower_reserve_commit_cancel_return_and_capacity_drop() {
    use oh_sim::economy::ManpowerOperation::*;
    let l = loaded();
    let mut e = sim(&l).economy().unwrap().clone();
    e.manpower(NationId(1), Reserve, 20).unwrap();
    e.manpower(NationId(1), CommitReservation, 20).unwrap();
    e.manpower(NationId(1), ReturnCommitted, 10).unwrap();
    let n = e.nation(NationId(1)).unwrap();
    assert_eq!((n.committed(), n.reserved(), n.available()), (70, 10, 45));
    let before = oh_core::canonical_bytes(&e).unwrap();
    for (op, amount) in [
        (Reserve, 46),
        (ReturnCommitted, 71),
        (CancelReservation, 11),
        (Consume, -1),
    ] {
        assert!(e.manpower(NationId(1), op, amount).is_err());
        assert_eq!(before, oh_core::canonical_bytes(&e).unwrap());
    }
    let mut low = l.clone();
    low.economy
        .as_mut()
        .unwrap()
        .laws
        .get_mut("civil")
        .unwrap()
        .conscription_ratio = "0.0625".into();
    let low = sim(&low);
    let n = low.economy().unwrap().nation(NationId(1)).unwrap();
    assert_eq!(
        (
            n.capacity(),
            n.committed(),
            n.reserved(),
            n.available(),
            n.overcommitted()
        ),
        (62, 60, 10, 0, 8)
    );
}
#[test]
fn req_eco_06_completion_next_day_output_and_zero_cap_skip() {
    let mut l = loaded();
    l.economy
        .as_mut()
        .unwrap()
        .buildings
        .get_mut("industry")
        .unwrap()
        .costs[1] = "2".into();
    let mut s = sim(&l);
    s.enqueue(
        0,
        NationId(1),
        1,
        Command::Economy(oh_sim::economy::Action::Construct {
            project: 1,
            state: 1,
            building: "industry".into(),
        }),
    )
    .unwrap();
    for _ in 0..24 {
        s.step().unwrap();
    }
    let n = s.economy().unwrap().nation(NationId(1)).unwrap();
    assert!(n.projects().is_empty());
    assert_eq!(n.ledger().total_ic, Qty::from_num(10));
    assert_eq!(
        n.construction_ledger().entries[0].consumed,
        Qty::from_num(2)
    );
    assert_eq!(n.construction_ledger().unused, Qty::from_num(0.5));
    assert_eq!(
        s.world()
            .unwrap()
            .state(oh_core::StateId(1))
            .unwrap()
            .buildings()["industry"],
        2
    );
    for _ in 0..24 {
        s.step().unwrap();
    }
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .ledger()
            .total_ic,
        Qty::from_num(20)
    );
    l.economy
        .as_mut()
        .unwrap()
        .buildings
        .get_mut("industry")
        .unwrap()
        .daily_cap = "0".into();
    let mut s = sim(&l);
    s.enqueue(
        0,
        NationId(1),
        1,
        Command::Economy(oh_sim::economy::Action::Construct {
            project: 2,
            state: 1,
            building: "industry".into(),
        }),
    )
    .unwrap();
    for _ in 0..24 {
        s.step().unwrap();
    }
    let n = s.economy().unwrap().nation(NationId(1)).unwrap();
    assert_eq!(n.projects()[0].progress, Qty::ZERO);
    assert_eq!(
        n.projects()[0].dormancy,
        Some(oh_sim::economy::Dormancy::ZeroCap)
    );
    assert_eq!(n.construction_ledger().unused, Qty::from_num(2.5));
}
#[test]
fn req_nat_02_boundary_scheduled_semantic_error_consumed_before_pc_income() {
    let mut l = loaded();
    l.economy
        .as_mut()
        .unwrap()
        .nations
        .get_mut(&1)
        .unwrap()
        .political_capital = "3".into();
    l.economy
        .as_mut()
        .unwrap()
        .laws
        .get_mut("war")
        .unwrap()
        .consumer_base = "0".into();
    l.scenario.effect_programs = Some(std::collections::BTreeMap::from([(
        "drain".into(),
        oh_data::trigger::EffectProgram {
            root: Some("NTH".into()),
            effects: vec![oh_data::trigger::Effect::AddPoliticalCapital("-3".into())],
        },
    )]));
    let mut s = sim(&l);
    s.enqueue(
        23,
        NationId(1),
        1,
        Command::Effects {
            program: "drain".into(),
        },
    )
    .unwrap();
    s.enqueue(
        23,
        NationId(1),
        2,
        Command::Economy(oh_sim::economy::Action::ChangeLaw { law: "war".into() }),
    )
    .unwrap();
    for _ in 0..23 {
        s.step().unwrap();
    }
    let step = s.step().unwrap();
    assert!(step.commands[0].result.is_ok());
    assert!(step.commands[1].result.is_err());
    assert!(s.pending_commands().is_empty());
    let n = s.economy().unwrap().nation(NationId(1)).unwrap();
    assert_eq!(n.political_capital(), Qty::from_num(3));
    assert_eq!(n.laws()["economy"], "civil");
    assert_eq!(s.snapshot().tick(), 24);
}
#[test]
fn req_eco_nat_wholephase_overflow_restores_state_clock_and_all_queue() {
    let mut l = loaded();
    let b = l
        .economy
        .as_mut()
        .unwrap()
        .buildings
        .get_mut("industry")
        .unwrap();
    b.ic_per_level = "140737488355327".into();
    b.costs[1] = "2".into();
    let mut s = sim(&l);
    s.enqueue(
        0,
        NationId(1),
        1,
        Command::Economy(oh_sim::economy::Action::Construct {
            project: 1,
            state: 1,
            building: "industry".into(),
        }),
    )
    .unwrap();
    for _ in 0..47 {
        s.step().unwrap();
    }
    s.enqueue(47, NationId(1), 2, Command::SetSpeed(5)).unwrap();
    s.enqueue(99, NationId(1), 3, Command::Pause(true)).unwrap();
    let before = oh_core::canonical_bytes(&s).unwrap();
    let hash = s.state_hash().unwrap();
    assert!(s.step().is_err());
    assert_eq!(before, oh_core::canonical_bytes(&s).unwrap());
    assert_eq!(hash, s.state_hash().unwrap());
    assert_eq!(s.pending_commands().len(), 2);
    assert_eq!(s.snapshot().tick(), 47);
}
#[test]
fn req_nat_03_effect_transaction_rolls_back_flags_pc_and_all_politics() {
    let mut l = loaded();
    l.scenario.flag_keys = Some(vec!["x".into()]);
    l.scenario.effect_programs = Some(std::collections::BTreeMap::from([(
        "bad".into(),
        oh_data::trigger::EffectProgram {
            root: Some("NTH".into()),
            effects: vec![
                oh_data::trigger::Effect::SetFlag("x".into()),
                oh_data::trigger::Effect::AddPoliticalCapital("3".into()),
                oh_data::trigger::Effect::AddStability("0.75".into()),
            ],
        },
    )]));
    let mut s = sim(&l);
    let before = oh_core::canonical_bytes(&s).unwrap();
    assert!(
        s.enqueue(
            0,
            NationId(1),
            1,
            Command::Effects {
                program: "bad".into()
            }
        )
        .is_err()
    );
    assert_eq!(before, oh_core::canonical_bytes(&s).unwrap());
    assert!(s.trigger_state().unwrap().flags[0].1.is_empty());
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .political_capital(),
        Qty::ZERO
    );
}
#[test]
fn req_eco_01_real_industry_end_score_preserves_input_tick_and_qty() {
    let mut l = loaded();
    l.scenario.end_conditions = Some(oh_data::trigger::Condition::Stability(
        oh_data::trigger::Compare::Gte("0.5".into()),
    ));
    l.scenario.end_root = Some("NTH".into());
    l.scenario.score_weights = Some(oh_data::trigger::ScoreWeights {
        victory_points: "0".into(),
        industrial_capacity: "1.5".into(),
        survival: "0".into(),
        faction_victory: "0".into(),
    });
    let s = sim(&l);
    assert!(s.is_ended());
    let score = &s.economy().unwrap().industrial_scores().unwrap()[&1];
    assert_eq!((score.tick, score.input_tick), (0, 0));
    assert_eq!(score.input, Qty::from_num(10));
    assert_eq!(score.term, Qty::from_num(15));
    let context = oh_sim::save_state::RestoreContext {
        scenario: "m1".into(),
        start_date: Date::new(2000, 2, 28).unwrap(),
        world: Some(World::from_loaded(&l).unwrap()),
    };
    let restored = Simulation::from_save_v5(s.export_save_v5().unwrap(), &context).unwrap();
    assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
    l.scenario.score_weights.as_mut().unwrap().survival = "1".into();
    assert!(World::from_loaded(&l).is_err());
}
#[test]
fn additive_empty_economy_presence_is_v5_and_has_no_zero_producers() {
    let mut l = loaded();
    let d = l.economy.as_mut().unwrap();
    d.buildings.clear();
    d.laws.clear();
    d.nations.clear();
    d.state_slots.clear();
    d.economy_category.clear();
    d.conscription_category.clear();
    let s = sim(&l);
    assert!(s.economy().is_some());
    assert!(s.economy().unwrap().nations().is_empty());
    let context = oh_sim::save_state::RestoreContext {
        scenario: "m1".into(),
        start_date: Date::new(2000, 2, 28).unwrap(),
        world: Some(World::from_loaded(&l).unwrap()),
    };
    let mut r = Simulation::from_save_v5(s.export_save_v5().unwrap(), &context).unwrap();
    assert_eq!(s.state_hash().unwrap(), r.state_hash().unwrap());
    assert!(
        r.enqueue(
            0,
            NationId(1),
            1,
            Command::Economy(oh_sim::economy::Action::ChangeLaw {
                law: "civil".into()
            })
        )
        .is_err()
    );
    l.scenario.end_conditions = Some(oh_data::trigger::Condition::Stability(
        oh_data::trigger::Compare::Eq("0".into()),
    ));
    l.scenario.end_root = Some("NTH".into());
    assert!(World::from_loaded(&l).is_err());
}
