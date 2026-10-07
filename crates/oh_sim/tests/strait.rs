use oh_core::{DivisionId, Fx, ProvinceId};
use oh_sim::{
    Command, Date, Simulation, TimeConfig,
    movement::{CrossingKind, Movement, StraitContext, StraitFactors, StraitInput, UnitInput},
};
use std::collections::{BTreeMap, BTreeSet};
fn sim() -> Simulation {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut d = oh_data::national::load_scenario(&root, "m1").unwrap();
    d.map.edges = vec![oh_data::map::Edge {
        a: 10,
        b: 40,
        kind: oh_data::map::EdgeKind::Strait,
        distance_km: Fx::from_num(10),
    }];
    let w = oh_sim::world::World::from_loaded(&d).unwrap();
    let n = w.inputs().nations()[0].id();
    let input = UnitInput {
        id: DivisionId(900),
        nation: n,
        province: ProvinceId(10),
        speed: Fx::from_num(2),
        allowed: BTreeSet::from([ProvinceId(10), ProvinceId(40)]),
        corrections: BTreeMap::new(),
    };
    let factors = StraitContext {
        kind: CrossingKind::Strait,
        factors: StraitFactors {
            terrain: Fx::ONE,
            infrastructure: Fx::ONE,
            supply: Fx::ONE,
            strait: Fx::from_num(2),
        },
    };
    let context = StraitInput {
        unit: DivisionId(900),
        corrections: BTreeMap::from([((ProvinceId(10), ProvinceId(40)), factors)]),
    };
    let m = Movement::with_straits(&w, vec![input], vec![context]).unwrap();
    Simulation::with_movement(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&d.pack.defines).unwrap(),
        w,
        m,
    )
    .unwrap()
}
#[test]
fn req_mil_04_strait_explicit_slot_9_10_tick_boundary() {
    let mut s = sim();
    let n = s.movement().unwrap().units()[0].nation();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(900),
            destination: ProvinceId(40),
        },
    )
    .unwrap();
    for _ in 0..9 {
        s.step().unwrap();
    }
    let u = &s.movement().unwrap().units()[0];
    assert_eq!(u.province(), ProvinceId(10));
    assert_eq!(u.elapsed().to_bits(), 38654705664);
    assert_eq!(u.remaining_hours().unwrap(), Fx::ONE);
    s.step().unwrap();
    let u = &s.movement().unwrap().units()[0];
    assert_eq!(u.province(), ProvinceId(40));
    assert_eq!(u.elapsed(), Fx::ZERO);
    assert!(u.remaining_route().is_empty());
}
fn inputs() -> (oh_data::national::LoadedNational, UnitInput, StraitInput) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut d = oh_data::national::load_scenario(&root, "m1").unwrap();
    d.map.edges = vec![oh_data::map::Edge {
        a: 10,
        b: 40,
        kind: oh_data::map::EdgeKind::Strait,
        distance_km: Fx::from_num(10),
    }];
    let unit = UnitInput {
        id: DivisionId(900),
        nation: oh_core::NationId(d.nations[0].id),
        province: ProvinceId(10),
        speed: Fx::from_num(2),
        allowed: BTreeSet::from([ProvinceId(10), ProvinceId(40)]),
        corrections: BTreeMap::new(),
    };
    let value = StraitContext {
        kind: CrossingKind::Strait,
        factors: StraitFactors {
            terrain: Fx::ONE,
            infrastructure: Fx::ONE,
            supply: Fx::ONE,
            strait: Fx::from_num(2),
        },
    };
    let ctx = StraitInput {
        unit: DivisionId(900),
        corrections: BTreeMap::from([((ProvinceId(10), ProvinceId(40)), value)]),
    };
    (d, unit, ctx)
}
fn build(
    d: &oh_data::national::LoadedNational,
    unit: UnitInput,
    ctx: StraitInput,
) -> Result<Simulation, oh_sim::movement::MovementError> {
    let w = oh_sim::world::World::from_loaded(d).unwrap();
    let m = Movement::with_straits(&w, vec![unit], vec![ctx])?;
    Ok(Simulation::with_movement(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&d.pack.defines).unwrap(),
        w,
        m,
    )
    .unwrap())
}
#[test]
fn req_mil_04_strait_alternate8_and_equal10_path_lexical() {
    use oh_data::map::{Edge, EdgeKind};
    use oh_sim::movement::Factors;
    for distance in [8, 10] {
        let (mut d, mut u, c) = inputs();
        d.map.edges.extend([
            Edge {
                a: 10,
                b: 20,
                kind: EdgeKind::Normal,
                distance_km: Fx::from_num(distance),
            },
            Edge {
                a: 20,
                b: 40,
                kind: EdgeKind::Normal,
                distance_km: Fx::from_num(distance),
            },
        ]);
        d.map.edges.sort_by_key(|e| (e.a, e.b));
        u.allowed.insert(ProvinceId(20));
        for (a, b) in [(10, 20), (20, 10), (20, 40), (40, 20)] {
            u.corrections.insert(
                (ProvinceId(a), ProvinceId(b)),
                Factors {
                    terrain: Fx::ONE,
                    infrastructure: Fx::ONE,
                    supply: Fx::ONE,
                    river: Fx::ONE,
                },
            );
        }
        let mut s = build(&d, u, c).unwrap();
        assert_eq!(
            s.movement()
                .unwrap()
                .path(s.world().unwrap(), DivisionId(900), ProvinceId(40))
                .unwrap(),
            [ProvinceId(20), ProvinceId(40)]
        );
        let n = s.movement().unwrap().units()[0].nation();
        s.enqueue(
            0,
            n,
            1,
            Command::Move {
                unit: DivisionId(900),
                destination: ProvinceId(40),
            },
        )
        .unwrap();
        for _ in 0..distance {
            s.step().unwrap();
        }
        assert_eq!(s.movement().unwrap().units()[0].province(), ProvinceId(40));
    }
}
#[test]
fn req_mil_04_strait_invalid_kind_refs_allowed_values_and_duplicate_slots() {
    use oh_sim::movement::{Factors, MovementError as E};
    let (d, u, c) = inputs();
    for value in [Fx::ZERO, -Fx::ONE] {
        let mut c = c.clone();
        c.corrections.values_mut().next().unwrap().factors.strait = value;
        assert_eq!(build(&d, u.clone(), c).unwrap_err(), E::InvalidValue);
    }
    for kind in [
        CrossingKind::Normal,
        CrossingKind::RiverSmall,
        CrossingKind::RiverLarge,
    ] {
        let mut c = c.clone();
        c.corrections.values_mut().next().unwrap().kind = kind;
        assert_eq!(build(&d, u.clone(), c).unwrap_err(), E::InvalidKind);
    }
    let mut wrong = d.clone();
    wrong.map.edges[0].kind = oh_data::map::EdgeKind::RiverSmall;
    assert_eq!(
        build(&wrong, u.clone(), c.clone()).unwrap_err(),
        E::InvalidKind
    );
    let value = *c.corrections.values().next().unwrap();
    for (a, b) in [(10, 50), (10, 60), (10, u16::MAX), (20, 40)] {
        let mut c = c.clone();
        c.corrections = BTreeMap::from([((ProvinceId(a), ProvinceId(b)), value)]);
        assert!(build(&d, u.clone(), c).is_err());
    }
    let mut denied = u.clone();
    denied.allowed.remove(&ProvinceId(40));
    assert!(build(&d, denied, c.clone()).is_err());
    let mut duplicate = u.clone();
    duplicate.corrections.insert(
        (ProvinceId(10), ProvinceId(40)),
        Factors {
            terrain: Fx::ONE,
            infrastructure: Fx::ONE,
            supply: Fx::ONE,
            river: Fx::ONE,
        },
    );
    assert!(build(&d, duplicate, c.clone()).is_err());
    let w = oh_sim::world::World::from_loaded(&d).unwrap();
    assert!(Movement::with_straits(&w, vec![u.clone()], vec![c.clone(), c.clone()]).is_err());
    assert!(Movement::with_straits(&w, vec![u.clone()], vec![]).is_err());
    let mut unknown = c;
    unknown.unit = DivisionId(u32::MAX);
    assert!(Movement::with_straits(&w, vec![u], vec![unknown]).is_err());
}
#[test]
fn req_mil_04_strait_missing_direction_overflow_underflow_and_authority_atomic() {
    use oh_sim::movement::MovementError as E;
    let (d, u, c) = inputs();
    let mut missing = c.clone();
    missing.corrections.clear();
    let mut s = build(&d, u.clone(), missing).unwrap();
    let hash = s.state_hash().unwrap();
    assert_eq!(
        s.enqueue(
            0,
            u.nation,
            1,
            Command::Move {
                unit: u.id,
                destination: ProvinceId(40)
            }
        )
        .unwrap_err(),
        oh_sim::Error::Movement(E::MissingContext)
    );
    assert_eq!(s.state_hash().unwrap(), hash);
    let mut overflow = c.clone();
    overflow
        .corrections
        .values_mut()
        .next()
        .unwrap()
        .factors
        .strait = Fx::MAX;
    let mut s = build(&d, u.clone(), overflow).unwrap();
    let hash = s.state_hash().unwrap();
    assert_eq!(
        s.enqueue(
            0,
            u.nation,
            1,
            Command::Move {
                unit: u.id,
                destination: ProvinceId(40)
            }
        )
        .unwrap_err(),
        oh_sim::Error::Movement(E::Overflow)
    );
    assert_eq!(s.state_hash().unwrap(), hash);
    let mut tiny = d.clone();
    tiny.map.edges[0].distance_km = Fx::from_bits(1);
    let mut fast = u.clone();
    fast.speed = Fx::MAX;
    let mut s = build(&tiny, fast, c.clone()).unwrap();
    let hash = s.state_hash().unwrap();
    assert_eq!(
        s.enqueue(
            0,
            u.nation,
            1,
            Command::Move {
                unit: u.id,
                destination: ProvinceId(40)
            }
        )
        .unwrap_err(),
        oh_sim::Error::Movement(E::InvalidValue)
    );
    assert_eq!(s.state_hash().unwrap(), hash);
    let mut s = build(&d, u.clone(), c).unwrap();
    let hash = s.state_hash().unwrap();
    assert_eq!(
        s.enqueue(
            0,
            oh_core::NationId(u16::MAX),
            1,
            Command::Move {
                unit: u.id,
                destination: ProvinceId(40)
            }
        )
        .unwrap_err(),
        oh_sim::Error::Movement(E::NotOwner)
    );
    assert_eq!(s.state_hash().unwrap(), hash);
    s.enqueue(
        0,
        u.nation,
        1,
        Command::Move {
            unit: u.id,
            destination: ProvinceId(40),
        },
    )
    .unwrap();
    for _ in 0..10 {
        s.step().unwrap();
    }
    let hash = s.state_hash().unwrap();
    assert_eq!(
        s.enqueue(
            10,
            u.nation,
            2,
            Command::Move {
                unit: u.id,
                destination: ProvinceId(10)
            }
        )
        .unwrap_err(),
        oh_sim::Error::Movement(E::MissingContext)
    );
    assert_eq!(s.state_hash().unwrap(), hash);
}
#[test]
fn req_mil_04_strait_formula_raw_oracle_and_no_river_slot() {
    let d = 12345678901i64;
    let v = 3456789012i64;
    let fs = [5678901234i64, 4567890123, 3456789012, 2345678901];
    let mut oracle = ((i128::from(d) << 32) / i128::from(v)) as i64;
    for f in fs {
        oracle = ((i128::from(oracle) * i128::from(f)) >> 32) as i64;
    }
    let f = StraitFactors {
        terrain: Fx::from_bits(fs[0]),
        infrastructure: Fx::from_bits(fs[1]),
        supply: Fx::from_bits(fs[2]),
        strait: Fx::from_bits(fs[3]),
    };
    assert_eq!(
        oh_sim::formula::strait_hours(Fx::from_bits(d), Fx::from_bits(v), f)
            .unwrap()
            .to_bits(),
        oracle
    );
    let (_, _, c) = inputs();
    assert_eq!(
        oh_sim::formula::strait_hours(
            Fx::from_num(10),
            Fx::from_num(2),
            c.corrections.values().next().unwrap().factors
        )
        .unwrap()
        .to_bits(),
        42949672960
    );
    let (d, u, _) = inputs();
    let defs = oh_data::Defines(BTreeMap::new());
    assert!(
        StraitFactors::from_defines(&defs, &d.map, u.province, ProvinceId(40), Fx::ONE, Fx::ONE)
            .is_err()
    );
}
#[test]
fn req_mil_04_strait_phase_failure_preserves_context_route_queue_clock() {
    use oh_data::national::ModifierInput;
    let (mut d, u, c) = inputs();
    d.map.states[0].infrastructure = 1_500_000_000;
    d.scenario.state_modifiers.insert(
        1,
        vec![
            ModifierInput {
                source: "fixture-add".into(),
                target_stat: "infrastructure".into(),
                operation: "add".into(),
                value: "-1000000000".into(),
                expires: Some(2),
            },
            ModifierInput {
                source: "fixture-mul".into(),
                target_stat: "infrastructure".into(),
                operation: "mul".into(),
                value: "2".into(),
                expires: None,
            },
        ],
    );
    let mut s = build(&d, u.clone(), c).unwrap();
    s.enqueue(
        0,
        u.nation,
        1,
        Command::Move {
            unit: u.id,
            destination: ProvinceId(40),
        },
    )
    .unwrap();
    s.step().unwrap();
    s.enqueue(1, u.nation, 2, Command::Stop { unit: u.id })
        .unwrap();
    let before = s.export_save_v3().unwrap();
    let hash = s.state_hash().unwrap();
    assert!(s.step().is_err());
    assert_eq!(s.export_save_v3().unwrap(), before);
    assert_eq!(s.state_hash().unwrap(), hash);
    assert_eq!(s.snapshot().tick(), 1);
}
#[test]
fn req_mil_04_strait_defines_separate_slot_no_fallback_and_identity_hash() {
    use oh_data::{DefineValue, Defines, Number};
    let (d, u, c) = inputs();
    let mut values = BTreeMap::from([
        (
            "terrain_hills".into(),
            DefineValue::Number(Number::Integer(1)),
        ),
        ("strait".into(), DefineValue::Number(Number::Integer(2))),
        (
            "river_small".into(),
            DefineValue::Number(Number::Integer(7)),
        ),
    ]);
    let defs = Defines(BTreeMap::from([("movement".into(), values.clone())]));
    let f = StraitFactors::from_defines(
        &defs,
        &d.map,
        ProvinceId(10),
        ProvinceId(40),
        Fx::ONE,
        Fx::ONE,
    )
    .unwrap();
    assert_eq!(
        oh_sim::formula::strait_hours(Fx::from_num(10), Fx::from_num(2), f).unwrap(),
        Fx::from_num(10)
    );
    for bad in [
        DefineValue::Array(vec![Number::Integer(2)]),
        DefineValue::Number(Number::Integer(0)),
        DefineValue::Number(Number::Integer(-1)),
        DefineValue::Number(Number::Integer(i64::MAX)),
    ] {
        values.insert("strait".into(), bad);
        let defs = Defines(BTreeMap::from([("movement".into(), values.clone())]));
        assert!(
            StraitFactors::from_defines(
                &defs,
                &d.map,
                ProvinceId(10),
                ProvinceId(40),
                Fx::ONE,
                Fx::ONE
            )
            .is_err()
        );
    }
    let a = build(&d, u.clone(), c).unwrap();
    let mut d2 = d.clone();
    d2.map.edges[0].kind = oh_data::map::EdgeKind::Normal;
    let w1 = oh_sim::world::World::from_loaded(&d).unwrap();
    let w2 = oh_sim::world::World::from_loaded(&d2).unwrap();
    assert_ne!(w1.definitions_hash(), w2.definitions_hash());
    let m1 = Movement::new(&w1, vec![u.clone()]).unwrap();
    let m2 = Movement::new(&w2, vec![u]).unwrap();
    let s1 = Simulation::with_movement(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        a.config().clone(),
        w1,
        m1,
    )
    .unwrap();
    let s2 = Simulation::with_movement(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        a.config().clone(),
        w2,
        m2,
    )
    .unwrap();
    assert_ne!(s1.state_hash().unwrap(), s2.state_hash().unwrap());
    assert_ne!(a.state_hash().unwrap(), s1.state_hash().unwrap());
}
#[test]
fn req_mil_04_strait_scheduled_missing_context_consumes_due_queue_only() {
    let (d, u, c) = inputs();
    let mut s = build(&d, u.clone(), c).unwrap();
    s.enqueue(
        0,
        u.nation,
        1,
        Command::Move {
            unit: u.id,
            destination: ProvinceId(40),
        },
    )
    .unwrap();
    s.enqueue(
        10,
        u.nation,
        2,
        Command::Move {
            unit: u.id,
            destination: ProvinceId(10),
        },
    )
    .unwrap();
    s.enqueue(20, u.nation, 3, Command::Stop { unit: u.id })
        .unwrap();
    for _ in 0..10 {
        s.step().unwrap();
    }
    let before = s.export_save_v3().unwrap();
    let step = s.step().unwrap();
    assert_eq!(
        step.commands[0].result,
        Err(oh_sim::Error::Movement(
            oh_sim::movement::MovementError::MissingContext
        ))
    );
    assert!(step.advanced);
    assert_eq!(s.snapshot().tick(), 11);
    assert_eq!(s.export_save_v3().unwrap().straits, before.straits);
    assert_eq!(s.export_save_v3().unwrap().base.units, before.base.units);
    assert_eq!(s.pending_commands().len(), 1);
}
#[test]
fn req_mil_04_strait_noncontiguous_zero_unit_order_is_canonical() {
    let (d, u, c) = inputs();
    let w = oh_sim::world::World::from_loaded(&d).unwrap();
    let mut zero = u.clone();
    zero.id = DivisionId(0);
    let mut zero_ctx = c.clone();
    zero_ctx.unit = DivisionId(0);
    let a = Movement::with_straits(
        &w,
        vec![u.clone(), zero.clone()],
        vec![c.clone(), zero_ctx.clone()],
    )
    .unwrap();
    let b = Movement::with_straits(&w, vec![zero, u], vec![zero_ctx, c]).unwrap();
    let make = |m| {
        Simulation::with_movement(
            "m1".into(),
            Date::new(2000, 1, 1).unwrap(),
            1,
            TimeConfig::from_defines(&d.pack.defines).unwrap(),
            w.clone(),
            m,
        )
        .unwrap()
    };
    let mut a = make(a);
    let mut b = make(b);
    assert_eq!(a.state_hash().unwrap(), b.state_hash().unwrap());
    assert_eq!(a.export_save_v3().unwrap(), b.export_save_v3().unwrap());
    let n = a.movement().unwrap().unit(DivisionId(0)).unwrap().nation();
    for s in [&mut a, &mut b] {
        s.enqueue(
            0,
            n,
            1,
            Command::Move {
                unit: DivisionId(0),
                destination: ProvinceId(40),
            },
        )
        .unwrap();
        s.step().unwrap();
    }
    assert_eq!(a.state_hash().unwrap(), b.state_hash().unwrap());
    assert_eq!(
        a.movement().unwrap().unit(DivisionId(0)).unwrap().elapsed(),
        Fx::ONE
    );
}
