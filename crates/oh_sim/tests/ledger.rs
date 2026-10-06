//! REQ-UI-04: authoritative calculation and breakdown must preserve every bit.
use oh_core::{Fx, canonical_bytes, from_canonical_bytes};
use oh_sim::{
    formula,
    ledger::{LedgerError, LedgerOp, Modifier, ModifierOp, StatLedger},
};
use proptest::prelude::*;

const UNIT: i64 = 1_i64 << 32;
const STAT: &str = "fixture:stat";

fn modifier(source: &str, op: ModifierOp, bits: i64, expires: Option<u64>) -> Modifier {
    Modifier {
        source: source.into(),
        target_stat: STAT.into(),
        op,
        value: Fx::from_bits(bits),
        expires,
    }
}

#[test]
fn req_ui_04_base_add_mul_trace_and_applied_bits() {
    // Synthetic spec fixture: (12 + 3 - 2) * 1/2 * 3/2 = 39/4.
    let mut other = modifier("ignored", ModifierOp::Add, 100 * UNIT, None);
    other.target_stat = "fixture:other".into();
    let modifiers = vec![
        modifier("terrain:z", ModifierOp::Mul, 3 * UNIT / 2, None),
        modifier("law:z", ModifierOp::Add, -2 * UNIT, None),
        other,
        modifier("terrain:a", ModifierOp::Mul, UNIT / 2, None),
        modifier("law:a", ModifierOp::Add, 3 * UNIT, None),
    ];
    let ledger = StatLedger::evaluate(STAT, Fx::from_bits(12 * UNIT), &modifiers, 7).unwrap();
    assert_eq!(ledger.target_stat(), STAT);
    assert_eq!(ledger.tick(), 7);
    assert_eq!(ledger.value().to_bits(), 39 * UNIT / 4);
    let trace: Vec<_> = ledger
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.source.as_deref(),
                entry.op,
                entry.value.to_bits(),
                entry.accumulated.to_bits(),
            )
        })
        .collect();
    assert_eq!(
        trace,
        vec![
            (None, LedgerOp::Base, 12 * UNIT, 12 * UNIT),
            (Some("law:a"), LedgerOp::Add, 3 * UNIT, 15 * UNIT),
            (Some("law:z"), LedgerOp::Add, -2 * UNIT, 13 * UNIT),
            (Some("terrain:a"), LedgerOp::Mul, UNIT / 2, 13 * UNIT / 2),
            (
                Some("terrain:z"),
                LedgerOp::Mul,
                3 * UNIT / 2,
                39 * UNIT / 4
            ),
        ]
    );
    let applied = formula::stat_value(Fx::from_bits(12 * UNIT), STAT, &modifiers, 7).unwrap();
    ledger.verify_applied_value(applied).unwrap();
    assert_eq!(
        ledger.entries().last().unwrap().accumulated.to_bits(),
        applied.to_bits()
    );
    assert!(matches!(
        ledger.verify_applied_value(Fx::from_bits(applied.to_bits() + 1)),
        Err(LedgerError::AppliedValueMismatch { .. })
    ));
}

#[test]
fn req_ui_04_expiry_boundary_target_isolation_and_base_only() {
    let modifiers = vec![
        modifier("expired", ModifierOp::Add, UNIT, Some(0)),
        modifier("boundary", ModifierOp::Add, 2 * UNIT, Some(8)),
        modifier("future", ModifierOp::Add, 3 * UNIT, Some(9)),
        modifier("permanent", ModifierOp::Add, 4 * UNIT, None),
    ];
    assert_eq!(
        StatLedger::evaluate(STAT, Fx::ZERO, &modifiers, 7)
            .unwrap()
            .value()
            .to_bits(),
        9 * UNIT
    );
    let boundary = StatLedger::evaluate(STAT, Fx::ZERO, &modifiers, 8).unwrap();
    assert_eq!(boundary.value().to_bits(), 7 * UNIT);
    assert_eq!(boundary.entries().len(), 3);
    assert_eq!(
        StatLedger::evaluate(STAT, Fx::ZERO, &modifiers, 9)
            .unwrap()
            .value()
            .to_bits(),
        4 * UNIT
    );
    assert_eq!(
        StatLedger::evaluate(STAT, Fx::ZERO, &modifiers, u64::MAX)
            .unwrap()
            .value()
            .to_bits(),
        4 * UNIT
    );
    let base = StatLedger::evaluate("fixture:absent", Fx::from_bits(-17), &modifiers, 8).unwrap();
    assert_eq!(base.entries().len(), 1);
    assert_eq!(base.value().to_bits(), -17);
    assert_eq!(base.entries()[0].op, LedgerOp::Base);
    assert_eq!(
        StatLedger::evaluate(STAT, Fx::ZERO, &[], 0)
            .unwrap()
            .value(),
        Fx::ZERO
    );
}

