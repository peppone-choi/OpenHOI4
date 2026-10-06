use std::collections::BTreeMap;

use oh_core::{
    DivisionId, EntityId, Fnv1a64, Fx, GameDay, NationId, ProvinceId, Qty, RngKey, StateId,
    SystemId, canonical_bytes, fnv1a64, from_canonical_bytes, simulation_rng, state_hash,
};
use proptest::prelude::*;
use rand_chacha::rand_core::Rng;
use serde::{Deserialize, Serialize};

mod common;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Snapshot {
    day: GameDay,
    amounts: BTreeMap<NationId, Qty>,
    ratio: Fx,
    rolls: Vec<u64>,
}

fn key() -> RngKey {
    RngKey::new(7, SystemId(11), GameDay(365), NationId(3))
}

fn rolls(key: RngKey) -> Vec<u64> {
    let mut rng = simulation_rng(key);
    (0..16).map(|_| rng.next_u64()).collect()
}

#[test]
fn req_gen_04_fixed_exact_arithmetic_and_rounding() {
    assert_eq!(Fx::from_num(3) / Fx::from_num(2), Fx::from_bits(3 << 31));
    assert_eq!(Qty::from_num(3) / Qty::from_num(2), Qty::from_bits(3 << 15));
    // Integer division truncates toward zero; fixed multiplication rounds down.
    assert_eq!((Fx::from_bits(-5) / Fx::from_num(2)).to_bits(), -2);
    assert_eq!((Fx::from_bits(-5) * Fx::from_bits(1 << 31)).to_bits(), -3);
    assert_eq!((Qty::from_bits(-5) / Qty::from_num(2)).to_bits(), -2);
    assert_eq!((Qty::from_bits(-5) * Qty::from_bits(1 << 15)).to_bits(), -3);
    assert!(Fx::MAX.checked_add(Fx::from_bits(1)).is_none());
    assert!(Qty::MIN.checked_sub(Qty::from_bits(1)).is_none());
    assert!(Fx::MAX.checked_mul(Fx::from_num(2)).is_none());
    assert!(Qty::MAX.checked_mul(Qty::from_num(2)).is_none());
    assert!(Fx::from_num(1).checked_div(Fx::ZERO).is_none());
    assert!(Qty::from_num(1).checked_div(Qty::ZERO).is_none());
}

#[test]
fn req_gen_04_fixed_and_id_wire_encoding() {
    // Postcard signed integers use zigzag varints, never native memory layout.
    assert_eq!(canonical_bytes(&Fx::from_bits(1)).unwrap(), [2]);
    assert_eq!(canonical_bytes(&Qty::from_bits(-2)).unwrap(), [3]);
    assert_eq!(canonical_bytes(&ProvinceId(128)).unwrap(), [128, 1]);
    assert_eq!(canonical_bytes(&StateId(128)).unwrap(), [128, 1]);
    assert_eq!(canonical_bytes(&NationId(128)).unwrap(), [128, 1]);
    assert_eq!(
        canonical_bytes(&DivisionId(u32::MAX)).unwrap(),
        [255, 255, 255, 255, 15]
    );
    assert_eq!(std::mem::size_of::<ProvinceId>(), 2);
    assert_eq!(std::mem::size_of::<DivisionId>(), 4);
    assert!(NationId(2) < NationId(10));
    assert_eq!(ProvinceId::from(42).get(), 42);
    assert_eq!(u32::from(DivisionId(42)), 42);
    for bits in [i64::MIN, i64::MAX, -1, 0, 1] {
        let values = (Fx::from_bits(bits), Qty::from_bits(bits));
        let decoded: (Fx, Qty) = from_canonical_bytes(&canonical_bytes(&values).unwrap()).unwrap();
        assert_eq!(decoded, values);
    }
}

#[test]
fn req_gen_04_rng_tuple_fields_and_entity_domains_are_distinct() {
    let original = key();
    let variants = [
        RngKey {
            game_seed: 8,
            ..original
        },
        RngKey {
            system: SystemId(12),
            ..original
        },
        RngKey {
            day: GameDay(366),
            ..original
        },
        RngKey::new(7, SystemId(11), GameDay(365), NationId(4)),
        RngKey::new(7, SystemId(11), GameDay(365), ProvinceId(3)),
        RngKey::new(7, SystemId(11), GameDay(365), StateId(3)),
        RngKey::new(7, SystemId(11), GameDay(365), DivisionId(3)),
    ];
    assert_eq!(rolls(original), rolls(original));
    for (index, variant) in variants.iter().enumerate() {
        assert_ne!(original.seed_bytes(), variant.seed_bytes());
        assert_ne!(rolls(original), rolls(*variant));
        for other in &variants[..index] {
            assert_ne!(variant.seed_bytes(), other.seed_bytes());
            assert_ne!(rolls(*variant), rolls(*other));
        }
    }
    let expected = [
        7, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 109, 1, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 3, 0, 0, 0, 0,
        0, 0, 0,
    ];
    assert_eq!(original.seed_bytes(), expected);
}

