use oh_core::{DivisionId, Fx, ProvinceId};
use oh_save::{SaveContext, decode, encode};
use oh_sim::{
    Command, Simulation,
    movement::{CrossingKind, Movement, StraitContext, StraitFactors, StraitInput, UnitInput},
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
    let n = w.inputs().nations()[0].id();
    let speed = w.defs().map().edge(10, 40).unwrap().distance_km;
    let unit = UnitInput {
        id: DivisionId(900),
        nation: n,
        province: ProvinceId(10),
        speed,
        allowed: BTreeSet::from([ProvinceId(10), ProvinceId(40)]),
        corrections: BTreeMap::new(),
    };
    let value = StraitContext {
        kind: CrossingKind::Strait,
        factors: StraitFactors {
            terrain: Fx::ONE,
            infrastructure: Fx::ONE,
            supply: Fx::ONE,
            strait: Fx::from_num(10),
        },
    };
    let m = Movement::with_straits(
        &w,
        vec![unit],
        vec![StraitInput {
            unit: DivisionId(900),
            corrections: BTreeMap::from([
                ((ProvinceId(10), ProvinceId(40)), value),
                ((ProvinceId(40), ProvinceId(10)), value),
            ]),
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
fn strait_v3_tick9_full_context_pending_and_resume() {
    let c = context();
    let mut s = sim(&c);
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
    for _ in 0..9 {
        s.step().unwrap();
    }
    let dto = s.export_save_v3().unwrap();
    assert_eq!(dto.base.units[0].elapsed, 38654705664);
    assert!(s.export_save_v2().is_err());
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    assert_eq!(&bytes[4..6], &3u16.to_le_bytes());
    let mut r = decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(dto, r.export_save_v3().unwrap());
    assert_eq!(s.state_hash().unwrap(), r.state_hash().unwrap());
    s.step().unwrap();
    r.step().unwrap();
    assert_eq!(r.movement().unwrap().units()[0].province(), ProvinceId(40));
    assert_eq!(s.state_hash().unwrap(), r.state_hash().unwrap());
}
#[test]
fn frozen_97f_v2_reader_writer_hash_dto_native_contract_unchanged() {
    let c = context();
    let bytes = include_bytes!("fixtures/movement-v2-97f.ohsave");
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/movement-v2-97f.expected.json")).unwrap();
    let mut s = decode(bytes, &c, false).unwrap().simulation;
    assert!(!s.movement().unwrap().has_strait_context());
    assert_eq!(
        format!("{:016x}", s.state_hash().unwrap()),
        "18bff036731f4f26"
    );
    assert_eq!(
        serde_json::to_value(s.export_save_v2().unwrap()).unwrap(),
        expected["split"]["dto"]
    );
    assert_eq!(encode(&s, &c, 0, vec![]).unwrap().as_slice(), bytes);
    while s.snapshot().tick() < 24 {
        s.step().unwrap();
    }
    assert_eq!(
        format!("{:016x}", s.state_hash().unwrap()),
        "a145b72ad0ad25e6"
    );
    assert_eq!(
        serde_json::to_value(s.export_save_v2().unwrap()).unwrap(),
        expected["continuous"]["dto"]
    );
    assert_eq!(
        oh_core::canonical_bytes(&s)
            .unwrap()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        expected["continuous"]["canonical_hex"].as_str().unwrap()
    );
}
#[test]
fn v3_kind_direction_context_raw_bits_and_idle_empty_version_selection() {
    let c = context();
    let s = sim(&c);
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    assert_eq!(&bytes[4..6], &3u16.to_le_bytes());
    let dto = s.export_save_v3().unwrap();
    assert_eq!(dto.straits[0].corrections[0].kind, 3);
    assert_eq!(
        dto.straits[0].corrections[0].factors,
        [4294967296, 4294967296, 4294967296, 42949672960]
    );
    for i in 0..4 {
        let mut d = dto.clone();
        d.straits[0].corrections[0].factors[i] += 1;
        let changed = Simulation::from_save_v3(d, c.restore_context()).unwrap();
        assert_ne!(s.state_hash().unwrap(), changed.state_hash().unwrap());
    }
    let mut missing_direction = dto.clone();
    missing_direction.straits[0].corrections.pop();
    let changed = Simulation::from_save_v3(missing_direction, c.restore_context()).unwrap();
    assert_ne!(s.state_hash().unwrap(), changed.state_hash().unwrap());
    let base = c.simulation(7).unwrap();
    let w = base.world().unwrap().clone();
    let m = Movement::with_straits(&w, vec![], vec![]).unwrap();
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
    assert_eq!(&b[4..6], &3u16.to_le_bytes());
    assert!(
        decode(&b, &c, false)
            .unwrap()
            .simulation
            .movement()
            .unwrap()
            .has_strait_context()
    );
    assert!(empty.export_save_v2().is_err());
}
#[test]
fn v3_malformed_context_route_queue_failure_preserves_live() {
    let c = context();
    let mut s = sim(&c);
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
    s.step().unwrap();
    let original = s.export_save_v3().unwrap();
    let hash = s.state_hash().unwrap();
    type Mutation = fn(&mut oh_sim::save_state::SimulationSaveV3);
    let cases: &[Mutation] = &[
        |d| d.straits[0].unit = u32::MAX,
        |d| d.straits.clear(),
        |d| d.straits.push(d.straits[0].clone()),
        |d| d.straits[0].corrections[0].kind = 0,
        |d| d.straits[0].corrections[0].kind = 1,
        |d| d.straits[0].corrections[0].kind = 255,
        |d| d.straits[0].corrections[0].factors[3] = 0,
        |d| d.straits[0].corrections[0].factors[3] = -1,
        |d| d.straits[0].corrections[0].factors[3] = i64::MAX,
        |d| d.straits[0].corrections[0].to = 50,
        |d| d.straits[0].corrections[0].from = u16::MAX,
        |d| d.straits[0].corrections.reverse(),
        |d| {
            let c = d.straits[0].corrections[0].clone();
            d.straits[0].corrections.push(c);
        },
        |d| {
            d.base.units[0].allowed.pop();
        },
        |d| d.base.units[0].elapsed = i64::MIN,
        |d| d.base.units[0].route[0].hours += 1,
    ];
    for mutate in cases {
        let mut d = original.clone();
        mutate(&mut d);
        assert!(Simulation::from_save_v3(d, c.restore_context()).is_err());
        assert_eq!(s.export_save_v3().unwrap(), original);
        assert_eq!(s.state_hash().unwrap(), hash);
    }
}
#[test]
fn v3_context_vector_and_kind_preflight_and_unknown_version() {
    let c = context();
    let s = sim(&c);
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    let end = 10 + u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
    let mut dto = s.export_save_v3().unwrap();
    dto.straits[0].corrections[0].kind = 0;
    let mut b = bytes[..end].to_vec();
    b.extend(
        zstd::stream::encode_all(oh_core::canonical_bytes(&dto).unwrap().as_slice(), 3).unwrap(),
    );
    assert!(
        decode(&b, &c, false)
            .err()
            .unwrap()
            .contains("Postcard: strait crossing kind")
    );
    let mut body = oh_core::canonical_bytes(&s.export_save_v3().unwrap().base).unwrap();
    let mut count = 65537u64;
    while count >= 128 {
        body.push((count as u8 & 127) | 128);
        count >>= 7;
    }
    body.push(count as u8);
    let mut b = bytes[..end].to_vec();
    b.extend(zstd::stream::encode_all(body.as_slice(), 3).unwrap());
    assert!(
        decode(&b, &c, false)
            .err()
            .unwrap()
            .contains("Limit: collection/string length")
    );
    let mut b = bytes.clone();
    b[4..6].copy_from_slice(&4u16.to_le_bytes());
    assert!(
        decode(&b, &c, false)
            .err()
            .unwrap()
            .contains("UnsupportedFormat")
    );
}
#[test]
fn v3_request6_stop_pause_reroute_pending_restore_boundary() {
    let c = context();
    let mut s = sim(&c);
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
    s.enqueue(9, n, 2, Command::Pause(true)).unwrap();
    s.enqueue(
        9,
        n,
        3,
        Command::Move {
            unit: DivisionId(900),
            destination: ProvinceId(10),
        },
    )
    .unwrap();
    s.step().unwrap();
    assert!(s.snapshot().paused());
    assert_eq!(s.movement().unwrap().units()[0].elapsed(), Fx::from_num(9));
    let bytes = encode(&s, &c, 0, vec![]).unwrap();
    let mut r = decode(&bytes, &c, false).unwrap().simulation;
    for s in [&mut s, &mut r] {
        s.enqueue(9, n, 4, Command::Pause(false)).unwrap();
        s.step().unwrap();
        assert_eq!(s.movement().unwrap().units()[0].province(), ProvinceId(40));
        assert_eq!(s.movement().unwrap().units()[0].elapsed(), Fx::ZERO);
        assert_eq!(
            s.movement().unwrap().units()[0].remaining_route(),
            [ProvinceId(10)]
        );
        s.enqueue(
            10,
            n,
            5,
            Command::Stop {
                unit: DivisionId(900),
            },
        )
        .unwrap();
        s.step().unwrap();
        assert!(
            s.movement().unwrap().units()[0]
                .remaining_route()
                .is_empty()
        );
        assert_eq!(s.movement().unwrap().units()[0].province(), ProvinceId(40));
    }
    assert_eq!(s.export_save_v3().unwrap(), r.export_save_v3().unwrap());
    assert_eq!(s.state_hash().unwrap(), r.state_hash().unwrap());
}