#[test]
fn req_ui_04_negative_zero_and_order_sensitive_rounding() {
    // 3 raw units: (3 * 1/2) * 3/2 = 1, reversed multiplication gives 2.
    let modifiers = vec![
        modifier("z", ModifierOp::Mul, 3 * UNIT / 2, None),
        modifier("a", ModifierOp::Mul, UNIT / 2, None),
    ];
    assert_eq!(
        formula::stat_value(Fx::from_bits(3), STAT, &modifiers, 0)
            .unwrap()
            .to_bits(),
        1
    );
    assert_eq!(
        formula::stat_value(Fx::from_bits(-3), STAT, &modifiers, 0)
            .unwrap()
            .to_bits(),
        -3
    );
    let negative = [modifier("negative", ModifierOp::Mul, -UNIT, None)];
    assert_eq!(
        formula::stat_value(Fx::from_bits(3), STAT, &negative, 0)
            .unwrap()
            .to_bits(),
        -3
    );
    let zero = [modifier("zero", ModifierOp::Mul, 0, None)];
    assert_eq!(
        formula::stat_value(Fx::MAX, STAT, &zero, 0).unwrap(),
        Fx::ZERO
    );
}

#[test]
fn req_ui_04_modifier_serialization_preserves_bits_and_expiry() {
    for bits in [i64::MIN, -1, 0, 1, i64::MAX] {
        for expires in [None, Some(0), Some(u64::MAX)] {
            for op in [ModifierOp::Add, ModifierOp::Mul] {
                let original = modifier("fixture:한글", op, bits, expires);
                let restored: Modifier =
                    from_canonical_bytes(&canonical_bytes(&original).unwrap()).unwrap();
                assert_eq!(restored, original);
                assert_eq!(restored.value.to_bits(), bits);
            }
        }
    }
}

#[test]
fn req_ui_04_duplicates_invalid_identifiers_and_overflow_are_errors() {
    let duplicate = modifier("same", ModifierOp::Add, UNIT, None);
    assert!(matches!(
        StatLedger::evaluate(STAT, Fx::ZERO, &[duplicate.clone(), duplicate.clone()], 0),
        Err(LedgerError::DuplicateModifier { .. })
    ));
    let expired = modifier("same", ModifierOp::Add, UNIT, Some(0));
    assert!(StatLedger::evaluate(STAT, Fx::ZERO, &[duplicate.clone(), expired], 0).is_ok());
    let same_source_mul = modifier("same", ModifierOp::Mul, 2 * UNIT, None);
    assert_eq!(
        StatLedger::evaluate(STAT, Fx::ZERO, &[same_source_mul, duplicate], 0)
            .unwrap()
            .value()
            .to_bits(),
        2 * UNIT
    );
    assert!(matches!(
        StatLedger::evaluate("", Fx::ZERO, &[], 0),
        Err(LedgerError::EmptyTargetStat)
    ));
    assert!(matches!(
        StatLedger::evaluate(STAT, Fx::ZERO, &[modifier("", ModifierOp::Add, 0, None)], 0),
        Err(LedgerError::EmptySource)
    ));
    for (base, op, bits) in [
        (Fx::MAX, ModifierOp::Add, 1),
        (Fx::MIN, ModifierOp::Add, -1),
        (Fx::MAX, ModifierOp::Mul, 2 * UNIT),
        (Fx::MIN, ModifierOp::Mul, -UNIT),
    ] {
        let modifiers = [modifier("overflow", op, bits, None)];
        assert!(matches!(
            StatLedger::evaluate(STAT, base, &modifiers, 0),
            Err(LedgerError::Overflow { .. })
        ));
        assert!(matches!(
            formula::stat_value(base, STAT, &modifiers, 0),
            Err(LedgerError::Overflow { .. })
        ));
    }
    // Cancellation must not hide an intermediate checked overflow.
    let intermediate = [
        modifier("a", ModifierOp::Add, 1, None),
        modifier("z", ModifierOp::Add, -1, None),
    ];
    assert!(matches!(
        StatLedger::evaluate(STAT, Fx::MAX, &intermediate, 0),
        Err(LedgerError::Overflow { .. })
    ));
}

