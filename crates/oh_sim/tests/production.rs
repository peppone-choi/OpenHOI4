use oh_core::{Fx, NationId, Qty};
use oh_sim::{Command, Date, Simulation, TimeConfig, production::Action, world::World};
#[path = "support/economy_fixture.rs"]
mod economy_fixture;

fn sim() -> Simulation {
    let mut l = economy_fixture::loaded();
    l.production = Some(fixture_definition());
    let world = World::from_loaded(&l).unwrap();
    Simulation::with_world(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&l.pack.defines).unwrap(),
        world,
    )
    .unwrap()
}
fn day(s: &mut Simulation) {
    for _ in 0..24 {
        s.step().unwrap();
    }
}
fn create(s: &mut Simulation, seq: u64, model: &str, ic: Qty) {
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        seq,
        Command::Production(Action::Create {
            model: model.into(),
            requested_ic: ic,
        }),
    )
    .unwrap();
}
#[test]
fn req_eco_03_fractional_carry_pre_growth_and_actual_productive_days() {
    let mut s = sim();
    create(&mut s, 1, "test_model_1", Qty::from_num(1));
    day(&mut s);
    let p = s.production().unwrap();
    let l = p.line(0).unwrap();
    assert_eq!(l.carry(), Qty::from_num(0.25));
    assert_eq!(l.efficiency(), Fx::from_num(0.2578125));
    assert_eq!(p.stock(NationId(1), "test_model_1"), Some(0));
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        2,
        Command::Production(Action::Pause {
            line: 0,
            paused: true,
        }),
    )
    .unwrap();
    day(&mut s);
    assert_eq!(
        s.production().unwrap().line(0).unwrap().carry(),
        Qty::from_num(0.25)
    );
    assert_eq!(
        s.production().unwrap().line(0).unwrap().efficiency(),
        Fx::from_num(0.2578125)
    );
}
#[test]
fn req_mil_01_whole_stock_and_same_model_preservation_different_model_discard() {
    let mut s = sim();
    create(&mut s, 1, "test_model_1", Qty::from_num(1));
    day(&mut s);
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        2,
        Command::Production(Action::Switch {
            line: 0,
            model: "test_model_1".into(),
        }),
    )
    .unwrap();
    s.step().unwrap();
    assert_eq!(
        s.production().unwrap().line(0).unwrap().carry(),
        Qty::from_num(0.25)
    );
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        3,
        Command::Production(Action::Switch {
            line: 0,
            model: "test_model_2".into(),
        }),
    )
    .unwrap();
    s.step().unwrap();
    assert_eq!(s.production().unwrap().line(0).unwrap().carry(), Qty::ZERO);
    assert_eq!(s.production().unwrap().discards().len(), 1);
    assert_eq!(
        s.production().unwrap().stock(NationId(1), "test_model_1"),
        Some(0)
    );
}
#[test]
fn req_eco_03_invalid_settings_do_not_mutate_state_or_queue() {
    let mut s = sim();
    let before = s.state_hash().unwrap();
    assert!(
        s.enqueue(
            0,
            NationId(1),
            1,
            Command::Production(Action::Create {
                model: "missing".into(),
                requested_ic: Qty::ZERO
            })
        )
        .is_err()
    );
    assert!(
        s.enqueue(
            0,
            NationId(1),
            2,
            Command::Production(Action::Create {
                model: "test_model_1".into(),
                requested_ic: Qty::MAX
            })
        )
        .is_err()
    );
    assert_eq!(before, s.state_hash().unwrap());
    assert!(s.pending_commands().is_empty());
}
#[test]
fn req_eco_03_two_fresh_runs_have_identical_production_hashes() {
    let mut a = sim();
    let mut b = sim();
    create(&mut a, 1, "test_model_1", Qty::from_num(1));
    create(&mut b, 1, "test_model_1", Qty::from_num(1));
    for _ in 0..100 {
        day(&mut a);
        day(&mut b);
    }
    assert_eq!(a.state_hash().unwrap(), b.state_hash().unwrap());
    assert!(
        a.production()
            .unwrap()
            .stock(NationId(1), "test_model_1")
            .unwrap()
            > 0
    );
}

fn fixture_definition() -> oh_data::production::Definition {
    use oh_data::production::{Definition, Model, NationInput, Tuning};
    use std::collections::{BTreeMap, BTreeSet};
    let m = Model {
        name_key: "test-model-1".into(),
        family: "test_family".into(),
        generation: 1,
        unit_cost: "1".into(),
        resources_per_item: BTreeMap::new(),
    };
    let mut m2 = m.clone();
    m2.generation = 2;
    m2.name_key = "test-model-2".into();
    let n = NationInput {
        allowed_models: BTreeSet::from(["test_model_1".into(), "test_model_2".into()]),
        stock: BTreeMap::from([("test_model_1".into(), 0), ("test_model_2".into(), 0)]),
    };
    Definition {
        models: BTreeMap::from([("test_model_1".into(), m), ("test_model_2".into(), m2)]),
        nations: BTreeMap::from([(1, n.clone()), (2, n)]),
        tuning: Some(Tuning {
            initial_efficiency: Fx::from_num(0.25),
            efficiency_cap: Fx::ONE,
            daily_efficiency_growth: Fx::from_num(0.0078125),
            same_family_newer_retention: Fx::from_num(0.75),
            other_retention: Fx::from_num(0.25),
            stability_output_low: Fx::from_num(0.5),
            stability_output_high: Fx::from_num(1.5),
            inventory_count_limit: i64::MAX,
            lines_max: 1024,
        }),
    }
}