#[test]
fn req_gen_04_rng_order_and_unrelated_draws_do_not_affect_entity_streams() {
    fn evaluate(ids: &[u16]) -> BTreeMap<NationId, Vec<u64>> {
        ids.iter()
            .map(|&id| {
                let id = NationId(id);
                let key = RngKey::new(7, SystemId(11), GameDay(365), id);
                (id, rolls(key))
            })
            .collect()
    }
    let before = evaluate(&[1, 2, 3, 4]);
    let mut unrelated = simulation_rng(RngKey::new(7, SystemId(12), GameDay(365), NationId(1)));
    for _ in 0..1000 {
        unrelated.next_u64();
    }
    assert_eq!(before, evaluate(&[4, 2, 1, 3]));
    let mut same = simulation_rng(key());
    let first = same.next_u64();
    let second = same.next_u64();
    assert_eq!([first, second], rolls(key())[..2]);
    assert_ne!(first, second);
}

#[test]
fn req_gen_04_canonical_map_order_and_strict_decode() {
    let forward = BTreeMap::from([
        (NationId(1), Qty::from_bits(2)),
        (NationId(2), Qty::from_bits(-3)),
    ]);
    let mut reverse = BTreeMap::new();
    reverse.insert(NationId(2), Qty::from_bits(-3));
    reverse.insert(NationId(1), Qty::from_bits(2));
    let bytes = canonical_bytes(&forward).unwrap();
    assert_eq!(bytes, [2, 1, 4, 2, 5]);
    assert_eq!(bytes, canonical_bytes(&reverse).unwrap());
    assert_eq!(state_hash(&forward).unwrap(), state_hash(&reverse).unwrap());
    let decoded: BTreeMap<NationId, Qty> = from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded, forward);
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(from_canonical_bytes::<BTreeMap<NationId, Qty>>(&trailing).is_err());
    assert!(from_canonical_bytes::<BTreeMap<NationId, Qty>>(&bytes[..bytes.len() - 1]).is_err());
    assert!(from_canonical_bytes::<NationId>(&[255; 20]).is_err());
    assert!(from_canonical_bytes::<NationId>(&[]).is_err());
}

#[test]
fn req_gen_04_fnv1a_reference_vectors_and_incremental_hash() {
    assert_eq!(fnv1a64(b""), 0xcbf29ce484222325);
    assert_eq!(fnv1a64(b"a"), 0xaf63dc4c8601ec8c);
    assert_eq!(fnv1a64(b"foobar"), 0x85944171f73967e8);
    let mut hash = Fnv1a64::new();
    hash.update(b"foo");
    hash.update(b"");
    hash.update(b"bar");
    assert_eq!(hash.finish(), fnv1a64(b"foobar"));
    assert_eq!(Fnv1a64::default().finish(), fnv1a64(b""));
}

#[test]
fn req_gen_04_serialization_errors_are_propagated() {
    struct CannotSerialize;
    impl Serialize for CannotSerialize {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("intentional test error"))
        }
    }
    assert!(canonical_bytes(&CannotSerialize).is_err());
    assert!(state_hash(&CannotSerialize).is_err());
}

#[test]
fn req_plat_02_portable_postcard_hash_vector() {
    let snapshot = Snapshot {
        day: GameDay(1),
        amounts: BTreeMap::from([(NationId(2), Qty::from_bits(-3))]),
        ratio: Fx::from_bits(4),
        rolls: vec![5, 6],
    };
    let bytes = canonical_bytes(&snapshot).unwrap();
    assert_eq!(bytes, [1, 1, 2, 5, 8, 2, 5, 6]);
    // Independent FNV reference calculation is recorded in WP-02 evidence.
    assert_eq!(state_hash(&snapshot).unwrap(), 0x45711636651ea5db);
    assert_eq!(from_canonical_bytes::<Snapshot>(&bytes).unwrap(), snapshot);
    assert_ne!(
        state_hash(&Snapshot {
            day: GameDay(2),
            ..snapshot
        })
        .unwrap(),
        fnv1a64(&bytes)
    );
}

#[test]
fn req_plat_02_chacha8_zero_seed_reference_vector() {
    let mut rng = simulation_rng(RngKey::new(0, SystemId(0), GameDay(0), EntityId::Global));
    let actual: Vec<u32> = (0..8).map(|_| rng.next_u32()).collect();
    assert_eq!(
        actual,
        [
            0x2fef003e, 0xd6405f89, 0xe8b85b7f, 0xa1a5091f, 0xc30e842c, 0x3b7f9ace, 0x88e11b18,
            0x1e1a71ef
        ]
    );
}