/// Independent raw i128 reference: arithmetic shift implements fixed floor.
fn reference(base: i64, modifiers: &[Modifier], tick: u64) -> Option<Vec<i64>> {
    let mut ordered: Vec<_> = modifiers
        .iter()
        .filter(|m| m.target_stat == STAT && m.expires.is_none_or(|expiry| tick < expiry))
        .collect();
    ordered.sort_by_key(|m| (m.op == ModifierOp::Mul, &m.source));
    let mut trace = vec![base];
    let mut raw = base;
    for m in ordered {
        let wide = match m.op {
            ModifierOp::Add => i128::from(raw) + i128::from(m.value.to_bits()),
            ModifierOp::Mul => (i128::from(raw) * i128::from(m.value.to_bits())) >> 32,
        };
        raw = i64::try_from(wide).ok()?;
        trace.push(raw);
    }
    Some(trace)
}

fn generated_modifiers(values: Vec<(i64, bool, Option<u64>, bool)>) -> Vec<Modifier> {
    values
        .into_iter()
        .enumerate()
        .map(|(index, (bits, mul, expiry, other))| {
            let mut m = modifier(
                &format!("source:{index:03}"),
                if mul {
                    ModifierOp::Mul
                } else {
                    ModifierOp::Add
                },
                bits,
                expiry,
            );
            if other {
                m.target_stat = "fixture:other".into();
            }
            m
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 512,
        failure_persistence: None,
        rng_seed: proptest::test_runner::RngSeed::Fixed(20261006),
        .. ProptestConfig::default()
    })]

    #[test]
    fn req_ui_04_ledger_matches_independent_reference_and_permutations(
        base in -10 * UNIT..=10 * UNIT,
        values in prop::collection::vec((-2 * UNIT..=2 * UNIT, any::<bool>(), prop::option::of(0_u64..20), any::<bool>()), 0..12),
        tick in 0_u64..20,
    ) {
        let modifiers = generated_modifiers(values);
        let expected = reference(base, &modifiers, tick).unwrap();
        let ledger = StatLedger::evaluate(STAT, Fx::from_bits(base), &modifiers, tick).unwrap();
        let applied = formula::stat_value(Fx::from_bits(base), STAT, &modifiers, tick).unwrap();
        let actual: Vec<_> = ledger.entries().iter().map(|e| e.accumulated.to_bits()).collect();
        prop_assert_eq!(actual, expected);
        prop_assert_eq!(ledger.value().to_bits(), applied.to_bits());
        prop_assert!(ledger.verify_applied_value(applied).is_ok());
        let mut reversed = modifiers.clone();
        reversed.reverse();
        let other = StatLedger::evaluate(STAT, Fx::from_bits(base), &reversed, tick).unwrap();
        prop_assert_eq!(canonical_bytes(&ledger).unwrap(), canonical_bytes(&other).unwrap());
    }

    #[test]
    fn req_ui_04_full_bit_range_matches_reference_or_checked_overflow(
        base in any::<i64>(),
        values in prop::collection::vec((any::<i64>(), any::<bool>(), prop::option::of(0_u64..20), any::<bool>()), 0..12),
        tick in 0_u64..20,
    ) {
        let modifiers = generated_modifiers(values);
        let ledger = StatLedger::evaluate(STAT, Fx::from_bits(base), &modifiers, tick);
        let applied = formula::stat_value(Fx::from_bits(base), STAT, &modifiers, tick);
        match reference(base, &modifiers, tick) {
            Some(expected) => {
                let ledger = ledger.unwrap();
                let actual: Vec<_> = ledger.entries().iter().map(|e| e.accumulated.to_bits()).collect();
                prop_assert_eq!(actual, expected);
                prop_assert_eq!(ledger.value().to_bits(), applied.unwrap().to_bits());
            }
            None => {
                prop_assert!(matches!(ledger, Err(LedgerError::Overflow { .. })), "ledger must reject overflow");
                prop_assert!(matches!(applied, Err(LedgerError::Overflow { .. })), "formula must reject overflow");
            }
        }
    }
}