fn from_definition(d: oh_data::production::Definition) -> Simulation {
    let mut l = economy_fixture::loaded();
    l.production = Some(d);
    let w = World::from_loaded(&l).unwrap();
    Simulation::with_world(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&l.pack.defines).unwrap(),
        w,
    )
    .unwrap()
}
#[test]
fn req_eco_03_shared_resources_are_proportional_with_actual_raw_debits() {
    let mut d = fixture_definition();
    d.models
        .get_mut("test_model_1")
        .unwrap()
        .resources_per_item
        .insert("steel".into(), "8".into());
    let mut s = from_definition(d);
    create(&mut s, 1, "test_model_1", Qty::from_num(1));
    s.step().unwrap();
    create(&mut s, 2, "test_model_1", Qty::from_num(1));
    while s.snapshot().tick() < 24 {
        s.step().unwrap();
    }
    let entries = &s.production().unwrap().day().unwrap().nations[&1].lines;
    for l in entries.values() {
        assert_eq!(l.fulfillment, Fx::from_num(0.5));
        assert_eq!(l.actual, Qty::from_num(0.125));
        assert_eq!(l.resources["steel"].debited, Qty::ONE);
        assert_eq!(l.ending_efficiency, Fx::from_num(0.2578125));
    }
}
#[test]
fn req_eco_03_budget_decrease_scales_existing_requests_without_resetting_them() {
    let mut s = sim();
    create(&mut s, 1, "test_model_1", Qty::ONE);
    s.step().unwrap();
    create(&mut s, 2, "test_model_1", Qty::ONE);
    s.step().unwrap();
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        3,
        Command::Economy(oh_sim::economy::Action::Allocate {
            ratios: [
                Fx::from_num(0.5),
                Fx::from_num(0.375),
                Fx::from_num(0.125),
                Fx::ZERO,
            ],
        }),
    )
    .unwrap();
    while s.snapshot().tick() < 24 {
        s.step().unwrap();
    }
    let p = s.production().unwrap();
    for line in p.lines().values() {
        assert_eq!(line.requested_ic(), Qty::ONE);
    }
    for entry in p.day().unwrap().nations[&1].lines.values() {
        assert_eq!(entry.effective_ic, Qty::from_num(0.625));
    }
}
#[test]
fn req_eco_03_count_overflow_rolls_back_entire_step_clock_queue_and_daily_ledgers() {
    let s = sim();
    let mut dto = s.export_save_v6().unwrap();
    dto.production
        .adjust_stock(s.world().unwrap(), NationId(1), "test_model_1", i64::MAX)
        .unwrap();
    let context = oh_sim::save_state::RestoreContext {
        scenario: "m1".into(),
        start_date: s.snapshot().date(),
        world: s.world().cloned(),
    };
    let mut s = Simulation::from_save_v6(dto, &context).unwrap();
    create(&mut s, 1, "test_model_1", Qty::from_num(2));
    for _ in 0..47 {
        s.step().unwrap();
    }
    s.enqueue(
        47,
        NationId(1),
        2,
        Command::Production(Action::SetIC {
            line: 0,
            requested_ic: Qty::from_num(2),
        }),
    )
    .unwrap();
    let bytes = oh_core::canonical_bytes(&s).unwrap();
    assert!(s.step().is_err());
    assert_eq!(bytes, oh_core::canonical_bytes(&s).unwrap());
    assert_eq!(s.snapshot().tick(), 47);
    assert_eq!(s.pending_commands().len(), 1);
}
#[test]
fn req_eco_03_zero_actual_output_does_not_grow_and_sensitivity_reaches_defined_caps() {
    let mut d = fixture_definition();
    d.models
        .get_mut("test_model_1")
        .unwrap()
        .resources_per_item
        .insert("steel".into(), "100000000".into());
    let mut s = from_definition(d);
    create(&mut s, 1, "test_model_1", Qty::ONE);
    day(&mut s);
    assert_eq!(
        s.production().unwrap().line(0).unwrap().efficiency(),
        Fx::from_num(0.25)
    );
    for (growth, days) in [(0.00390625, 192), (0.0078125, 96), (0.015625, 48)] {
        let mut d = fixture_definition();
        d.tuning.as_mut().unwrap().daily_efficiency_growth = Fx::from_num(growth);
        let mut s = from_definition(d);
        create(&mut s, 1, "test_model_1", Qty::ONE);
        for _ in 0..days {
            day(&mut s)
        }
        assert_eq!(
            s.production().unwrap().line(0).unwrap().efficiency(),
            Fx::ONE
        );
    }
}