#[test]
fn req_plat_02_full_diagnostic_matches_independent_reference() {
    let snapshot = common::diagnostic_snapshot();
    let bytes = canonical_bytes(&snapshot).unwrap();
    let reference = include_str!("fixtures/core-format-v1.hex").trim();
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(hex, reference);
    let expected_hash =
        u64::from_str_radix(include_str!("fixtures/core-format-v1.hash").trim(), 16).unwrap();
    assert_eq!(state_hash(&snapshot).unwrap(), expected_hash);
    assert_eq!(
        from_canonical_bytes::<common::DiagnosticSnapshot>(&bytes).unwrap(),
        snapshot
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: None,
        rng_seed: proptest::test_runner::RngSeed::Fixed(20261006),
        .. ProptestConfig::default()
    })]

    #[test]
    fn req_gen_04_fixed_serialization_preserves_all_bits(fx in any::<i64>(), qty in any::<i64>()) {
        let values = (Fx::from_bits(fx), Qty::from_bits(qty));
        let decoded: (Fx, Qty) = from_canonical_bytes(&canonical_bytes(&values).unwrap()).unwrap();
        prop_assert_eq!(decoded.0.to_bits(), fx);
        prop_assert_eq!(decoded.1.to_bits(), qty);
    }

    #[test]
    fn req_gen_04_fixed_checked_arithmetic_matches_integer_reference(a in any::<i64>(), b in any::<i64>()) {
        prop_assert_eq!(Fx::from_bits(a).checked_add(Fx::from_bits(b)).map(Fx::to_bits), a.checked_add(b));
        prop_assert_eq!(Qty::from_bits(a).checked_sub(Qty::from_bits(b)).map(Qty::to_bits), a.checked_sub(b));
        let fx_product = (i128::from(a) * i128::from(b)) >> 32;
        let qty_product = (i128::from(a) * i128::from(b)) >> 16;
        prop_assert_eq!(Fx::from_bits(a).checked_mul(Fx::from_bits(b)).map(Fx::to_bits), i64::try_from(fx_product).ok());
        prop_assert_eq!(Qty::from_bits(a).checked_mul(Qty::from_bits(b)).map(Qty::to_bits), i64::try_from(qty_product).ok());
        if b != 0 {
            let fx_quotient = (i128::from(a) << 32) / i128::from(b);
            let qty_quotient = (i128::from(a) << 16) / i128::from(b);
            prop_assert_eq!(Fx::from_bits(a).checked_div(Fx::from_bits(b)).map(Fx::to_bits), i64::try_from(fx_quotient).ok());
            prop_assert_eq!(Qty::from_bits(a).checked_div(Qty::from_bits(b)).map(Qty::to_bits), i64::try_from(qty_quotient).ok());
        }
    }

    #[test]
    fn req_gen_04_typed_ids_round_trip(province in any::<u16>(), division in any::<u32>()) {
        let ids = (ProvinceId(province), StateId(province), NationId(province), DivisionId(division));
        let decoded: (ProvinceId, StateId, NationId, DivisionId) = from_canonical_bytes(&canonical_bytes(&ids).unwrap()).unwrap();
        prop_assert_eq!(decoded, ids);
    }

    #[test]
    fn req_gen_04_rng_reconstruction(seed in any::<u64>(), system in any::<u32>(), day in any::<u64>(), entity in any::<u32>()) {
        let key = RngKey::new(seed, SystemId(system), GameDay(day), DivisionId(entity));
        let decoded: RngKey = from_canonical_bytes(&canonical_bytes(&key).unwrap()).unwrap();
        prop_assert_eq!(key, decoded);
        prop_assert_eq!(rolls(key), rolls(decoded));
        prop_assert_ne!(key.seed_bytes(), RngKey { game_seed: seed ^ 1, ..key }.seed_bytes());
        prop_assert_ne!(key.seed_bytes(), RngKey { system: SystemId(system ^ 1), ..key }.seed_bytes());
        prop_assert_ne!(key.seed_bytes(), RngKey { day: GameDay(day ^ 1), ..key }.seed_bytes());
        prop_assert_ne!(key.seed_bytes(), RngKey { entity: EntityId::from(DivisionId(entity ^ 1)), ..key }.seed_bytes());
    }

    #[test]
    fn req_gen_04_map_permutations_are_canonical(entries in prop::collection::vec((any::<u16>(), any::<i64>()), 0..100)) {
        let forward: BTreeMap<_, _> = entries.iter().map(|&(id, bits)| (NationId(id), Qty::from_bits(bits))).collect();
        // Iterate the final unique entries backwards: duplicate insertion semantics are not reordered.
        let reverse: BTreeMap<_, _> = forward.iter().rev().map(|(&id, &qty)| (id, qty)).collect();
        prop_assert_eq!(canonical_bytes(&forward).unwrap(), canonical_bytes(&reverse).unwrap());
        prop_assert_eq!(state_hash(&forward).unwrap(), state_hash(&reverse).unwrap());
    }

    #[test]
    fn req_gen_04_fnv_chunking_is_independent(bytes in prop::collection::vec(any::<u8>(), 0..2048), split in any::<usize>()) {
        let split = split % (bytes.len() + 1);
        let mut hash = Fnv1a64::new();
        hash.update(&bytes[..split]);
        hash.update(&bytes[split..]);
        prop_assert_eq!(hash.finish(), fnv1a64(&bytes));
    }
}
