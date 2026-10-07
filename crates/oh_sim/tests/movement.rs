use oh_core::{DivisionId, Fx, NationId, ProvinceId};
use oh_sim::{
    Command, Date, Simulation, TimeConfig,
    movement::{Factors, Movement, UnitInput},
};
use std::collections::{BTreeMap, BTreeSet};
fn sim() -> Simulation {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut d = oh_data::national::load_scenario(&root, "m1").unwrap();
    // Synthetic graph, never alter legacy content. Noncontiguous IDs already 10/20/30/40.
    d.map
        .edges
        .retain(|e| [10, 20, 30, 40].contains(&e.a) && [10, 20, 30, 40].contains(&e.b));
    for e in &mut d.map.edges {
        e.distance_km = Fx::from_num(10);
        e.kind = oh_data::map::EdgeKind::Normal;
    }
    let world = oh_sim::world::World::from_loaded(&d).unwrap();
    let nation = world.inputs().nations()[0].id();
    let mut corrections = BTreeMap::new();
    for e in &d.map.edges {
        for (a, b) in [(e.a, e.b), (e.b, e.a)] {
            corrections.insert(
                (ProvinceId(a), ProvinceId(b)),
                Factors {
                    terrain: Fx::from_num(1.5),
                    infrastructure: Fx::ONE,
                    supply: Fx::ONE,
                    river: Fx::ONE,
                },
            );
        }
    }
    let allowed = d
        .map
        .provinces
        .iter()
        .filter(|p| p.kind == oh_data::map::ProvinceKind::Land)
        .map(|p| ProvinceId(p.id))
        .collect::<BTreeSet<_>>();
    let m = Movement::new(
        &world,
        vec![UnitInput {
            id: DivisionId(700),
            nation,
            province: ProvinceId(10),
            speed: Fx::from_num(2),
            allowed,
            corrections,
        }],
    )
    .unwrap();
    Simulation::with_movement(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&d.pack.defines).unwrap(),
        world,
        m,
    )
    .unwrap()
}
#[test]
fn req_mil_04_direct_command_actual_phase_and_fractional_carry() {
    let mut s = sim();
    let n = s
        .movement()
        .unwrap()
        .unit(DivisionId(700))
        .unwrap()
        .nation();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    for _ in 0..7 {
        s.step().unwrap();
    }
    let u = s.movement().unwrap().unit(DivisionId(700)).unwrap();
    assert_eq!(u.province(), ProvinceId(10));
    assert_eq!(u.elapsed(), Fx::from_num(7));
    s.step().unwrap();
    assert_eq!(
        s.movement()
            .unwrap()
            .unit(DivisionId(700))
            .unwrap()
            .province(),
        ProvinceId(20)
    );
}
#[test]
fn req_mil_04_authority_and_pause_are_atomic() {
    let mut s = sim();
    let hash = s.state_hash().unwrap();
    assert!(
        s.enqueue(
            0,
            NationId(u16::MAX),
            1,
            Command::Move {
                unit: DivisionId(700),
                destination: ProvinceId(20)
            }
        )
        .is_err()
    );
    assert_eq!(s.state_hash().unwrap(), hash);
    assert!(s.pending_commands().is_empty());
    let n = s
        .movement()
        .unwrap()
        .unit(DivisionId(700))
        .unwrap()
        .nation();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    s.step().unwrap();
    s.enqueue(1, n, 2, Command::Pause(true)).unwrap();
    s.step().unwrap();
    let before = s.state_hash().unwrap();
    s.step().unwrap();
    assert_eq!(s.state_hash().unwrap(), before);
}
// Independent synthetic edge lists (distances are test inputs, never content defaults).
fn graph(
    edges: &[(u16, u16, oh_data::map::EdgeKind, Fx)],
    speed: Fx,
    factors: Factors,
) -> (oh_data::national::LoadedNational, Simulation) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut d = oh_data::national::load_scenario(&root, "m1").unwrap();
    d.map.edges = edges
        .iter()
        .map(|&(a, b, kind, distance_km)| oh_data::map::Edge {
            a,
            b,
            kind,
            distance_km,
        })
        .collect();
    d.map.edges.sort_by_key(|e| (e.a, e.b));
    let w = oh_sim::world::World::from_loaded(&d).unwrap();
    let n = w.inputs().nations()[0].id();
    let allowed = d
        .map
        .provinces
        .iter()
        .filter(|p| p.kind == oh_data::map::ProvinceKind::Land)
        .map(|p| ProvinceId(p.id))
        .collect();
    let mut corrections = BTreeMap::new();
    for e in &d.map.edges {
        if matches!(
            e.kind,
            oh_data::map::EdgeKind::Normal
                | oh_data::map::EdgeKind::RiverSmall
                | oh_data::map::EdgeKind::RiverLarge
        ) && e.b < 50
        {
            for (a, b) in [(e.a, e.b), (e.b, e.a)] {
                corrections.insert((ProvinceId(a), ProvinceId(b)), factors);
            }
        }
    }
    let m = Movement::new(
        &w,
        vec![UnitInput {
            id: DivisionId(700),
            nation: n,
            province: ProvinceId(10),
            speed,
            allowed,
            corrections,
        }],
    )
    .unwrap();
    let s = Simulation::with_movement(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&d.pack.defines).unwrap(),
        w,
        m,
    )
    .unwrap();
    (d, s)
}
fn factors() -> Factors {
    Factors {
        terrain: Fx::from_num(1.5),
        infrastructure: Fx::ONE,
        supply: Fx::ONE,
        river: Fx::ONE,
    }
}
#[test]
fn req_mil_04_path_golden_tie_full_id_order_and_tick_carry() {
    use oh_data::map::EdgeKind::Normal as N;
    let (_, mut s) = graph(
        &[
            (10, 20, N, Fx::from_num(10)),
            (10, 30, N, Fx::from_num(10)),
            (20, 40, N, Fx::from_num(10)),
            (30, 40, N, Fx::from_num(10)),
        ],
        Fx::from_num(2),
        factors(),
    );
    assert_eq!(
        s.movement()
            .unwrap()
            .path(s.world().unwrap(), DivisionId(700), ProvinceId(40))
            .unwrap(),
        [ProvinceId(20), ProvinceId(40)]
    );
    let n = s.movement().unwrap().units()[0].nation();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(40),
        },
    )
    .unwrap();
    for _ in 0..8 {
        s.step().unwrap();
    }
    let u = &s.movement().unwrap().units()[0];
    assert_eq!(u.province(), ProvinceId(20));
    assert_eq!(u.elapsed(), Fx::from_num(0.5));
    assert_eq!(u.remaining_hours().unwrap(), Fx::from_num(7));
    for _ in 0..7 {
        s.step().unwrap();
    }
    let u = &s.movement().unwrap().units()[0];
    assert_eq!(u.province(), ProvinceId(40));
    assert!(u.remaining_route().is_empty());
    assert_eq!(u.elapsed(), Fx::ZERO);
    s.enqueue(
        15,
        n,
        2,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(40),
        },
    )
    .unwrap();
    s.step().unwrap();
    assert_eq!(
        s.movement().unwrap().units()[0].remaining_hours().unwrap(),
        Fx::ZERO
    );
}
#[test]
fn req_mil_04_water_impassable_strait_missing_and_overflow_reject_atomically() {
    use oh_data::map::EdgeKind as E;
    for kind in [E::Impassable, E::Strait] {
        let (_, mut s) = graph(&[(10, 20, kind, Fx::ONE)], Fx::ONE, factors());
        let hash = s.state_hash().unwrap();
        let n = s.movement().unwrap().units()[0].nation();
        assert!(
            s.enqueue(
                0,
                n,
                1,
                Command::Move {
                    unit: DivisionId(700),
                    destination: ProvinceId(20)
                }
            )
            .is_err()
        );
        assert_eq!(s.state_hash().unwrap(), hash);
    }
    let (_, mut s) = graph(
        &[(10, 50, E::Normal, Fx::ONE), (10, 60, E::Normal, Fx::ONE)],
        Fx::ONE,
        factors(),
    );
    let n = s.movement().unwrap().units()[0].nation();
    let hash = s.state_hash().unwrap();
    for p in [20, 50, 60, u16::MAX] {
        assert!(
            s.enqueue(
                0,
                n,
                1,
                Command::Move {
                    unit: DivisionId(700),
                    destination: ProvinceId(p)
                }
            )
            .is_err()
        );
        assert_eq!(s.state_hash().unwrap(), hash);
    }
    let (_, mut s) = graph(&[(10, 20, E::Normal, Fx::MAX)], Fx::from_bits(1), factors());
    let n = s.movement().unwrap().units()[0].nation();
    let hash = s.state_hash().unwrap();
    assert!(
        s.enqueue(
            0,
            n,
            1,
            Command::Move {
                unit: DivisionId(700),
                destination: ProvinceId(20)
            }
        )
        .is_err()
    );
    assert_eq!(s.state_hash().unwrap(), hash);
    let (_, mut s) = graph(&[(10, 20, E::Normal, Fx::from_bits(1))], Fx::MAX, factors());
    let n = s.movement().unwrap().units()[0].nation();
    assert!(
        s.enqueue(
            0,
            n,
            1,
            Command::Move {
                unit: DivisionId(700),
                destination: ProvinceId(20)
            }
        )
        .is_err()
    );
}
#[test]
fn req_mil_04_formula_order_exact_floor_missing_define_no_defaults() {
    use oh_sim::formula::movement_hours;
    let f = factors();
    assert_eq!(
        movement_hours(Fx::from_num(10), Fx::from_num(2), f)
            .unwrap()
            .to_bits(),
        15i64 << 31
    );
    // Integer oracle: fixed division/multiplication floor after each operation.
    let d = 12345678901i64;
    let v = 3456789012i64;
    let fs = [5678901234i64, 4567890123, 3456789012, 2345678901];
    let mut expected = ((i128::from(d) << 32) / i128::from(v)) as i64;
    for factor in fs {
        expected = ((i128::from(expected) * i128::from(factor)) >> 32) as i64;
    }
    assert_eq!(
        movement_hours(
            Fx::from_bits(d),
            Fx::from_bits(v),
            Factors {
                terrain: Fx::from_bits(fs[0]),
                infrastructure: Fx::from_bits(fs[1]),
                supply: Fx::from_bits(fs[2]),
                river: Fx::from_bits(fs[3])
            }
        )
        .unwrap()
        .to_bits(),
        expected
    );
    for speed in [Fx::ZERO, -Fx::ONE] {
        assert!(movement_hours(Fx::ONE, speed, f).is_err());
    }
    for bad in [Fx::ZERO, -Fx::ONE, Fx::MAX] {
        assert!(movement_hours(Fx::from_num(10), Fx::ONE, Factors { terrain: bad, ..f }).is_err());
    }
    let s = sim();
    let defs = oh_data::Defines(BTreeMap::new());
    assert!(
        Factors::from_defines(
            &defs,
            s.world().unwrap().defs().map(),
            ProvinceId(10),
            ProvinceId(20),
            Fx::ONE,
            Fx::ONE
        )
        .is_err()
    );
}
#[test]
fn req_mil_04_context_references_speed_duplicate_and_missing_context() {
    let s = sim();
    let w = s.world().unwrap();
    let valid = UnitInput {
        id: DivisionId(0),
        nation: w.inputs().nations()[0].id(),
        province: ProvinceId(10),
        speed: Fx::ONE,
        allowed: BTreeSet::from([ProvinceId(10), ProvinceId(20)]),
        corrections: BTreeMap::new(),
    };
    for speed in [Fx::ZERO, -Fx::ONE] {
        assert!(
            Movement::new(
                w,
                vec![UnitInput {
                    speed,
                    ..valid.clone()
                }]
            )
            .is_err()
        );
    }
    assert!(Movement::new(w, vec![valid.clone(), valid.clone()]).is_err());
    assert!(
        Movement::new(
            w,
            vec![UnitInput {
                nation: NationId(u16::MAX),
                ..valid.clone()
            }]
        )
        .is_err()
    );
    assert!(
        Movement::new(
            w,
            vec![UnitInput {
                province: ProvinceId(50),
                ..valid.clone()
            }]
        )
        .is_err()
    );
    let m = Movement::new(w, vec![valid]).unwrap();
    assert!(m.path(w, DivisionId(0), ProvinceId(20)).is_err());
    assert!(m.path(w, DivisionId(u32::MAX), ProvinceId(10)).is_err());
}
#[test]
fn req_mil_04_new_authority_and_pending_command_hash_sensitivity() {
    let original = sim();
    let hash = original.state_hash().unwrap();
    let context = oh_sim::save_state::RestoreContext {
        scenario: "m1".into(),
        start_date: Date::new(2000, 1, 1).unwrap(),
        world: original.world().cloned(),
    };
    let dto = original.export_save_v2().unwrap();
    let mut changed = dto.clone();
    changed.units[0].speed += 1;
    let s = Simulation::from_save_v2(changed, &context).unwrap();
    assert_ne!(s.state_hash().unwrap(), hash);
    for i in 0..4 {
        let mut changed = dto.clone();
        changed.units[0].corrections[0].factors[i] += 1;
        let s = Simulation::from_save_v2(changed, &context).unwrap();
        assert_ne!(s.state_hash().unwrap(), hash);
    }
    let n = original.movement().unwrap().units()[0].nation();
    let mut a = original.clone();
    a.enqueue(
        20,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    let mut b = original.clone();
    b.enqueue(
        20,
        n,
        1,
        Command::Stop {
            unit: DivisionId(700),
        },
    )
    .unwrap();
    assert_ne!(a.state_hash().unwrap(), b.state_hash().unwrap());
    assert_ne!(a.state_hash().unwrap(), hash);
    let mut a = original.clone();
    let mut b = original.clone();
    a.enqueue(
        10,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    a.enqueue(
        20,
        n,
        2,
        Command::Stop {
            unit: DivisionId(700),
        },
    )
    .unwrap();
    b.enqueue(
        20,
        n,
        2,
        Command::Stop {
            unit: DivisionId(700),
        },
    )
    .unwrap();
    b.enqueue(
        10,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    assert_eq!(a.state_hash().unwrap(), b.state_hash().unwrap());
}
#[test]
fn req_mil_04_mid_edge_stop_reroute_preserves_leg_and_elapsed() {
    use oh_data::map::EdgeKind::Normal as N;
    let (_, initial) = graph(
        &[
            (10, 20, N, Fx::from_num(10)),
            (20, 30, N, Fx::from_num(10)),
            (20, 40, N, Fx::from_num(10)),
        ],
        Fx::from_num(2),
        factors(),
    );
    let n = initial.movement().unwrap().units()[0].nation();
    let mut stop = initial.clone();
    stop.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(30),
        },
    )
    .unwrap();
    for _ in 0..7 {
        stop.step().unwrap();
    }
    stop.enqueue(
        7,
        n,
        2,
        Command::Stop {
            unit: DivisionId(700),
        },
    )
    .unwrap();
    stop.step().unwrap();
    let u = &stop.movement().unwrap().units()[0];
    assert_eq!(u.province(), ProvinceId(20));
    assert_eq!(u.elapsed(), Fx::ZERO);
    assert!(u.remaining_route().is_empty());
    let mut reroute = initial;
    reroute
        .enqueue(
            0,
            n,
            1,
            Command::Move {
                unit: DivisionId(700),
                destination: ProvinceId(30),
            },
        )
        .unwrap();
    for _ in 0..7 {
        reroute.step().unwrap();
    }
    let before = reroute.state_hash().unwrap();
    assert!(
        reroute
            .enqueue(
                7,
                n,
                2,
                Command::Move {
                    unit: DivisionId(700),
                    destination: ProvinceId(60)
                }
            )
            .is_err()
    );
    assert_eq!(reroute.state_hash().unwrap(), before);
    reroute
        .enqueue(
            7,
            n,
            2,
            Command::Move {
                unit: DivisionId(700),
                destination: ProvinceId(40),
            },
        )
        .unwrap();
    reroute.step().unwrap();
    let u = &reroute.movement().unwrap().units()[0];
    assert_eq!(u.province(), ProvinceId(20));
    assert_eq!(u.elapsed(), Fx::from_num(0.5));
    assert_eq!(u.remaining_route(), [ProvinceId(40)]);
}
#[test]
fn req_mil_04_scheduled_rejection_consumes_only_due_commands() {
    use oh_data::map::EdgeKind::Normal as N;
    let (_, s) = graph(
        &[(10, 20, N, Fx::from_num(10)), (10, 30, N, Fx::from_num(10))],
        Fx::from_num(2),
        factors(),
    );
    let context = oh_sim::save_state::RestoreContext {
        scenario: "m1".into(),
        start_date: s.snapshot().date(),
        world: s.world().cloned(),
    };
    let mut dto = s.export_save_v2().unwrap();
    dto.units[0]
        .corrections
        .retain(|c| (c.from, c.to) != (30, 10));
    let mut s = Simulation::from_save_v2(dto, &context).unwrap();
    let n = s.movement().unwrap().units()[0].nation();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(30),
        },
    )
    .unwrap();
    s.enqueue(
        8,
        n,
        2,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    s.enqueue(
        20,
        n,
        3,
        Command::Stop {
            unit: DivisionId(700),
        },
    )
    .unwrap();
    for _ in 0..8 {
        s.step().unwrap();
    }
    let before = s.export_save_v2().unwrap();
    let step = s.step().unwrap();
    assert!(step.advanced);
    assert!(step.commands[0].result.is_err());
    assert_eq!(s.snapshot().tick(), 9);
    assert_eq!(s.export_save_v2().unwrap().units, before.units);
    assert_eq!(s.pending_commands().len(), 1);
}
#[test]
fn req_mil_04_phase_error_preserves_movement_queue_and_clock() {
    use oh_data::{map::EdgeKind::Normal as N, national::ModifierInput};
    let (mut d, _) = graph(&[(10, 20, N, Fx::from_num(10))], Fx::from_num(2), factors());
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
    let w = oh_sim::world::World::from_loaded(&d).unwrap();
    let n = w.inputs().nations()[0].id();
    let m = Movement::new(
        &w,
        vec![UnitInput {
            id: DivisionId(700),
            nation: n,
            province: ProvinceId(10),
            speed: Fx::from_num(2),
            allowed: BTreeSet::from([ProvinceId(10), ProvinceId(20)]),
            corrections: BTreeMap::from([
                ((ProvinceId(10), ProvinceId(20)), factors()),
                ((ProvinceId(20), ProvinceId(10)), factors()),
            ]),
        }],
    )
    .unwrap();
    let mut s = Simulation::with_movement(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&d.pack.defines).unwrap(),
        w,
        m,
    )
    .unwrap();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    s.step().unwrap();
    s.enqueue(1, n, 2, Command::SetSpeed(5)).unwrap();
    let before = s.export_save_v2().unwrap();
    let hash = s.state_hash().unwrap();
    assert!(s.step().is_err());
    assert_eq!(s.export_save_v2().unwrap(), before);
    assert_eq!(s.state_hash().unwrap(), hash);
    assert_eq!(s.snapshot().tick(), 1);
    assert_eq!(s.pending_commands().len(), 1);
}
#[test]
fn req_mil_04_zero_elapsed_stop_and_back_to_origin_no_teleport() {
    use oh_data::map::EdgeKind::Normal as N;
    let (_, initial) = graph(&[(10, 20, N, Fx::from_num(10))], Fx::from_num(2), factors());
    let n = initial.movement().unwrap().units()[0].nation();
    let mut s = initial.clone();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(700),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    s.enqueue(
        0,
        n,
        2,
        Command::Stop {
            unit: DivisionId(700),
        },
    )
    .unwrap();
    s.step().unwrap();
    assert_eq!(s.movement().unwrap().units()[0].province(), ProvinceId(10));
    assert!(
        s.movement().unwrap().units()[0]
            .remaining_route()
            .is_empty()
    );
    for at in [1u64, 7] {
        let mut s = initial.clone();
        s.enqueue(
            0,
            n,
            1,
            Command::Move {
                unit: DivisionId(700),
                destination: ProvinceId(20),
            },
        )
        .unwrap();
        for _ in 0..at {
            s.step().unwrap();
        }
        s.enqueue(
            at,
            n,
            2,
            Command::Move {
                unit: DivisionId(700),
                destination: ProvinceId(10),
            },
        )
        .unwrap();
        for _ in at..8 {
            s.step().unwrap();
        }
        assert_eq!(s.movement().unwrap().units()[0].province(), ProvinceId(20));
        assert_eq!(
            s.movement().unwrap().units()[0].elapsed(),
            Fx::from_num(0.5)
        );
        for _ in 8..15 {
            s.step().unwrap();
        }
        assert_eq!(s.movement().unwrap().units()[0].province(), ProvinceId(10));
        assert!(
            s.movement().unwrap().units()[0]
                .remaining_route()
                .is_empty()
        );
    }
}