#[test]
fn req_mil_01_retention_and_cancel_keep_completed_inventory() {
    let mut s = sim();
    create(&mut s, 1, "test_model_1", Qty::ONE);
    for _ in 0..96 {
        day(&mut s);
    }
    let completed = s
        .production()
        .unwrap()
        .stock(NationId(1), "test_model_1")
        .unwrap();
    assert!(completed > 0);
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        2,
        Command::Production(Action::Switch {
            line: 0,
            model: "test_model_2".into(),
        }),
    )
    .unwrap();
    s.step().unwrap();
    assert_eq!(
        s.production().unwrap().line(0).unwrap().efficiency(),
        Fx::from_num(0.75)
    );
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        3,
        Command::Production(Action::Switch {
            line: 0,
            model: "test_model_1".into(),
        }),
    )
    .unwrap();
    s.step().unwrap();
    assert_eq!(
        s.production().unwrap().line(0).unwrap().efficiency(),
        Fx::from_num(0.25)
    );
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        4,
        Command::Production(Action::Cancel { line: 0 }),
    )
    .unwrap();
    s.step().unwrap();
    assert_eq!(
        s.production().unwrap().stock(NationId(1), "test_model_1"),
        Some(completed)
    );
    assert!(s.production().unwrap().lines().is_empty());
}
#[test]
fn req_eco_03_raw_proportion_residuals_and_formula_range_controls() {
    use oh_sim::formula::*;
    let shares = production_split(5, &[(0, 1), (1, 1), (2, 1)]).unwrap();
    assert_eq!(shares.values().sum::<i128>(), 5);
    assert_eq!(shares[&0], 2);
    assert_eq!(shares[&2], 1);
    assert!(production_split(5, &[(1, 1), (0, 1)]).is_err());
    assert!(production_divide(Qty::ONE, Qty::ZERO).is_err());
    assert!(production_resource(Qty::MAX, Qty::MAX).is_err());
    assert!(production_ratio(-1, 1).is_err());
    assert_eq!(
        production_resource(Qty::from_bits(1), Qty::from_bits(1))
            .unwrap()
            .to_bits(),
        1
    );
    assert_eq!(production_ratio(1, 2).unwrap(), Fx::from_num(0.5));
    // 1024 admitted i64 raw demands: the direct product exceeds i128, while
    // each share and the conserved aggregate are representable.
    let weights: Vec<_> = (0..1024).map(|id| (id, i128::from(i64::MAX))).collect();
    let total = weights.iter().map(|(_, w)| w).sum::<i128>();
    let shares = production_split(total - 1, &weights).unwrap();
    assert_eq!(shares.values().sum::<i128>(), total - 1);
    assert_eq!(shares[&0], i128::from(i64::MAX));
    assert_eq!(shares[&1023], i128::from(i64::MAX) - 1);
    for a in 0..20 {
        for b in 0..20 {
            let weights = [(0, b), (1, 20 - b)];
            let split = production_split(a, &weights).unwrap();
            assert_eq!(split.values().sum::<i128>(), a);
            let floor = a * b / 20;
            assert!((floor..=floor + 1).contains(&split[&0]));
        }
    }
}

#[test]
fn req_eco_03_multi_resource_minimum_has_no_hidden_redistribution() {
    use std::collections::BTreeMap;
    let mut s = sim();
    create(&mut s, 1, "test_model_1", Qty::ONE);
    s.step().unwrap();
    create(&mut s, 2, "test_model_2", Qty::ONE);
    s.step().unwrap();
    let mut d = fixture_definition();
    for model in d.models.values_mut() {
        model.resources_per_item.insert("steel".into(), "8".into());
    }
    d.models
        .get_mut("test_model_1")
        .unwrap()
        .resources_per_item
        .insert("oil".into(), "1".into());
    // Pure formula input deliberately declares both resources; the strict pack
    // loader separately checks that actual game resource IDs are registered.
    let result = oh_sim::production::calculate_day(
        &d,
        s.production().unwrap().lines(),
        Qty::from_num(2),
        &BTreeMap::from([("steel".into(), 2), ("oil".into(), 0)]),
        Fx::from_num(0.5),
    )
    .unwrap();
    assert_eq!(result.lines[&0].fulfillment, Fx::ZERO);
    assert_eq!(result.lines[&0].actual, Qty::ZERO);
    assert_eq!(result.lines[&0].ending_efficiency, Fx::from_num(0.25));
    assert_eq!(result.lines[&1].fulfillment, Fx::from_num(0.5));
    assert_eq!(result.lines[&1].actual, Qty::from_num(0.125));
    assert_eq!(result.lines[&1].resources["steel"].debited, Qty::ONE);
    assert_eq!(result.lines[&0].resources["steel"].reserved, Qty::ONE);
    assert_eq!(result.lines[&0].resources["steel"].debited, Qty::ZERO);
}
