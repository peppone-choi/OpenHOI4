use oh_core::{DivisionId, Fx, ProvinceId};
use oh_save::{Limits, SaveContext, decode, decode_with_limits, encode, inspect_header};
use oh_sim::{
    Command, Simulation,
    movement::{Factors, Movement, UnitInput},
    save_state::SimulationSaveV2,
};
use std::collections::{BTreeMap, BTreeSet};
fn context() -> SaveContext {
    SaveContext::national(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap()
}
fn sim(c: &SaveContext) -> Simulation {
    let base = c.simulation(7).unwrap();
    let w = base.world().unwrap().clone();
    let allowed = w
        .defs()
        .map()
        .provinces
        .iter()
        .filter(|p| p.kind == oh_data::map::ProvinceKind::Land)
        .map(|p| ProvinceId(p.id))
        .collect::<BTreeSet<_>>();
    let mut corrections = BTreeMap::new();
    for e in &w.defs().map().edges {
        for (a, b) in [(e.a, e.b), (e.b, e.a)] {
            if allowed.contains(&ProvinceId(a))
                && allowed.contains(&ProvinceId(b))
                && matches!(
                    e.kind,
                    oh_data::map::EdgeKind::Normal
                        | oh_data::map::EdgeKind::RiverSmall
                        | oh_data::map::EdgeKind::RiverLarge
                )
            {
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
    }
    let m = Movement::new(
        &w,
        vec![UnitInput {
            id: DivisionId(900),
            nation: w.inputs().nations()[0].id(),
            province: ProvinceId(10),
            speed: Fx::from_num(2),
            allowed,
            corrections,
        }],
    )
    .unwrap();
    Simulation::with_movement(
        "m1".into(),
        base.snapshot().date(),
        7,
        base.config().clone(),
        w,
        m,
    )
    .unwrap()
}
#[test]
fn movement_v2_mid_edge_pending_pause_and_exact_resume() {
    let c = context();
    let mut s = sim(&c);
    let n = s.movement().unwrap().units()[0].nation();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(900),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    s.enqueue(
        20,
        n,
        2,
        Command::Move {
            unit: DivisionId(900),
            destination: ProvinceId(10),
        },
    )
    .unwrap();
    s.enqueue(4, n, 3, Command::Pause(true)).unwrap();
    s.enqueue(4, n, 4, Command::Pause(false)).unwrap();
    s.step().unwrap();
    assert_eq!(s.movement().unwrap().units()[0].elapsed(), Fx::ONE);
    assert!(s.export_save().unwrap_err().contains("MovementRequiresV2"));
    let before = s.export_save_v2().unwrap();
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    assert_eq!(&bytes[4..6], &2u16.to_le_bytes());
    let mut r = decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(before, r.export_save_v2().unwrap());
    assert_eq!(s.state_hash().unwrap(), r.state_hash().unwrap());
    assert_eq!(encode(&r, &c, 0, vec![]).unwrap(), bytes);
    for _ in 0..30 {
        s.step().unwrap();
        r.step().unwrap();
        assert_eq!(s.export_save_v2().unwrap(), r.export_save_v2().unwrap());
        assert_eq!(s.state_hash().unwrap(), r.state_hash().unwrap());
    }
    assert_eq!(s.movement().unwrap().units()[0].province(), ProvinceId(10));
}
#[test]
fn v2_idle_empty_and_legacy_format_selection() {
    let c = context();
    let s = sim(&c);
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    assert_eq!(
        inspect_header(&bytes, &Limits::default())
            .unwrap()
            .format_version,
        2
    );
    let base = c.simulation(7).unwrap();
    assert_eq!(
        inspect_header(&encode(&base, &c, 0, vec![]).unwrap(), &Limits::default())
            .unwrap()
            .format_version,
        1
    );
    let w = base.world().unwrap().clone();
    let m = Movement::new(&w, vec![]).unwrap();
    let empty = Simulation::with_movement(
        "m1".into(),
        base.snapshot().date(),
        7,
        base.config().clone(),
        w,
        m,
    )
    .unwrap();
    let b = encode(&empty, &c, 0, vec![]).unwrap();
    assert_eq!(&b[4..6], &2u16.to_le_bytes());
    assert!(
        decode(&b, &c, false)
            .unwrap()
            .simulation
            .movement()
            .is_some()
    );
    assert_ne!(empty.state_hash().unwrap(), base.state_hash().unwrap());
}
#[test]
fn v2_untrusted_reference_progress_context_and_queue_preserve_live() {
    let c = context();
    let mut s = sim(&c);
    let n = s.movement().unwrap().units()[0].nation();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(900),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    s.step().unwrap();
    let original = s.export_save_v2().unwrap();
    let hash = s.state_hash().unwrap();
    type Mutation = fn(&mut SimulationSaveV2);
    let cases: &[Mutation] = &[
        |d| d.units[0].nation = u16::MAX,
        |d| d.units[0].province = u16::MAX,
        |d| d.units[0].speed = 0,
        |d| d.units[0].speed = -1,
        |d| d.units[0].elapsed = -1,
        |d| d.units[0].elapsed = i64::MIN,
        |d| d.units[0].elapsed = d.units[0].route[0].hours,
        |d| d.units[0].route[0].hours += 1,
        |d| d.units[0].route[0].to = u16::MAX,
        |d| d.units[0].corrections[0].factors[0] = 0,
        |d| d.units[0].corrections[0].factors[1] = i64::MAX,
        |d| d.units[0].allowed.reverse(),
        |d| d.units.push(d.units[0].clone()),
        |d| d.units[0].corrections.clear(),
    ];
    for change in cases {
        let mut d = original.clone();
        change(&mut d);
        assert!(Simulation::from_save_v2(d, c.restore_context()).is_err());
        assert_eq!(s.state_hash().unwrap(), hash);
        assert_eq!(s.export_save_v2().unwrap(), original);
    }
    let mut invalid = original.clone();
    invalid.queue.push(oh_sim::save_state::PendingV2 {
        tick: 0,
        nation: n.0,
        sequence: 2,
        command: oh_sim::save_state::CommandV2::Stop { unit: 900 },
    });
    assert!(Simulation::from_save_v2(invalid, c.restore_context()).is_err());
}
#[test]
fn v2_preflight_limits_unknown_version_and_hash_tamper() {
    let c = context();
    let s = sim(&c);
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    let l = Limits {
        map_entries_max: 0,
        ..Limits::default()
    };
    assert!(
        decode_with_limits(&bytes, &c, false, &l)
            .unwrap_err_string()
            .contains("Limit")
    );
    let l = Limits {
        allocation_budget_bytes: 1,
        ..Limits::default()
    };
    assert!(decode_with_limits(&bytes, &c, false, &l).is_err());
    let mut b = bytes.clone();
    b[4..6].copy_from_slice(&3u16.to_le_bytes());
    assert!(
        decode(&b, &c, false)
            .err()
            .unwrap()
            .contains("UnsupportedFormat")
    );
    let end = 10 + u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
    let mut h = inspect_header(&bytes, &Limits::default()).unwrap();
    h.state_hash ^= 1;
    let hb = oh_core::canonical_bytes(&h).unwrap();
    let mut b = bytes[..6].to_vec();
    b.extend((hb.len() as u32).to_le_bytes());
    b.extend(hb);
    b.extend(&bytes[end..]);
    assert!(
        decode(&b, &c, false)
            .err()
            .unwrap()
            .contains("StateHashMismatch")
    );
}
trait ErrText {
    fn unwrap_err_string(self) -> String;
}
impl<T> ErrText for Result<T, String> {
    fn unwrap_err_string(self) -> String {
        match self {
            Err(e) => e,
            Ok(_) => panic!("expected rejection"),
        }
    }
}
#[test]
fn movement_zero_typed_ids_are_not_dense_indices() {
    fn copy(src: &std::path::Path, dst: &std::path::Path) {
        std::fs::create_dir_all(dst).unwrap();
        for entry in std::fs::read_dir(src).unwrap() {
            let e = entry.unwrap();
            let next = dst.join(e.file_name());
            if e.file_type().unwrap().is_dir() {
                copy(&e.path(), &next)
            } else {
                std::fs::copy(e.path(), next).unwrap();
            }
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pack");
    copy(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &root,
    );
    for (file, changes) in [
        (
            "scenarios/m1/nations/NTH.toml",
            vec![("id = 1", "id = 0"), ("capital = 10", "capital = 0")],
        ),
        (
            "maps/testland/provinces.csv",
            vec![("10,200,40,40", "0,200,40,40")],
        ),
        (
            "maps/testland/states.toml",
            vec![("[10, 20]", "[0, 20]"), ("province = 10", "province = 0")],
        ),
        (
            "maps/testland/adjacency_overrides.csv",
            vec![("10,20", "0,20"), ("10,40", "0,40")],
        ),
    ] {
        let p = root.join(file);
        let mut text = std::fs::read_to_string(&p).unwrap();
        for (from, to) in changes {
            assert!(text.contains(from));
            text = text.replace(from, to);
        }
        std::fs::write(p, text).unwrap();
    }
    let c = SaveContext::national(&root, "m1").unwrap();
    let base = c.simulation(7).unwrap();
    let w = base.world().unwrap().clone();
    let n = oh_core::NationId(0);
    let factors = Factors {
        terrain: Fx::from_num(1.5),
        infrastructure: Fx::ONE,
        supply: Fx::ONE,
        river: Fx::ONE,
    };
    let m = Movement::new(
        &w,
        vec![UnitInput {
            id: DivisionId(0),
            nation: n,
            province: ProvinceId(0),
            speed: Fx::from_num(2),
            allowed: BTreeSet::from([ProvinceId(0), ProvinceId(20)]),
            corrections: BTreeMap::from([
                ((ProvinceId(0), ProvinceId(20)), factors),
                ((ProvinceId(20), ProvinceId(0)), factors),
            ]),
        }],
    )
    .unwrap();
    let mut s = Simulation::with_movement(
        "m1".into(),
        base.snapshot().date(),
        7,
        base.config().clone(),
        w,
        m,
    )
    .unwrap();
    s.enqueue(
        0,
        n,
        1,
        Command::Move {
            unit: DivisionId(0),
            destination: ProvinceId(20),
        },
    )
    .unwrap();
    s.step().unwrap();
    let bytes = encode(&s, &c, 0, vec![0]).unwrap();
    let mut r = decode(&bytes, &c, false).unwrap().simulation;
    for _ in 0..2 {
        s.step().unwrap();
        r.step().unwrap();
    }
    assert_eq!(
        s.movement()
            .unwrap()
            .unit(DivisionId(0))
            .unwrap()
            .province(),
        ProvinceId(20)
    );
    assert_eq!(s.state_hash().unwrap(), r.state_hash().unwrap());
}
#[test]
fn v2_new_queue_limit_and_oversized_unit_vector_preflight() {
    let c = context();
    let mut s = sim(&c);
    let n = s.movement().unwrap().units()[0].nation();
    s.enqueue(
        10,
        n,
        1,
        Command::Stop {
            unit: DivisionId(900),
        },
    )
    .unwrap();
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    let l = Limits {
        queue_max_entries: 0,
        ..Limits::default()
    };
    assert!(
        decode_with_limits(&bytes, &c, false, &l)
            .err()
            .unwrap()
            .contains("Limit")
    );
    // A tiny forged body advertises more units than policy before any allocation.
    let end = 10 + u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
    let mut body = oh_core::canonical_bytes(&s.export_save_v2().unwrap().base).unwrap();
    body.push(0);
    let mut count = 65537u64;
    while count >= 128 {
        body.push((count as u8 & 127) | 128);
        count >>= 7;
    }
    body.push(count as u8);
    let mut forged = bytes[..end].to_vec();
    forged.extend(zstd::stream::encode_all(body.as_slice(), 3).unwrap());
    assert!(
        decode(&forged, &c, false)
            .err()
            .unwrap()
            .contains("Limit: collection/string length")
    );
}
