use oh_core::{Fx, Qty};
use oh_sim::formula::{
    economy_allocate, economy_capacity, economy_construction, economy_minimum,
    economy_political_income, quantity_times_ratio,
};
fn q(v: i64) -> Qty {
    Qty::from_num(v)
}
#[test]
fn req_eco_01_quantity_keeps_wide_industry_and_checks_overflow() {
    assert_eq!(
        quantity_times_ratio(q(3_000_000_000), Fx::ONE).unwrap(),
        q(3_000_000_000)
    );
    assert!(quantity_times_ratio(Qty::MAX, Fx::from_num(2)).is_err());
}
#[test]
fn req_eco_02_exact_four_way_residual_and_minimum() {
    let ratios = [Fx::from_num(0.25); 4];
    assert_eq!(
        economy_allocate(Qty::from_bits(5), ratios, Fx::ZERO)
            .unwrap()
            .map(|v| v.to_bits()),
        [2, 1, 1, 1]
    );
    assert!(economy_allocate(Qty::ZERO, ratios, Fx::from_num(0.5)).is_err());
    assert!(economy_allocate(q(10), [Fx::ONE; 4], Fx::ZERO).is_err());
    assert_eq!(
        economy_minimum(Fx::from_num(0.25), Fx::from_num(0.5), Fx::from_num(0.5)).unwrap(),
        Fx::from_num(0.5)
    );
}
#[test]
fn req_eco_04_negative_resource_flow_is_rejected_by_quantity_arithmetic() {
    assert!(quantity_times_ratio(q(-1), Fx::ONE).is_err());
    assert!(quantity_times_ratio(q(1), Fx::from_num(-1)).is_err());
}
#[test]
fn req_eco_06_raw_ic_ceil_discard_and_zero_skip() {
    let (consumed, applied, discard) =
        economy_construction(Qty::from_bits(10), Qty::from_bits(3), Fx::from_num(2)).unwrap();
    assert_eq!(
        (consumed.to_bits(), applied.to_bits(), discard.to_bits()),
        (2, 3, 1)
    );
    assert_eq!(
        economy_construction(q(10), q(3), Fx::ZERO).unwrap(),
        (Qty::ZERO, Qty::ZERO, Qty::ZERO)
    );
    assert_eq!(
        economy_construction(Qty::ZERO, q(3), Fx::ONE).unwrap(),
        (Qty::ZERO, Qty::ZERO, Qty::ZERO)
    );
}
#[test]
fn req_eco_07_manpower_floor_capacity_is_not_daily_income() {
    assert_eq!(economy_capacity(1000, Fx::from_num(0.125)).unwrap(), 125);
    assert_eq!(economy_capacity(7, Fx::from_num(0.5)).unwrap(), 3);
    assert!(economy_capacity(-1, Fx::ONE).is_err());
    assert_eq!(economy_capacity(i64::MAX, Fx::ONE).unwrap(), i64::MAX);
}
#[test]
fn req_nat_02_pc_income_caps_without_intermediate_overflow() {
    assert_eq!(
        economy_political_income(Qty::MAX, Qty::MAX, Qty::MAX).unwrap(),
        Qty::MAX
    );
    assert_eq!(
        economy_political_income(q(98), q(3), q(100)).unwrap(),
        q(100)
    );
    assert!(economy_political_income(q(101), q(3), q(100)).is_err());
}
#[test]
fn req_nat_03_stability_and_mobilization_are_bounded_not_clamped() {
    assert!(economy_minimum(Fx::ZERO, Fx::ZERO, Fx::from_bits(Fx::ONE.to_bits() + 1)).is_err());
    assert!(economy_minimum(Fx::ZERO, Fx::ZERO, Fx::from_bits(-1)).is_err());
}
#[test]
fn req_nat_04_law_coefficient_and_consumer_minimum_validation() {
    assert!(economy_minimum(Fx::from_num(-1), Fx::ZERO, Fx::ONE).is_err());
    assert!(economy_minimum(Fx::ZERO, Fx::from_num(-1), Fx::ONE).is_err());
    assert_eq!(
        economy_minimum(Fx::ONE, Fx::MAX, Fx::ZERO).unwrap(),
        Fx::ONE
    );
}
