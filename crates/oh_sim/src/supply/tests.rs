use super::*;
fn p(i: u16) -> ProvinceId {
    ProvinceId(i)
}
fn q(i: i64) -> Qty {
    Qty::from_num(i)
}
fn fx(n: i64, d: i64) -> Fx {
    Fx::from_num(n) / Fx::from_num(d)
}
fn base() -> Input {
    Input {
        network: Network {
            nation: NationId(0),
            capital: p(0),
            nodes: (0..8)
                .map(|i| {
                    (
                        p(i),
                        Node {
                            controller: Some(NationId(0)),
                            land: true,
                        },
                    )
                })
                .collect(),
            land: BTreeMap::new(),
            rails: BTreeMap::new(),
            sources: vec![],
            decay: Fx::ZERO,
        },
        demands: vec![],
    }
}
fn land(x: &mut Input, a: u16, b: u16, c: i64) {
    x.land.insert(
        (p(a.min(b)), p(a.max(b))),
        LandEdge {
            cost: Fx::from_num(c),
            reverse_cost: Fx::from_num(c),
            kind: LandKind::Normal,
        },
    );
}
fn rail(x: &mut Input, a: u16, b: u16, c: i64, cap: i64) {
    land(x, a, b, c);
    x.rails.insert(
        (p(a.min(b)), p(a.max(b))),
        Rail {
            cost: Fx::from_num(c),
            reverse_cost: Fx::from_num(c),
            capacity: q(cap),
        },
    );
}
fn source(x: &mut Input, id: u64, node: u16, cap: i64) {
    x.sources.push(Source {
        id: SourceId(id),
        province: p(node),
        kind: if node == 0 {
            SourceKind::Capital
        } else {
            SourceKind::Hub
        },
        capacity: q(cap),
    });
}
fn demand(x: &mut Input, id: u32, node: u16, amount: i64) {
    x.demands.push(Demand {
        division: DivisionId(id),
        province: p(node),
        amount: q(amount),
    });
}
#[test]
fn finite_budget_and_actual_ratio() {
    let mut x = base();
    source(&mut x, 0, 0, 10);
    land(&mut x, 0, 1, 1);
    land(&mut x, 0, 2, 1);
    demand(&mut x, 0, 1, 10);
    demand(&mut x, 1, 2, 10);
    let y = calculate(&x).unwrap();
    assert_eq!(y.sources[&SourceId(0)].dispatch, q(10));
    for id in [1, 2] {
        assert_eq!(y.provinces[&p(id)].delivered, q(5));
        assert_eq!(y.provinces[&p(id)].ratio, fx(1, 2));
    }
}
#[test]
fn distance_loss_is_conserved() {
    let mut x = base();
    source(&mut x, 0, 0, 10);
    land(&mut x, 0, 1, 1);
    demand(&mut x, 0, 1, 10);
    x.decay = fx(1, 2);
    let y = calculate(&x).unwrap();
    let s = &y.sources[&SourceId(0)];
    assert_eq!((s.dispatch, s.delivered, s.loss), (q(10), q(5), q(5)));
}
#[test]
fn shared_feeder_capacity() {
    let mut x = base();
    rail(&mut x, 0, 1, 1, 10);
    rail(&mut x, 1, 2, 1, 100);
    rail(&mut x, 1, 3, 1, 100);
    source(&mut x, 2, 2, 10);
    source(&mut x, 3, 3, 10);
    demand(&mut x, 0, 2, 10);
    demand(&mut x, 1, 3, 10);
    let y = calculate(&x).unwrap();
    assert_eq!(y.rails[&(p(0), p(1))].dispatch, q(10));
    assert_eq!(y.provinces[&p(2)].delivered, q(5));
    assert_eq!(y.provinces[&p(3)].delivered, q(5));
}
#[test]
fn bottleneck_before_distance_and_cut() {
    let mut x = base();
    rail(&mut x, 0, 1, 1, 2);
    rail(&mut x, 1, 3, 1, 2);
    rail(&mut x, 0, 2, 3, 8);
    rail(&mut x, 2, 3, 3, 8);
    source(&mut x, 3, 3, 10);
    demand(&mut x, 0, 3, 10);
    let y = calculate(&x).unwrap();
    assert_eq!(y.sources[&SourceId(3)].feeder, Some(vec![p(0), p(2), p(3)]));
    assert_eq!(y.provinces[&p(3)].delivered, q(8));
    x.nodes.get_mut(&p(1)).unwrap().controller = Some(NationId(1));
    x.nodes.get_mut(&p(2)).unwrap().controller = Some(NationId(1));
    let z = calculate(&x).unwrap();
    assert_eq!(z.provinces[&p(3)].ratio, Fx::ZERO);
    assert_eq!(z.sources[&SourceId(3)].feeder, None);
}
#[test]
fn zero_isolated_and_excluded_strait() {
    let mut x = base();
    source(&mut x, 0, 0, 10);
    land(&mut x, 0, 1, 1);
    x.land.get_mut(&(p(0), p(1))).unwrap().kind = LandKind::Strait;
    demand(&mut x, 0, 1, 10);
    demand(&mut x, 1, 2, 0);
    let y = calculate(&x).unwrap();
    assert_eq!(y.provinces[&p(1)].reason, Reason::Isolated);
    assert_eq!(y.provinces[&p(2)].ratio, Fx::ONE);
}
#[test]
fn invalid_inputs_do_not_mutate() {
    let mut x = base();
    source(&mut x, 0, 0, 10);
    demand(&mut x, 0, 7, 1);
    x.demands[0].amount = Qty::from_bits(-1);
    let before = x.clone();
    assert_eq!(calculate(&x), Err(Error::InvalidValue));
    assert_eq!(x, before);
}

fn conserved(x: &Input, y: &Day) {
    let mut per_source = BTreeMap::<SourceId, (i128, i128)>::new();
    let mut per_rail = BTreeMap::<RailKey, i128>::new();
    for (&id, p) in &y.provinces {
        assert!(p.delivered >= Qty::ZERO && p.delivered <= p.demand);
        let delivered: i128 = p
            .shares
            .iter()
            .map(|s| i128::from(s.delivered.to_bits()))
            .sum();
        assert_eq!(delivered, i128::from(p.delivered.to_bits()));
        if p.demand > Qty::ZERO {
            assert_eq!(
                p.ratio.to_bits(),
                ((delivered << 32) / i128::from(p.demand.to_bits())) as i64
            );
        }
        for share in &p.shares {
            let entry = per_source.entry(share.source).or_default();
            entry.0 += i128::from(share.dispatch.to_bits());
            entry.1 += i128::from(share.delivered.to_bits());
            assert!(share.dispatch >= share.delivered);
            assert_eq!(
                i128::from(share.dispatch.to_bits()),
                if share.delivered == Qty::ZERO {
                    0
                } else {
                    ((i128::from(share.delivered.to_bits()) << 32)
                        + i128::from(share.attenuation.to_bits())
                        - 1)
                        / i128::from(share.attenuation.to_bits())
                }
            );
            assert!(x.nodes.contains_key(&id));
        }
    }
    for (&id, s) in &y.sources {
        let (dispatch, delivered) = per_source.get(&id).copied().unwrap_or_default();
        assert_eq!(dispatch, i128::from(s.dispatch.to_bits()));
        assert_eq!(delivered, i128::from(s.delivered.to_bits()));
        assert_eq!(
            s.loss.to_bits(),
            s.dispatch.to_bits() - s.delivered.to_bits()
        );
        let input = x.sources.iter().find(|v| v.id == id).unwrap();
        assert_eq!(
            s.unused.to_bits(),
            input.capacity.to_bits() - s.dispatch.to_bits()
        );
        if let Some(path) = &s.feeder {
            for pair in path.windows(2) {
                *per_rail.entry(graph::key(pair[0], pair[1])).or_default() += dispatch;
            }
        }
    }
    for (&key, r) in &y.rails {
        assert_eq!(
            i128::from(r.dispatch.to_bits()),
            per_rail.get(&key).copied().unwrap_or(0)
        );
        assert_eq!(
            r.unused.to_bits(),
            x.rails[&key].capacity.to_bits() - r.dispatch.to_bits()
        );
        assert!(r.unused >= Qty::ZERO);
    }
}
#[test]
fn fixed_weights_strand_capacity_and_unrelated_component_fills() {
    let mut x = base();
    land(&mut x, 0, 1, 1);
    land(&mut x, 0, 2, 1);
    land(&mut x, 1, 3, 1);
    rail(&mut x, 0, 3, 10, 100);
    rail(&mut x, 0, 4, 100, 100);
    source(&mut x, 0, 0, 20);
    source(&mut x, 3, 3, 20);
    source(&mut x, 4, 4, 100);
    demand(&mut x, 0, 1, 10);
    demand(&mut x, 1, 2, 10);
    demand(&mut x, 2, 4, 10);
    x.decay = fx(1, 2);
    let y = calculate(&x).unwrap();
    conserved(&x, &y);
    for id in [1, 2] {
        assert!((y.provinces[&p(id)].ratio.to_bits() - fx(2, 3).to_bits()).abs() < 13108);
        assert!(y.provinces[&p(id)].ratio < Fx::ONE);
    }
    assert_eq!(y.provinces[&p(4)].ratio, Fx::ONE);
    assert!(y.sources[&SourceId(3)].unused > q(13));
    // Adaptive counterexample: B dispatch20 supplies P10 and A dispatch20 supplies Q10.
    // Both fit caps with attenuation1/2; fixed weights intentionally cannot select that vector.
    assert_eq!(q(20) / q(2), q(10));
}
#[test]
fn widest_then_shortest_does_not_keep_wrong_prefix() {
    let mut x = base();
    rail(&mut x, 0, 1, 10, 10);
    rail(&mut x, 0, 2, 1, 8);
    rail(&mut x, 1, 2, 1, 8);
    rail(&mut x, 1, 3, 1, 5);
    source(&mut x, 3, 3, 10);
    demand(&mut x, 0, 3, 10);
    let y = calculate(&x).unwrap();
    assert_eq!(
        y.sources[&SourceId(3)].feeder,
        Some(vec![p(0), p(2), p(1), p(3)])
    );
}
#[test]
fn route_ties_are_lexicographic_and_input_order_independent() {
    let mut x = base();
    for (a, b) in [(0, 2), (2, 3), (0, 1), (1, 3)] {
        rail(&mut x, a, b, 1, 8);
    }
    source(&mut x, 3, 3, 10);
    source(&mut x, 0, 0, 10);
    demand(&mut x, 8, 3, 10);
    demand(&mut x, 0, 2, 1);
    let a = calculate(&x).unwrap();
    assert_eq!(a.sources[&SourceId(3)].feeder, Some(vec![p(0), p(1), p(3)]));
    x.sources.reverse();
    x.demands.reverse();
    assert_eq!(a, calculate(&x).unwrap());
}
#[test]
fn additive_sources_and_fractional_demand() {
    let mut x = base();
    rail(&mut x, 0, 1, 1, 100);
    source(&mut x, 0, 0, 6);
    source(&mut x, 1, 1, 6);
    demand(&mut x, 0, 1, 10);
    let y = calculate(&x).unwrap();
    assert_eq!(y.provinces[&p(1)].delivered, q(10));
    for id in [0, 1] {
        assert_eq!(y.sources[&SourceId(id)].dispatch, q(5));
    }
    x.demands[0].amount = Qty::from_bits(3);
    let z = calculate(&x).unwrap();
    conserved(&x, &z);
    assert_eq!(z.provinces[&p(1)].delivered, Qty::from_bits(2));
    assert_eq!(z.provinces[&p(1)].rounding_residual, Qty::from_bits(1));
    assert_eq!(z.provinces[&p(1)].target, Fx::ONE);
}
#[test]
fn no_capacity_out_of_range_and_lost_capital_control() {
    let mut x = base();
    source(&mut x, 0, 0, 10);
    land(&mut x, 0, 1, 1);
    demand(&mut x, 0, 1, 10);
    x.decay = Fx::ONE;
    assert_eq!(
        calculate(&x).unwrap().provinces[&p(1)].reason,
        Reason::OutOfRange
    );
    x.decay = Fx::ZERO;
    x.sources[0].capacity = Qty::ZERO;
    assert_eq!(
        calculate(&x).unwrap().provinces[&p(1)].reason,
        Reason::NoCapacity
    );
    x.sources[0].capacity = q(10);
    x.nodes.get_mut(&p(0)).unwrap().controller = Some(NationId(1));
    assert_eq!(
        calculate(&x).unwrap().provinces[&p(1)].reason,
        Reason::Isolated
    );
}
#[test]
fn references_duplicates_nonpositive_costs_and_negative_values_rejected() {
    let mut x = base();
    source(&mut x, 0, 0, 10);
    demand(&mut x, 0, 1, 1);
    let mut bad = x.clone();
    let duplicate_source = bad.sources[0].clone();
    bad.sources.push(duplicate_source);
    assert_eq!(calculate(&bad), Err(Error::Duplicate));
    let mut bad = x.clone();
    bad.demands.push(bad.demands[0].clone());
    assert_eq!(calculate(&bad), Err(Error::Duplicate));
    let mut bad = x.clone();
    bad.sources[0].kind = SourceKind::Hub;
    assert_eq!(calculate(&bad), Err(Error::InvalidReference));
    let mut bad = x.clone();
    bad.sources[0].province = p(9);
    assert_eq!(calculate(&bad), Err(Error::InvalidReference));
    let mut bad = x.clone();
    bad.sources[0].capacity = Qty::from_bits(-1);
    assert_eq!(calculate(&bad), Err(Error::InvalidValue));
    let mut bad = x.clone();
    land(&mut bad, 0, 1, 0);
    assert_eq!(calculate(&bad), Err(Error::InvalidValue));
    let mut bad = x.clone();
    bad.rails.insert(
        (p(0), p(1)),
        Rail {
            cost: Fx::ONE,
            reverse_cost: Fx::ONE,
            capacity: q(1),
        },
    );
    assert_eq!(calculate(&bad), Err(Error::InvalidReference));
    let mut bad = x.clone();
    bad.decay = Fx::from_bits(-1);
    assert_eq!(calculate(&bad), Err(Error::InvalidValue));
    let mut bad = x.clone();
    rail(&mut bad, 0, 1, 1, 1);
    bad.rails.get_mut(&(p(0), p(1))).unwrap().capacity = Qty::from_bits(-1);
    assert_eq!(calculate(&bad), Err(Error::InvalidValue));
}
#[test]
fn path_cost_and_grouped_demand_overflow_are_errors() {
    let mut x = base();
    source(&mut x, 0, 0, 10);
    land(&mut x, 0, 1, 1);
    land(&mut x, 1, 2, 1);
    x.land.get_mut(&(p(0), p(1))).unwrap().cost = Fx::from_bits(i64::MAX);
    assert_eq!(calculate(&x), Err(Error::Overflow));
    let mut x = base();
    source(&mut x, 0, 0, 10);
    demand(&mut x, 0, 0, 1);
    demand(&mut x, 1, 0, 1);
    x.demands[0].amount = Qty::from_bits(i64::MAX);
    assert_eq!(calculate(&x), Err(Error::Overflow));
}
#[test]
fn wide_trial_above_qty_range_is_infeasible_not_narrowed() {
    let mut x = base();
    source(&mut x, 0, 0, 1);
    land(&mut x, 0, 1, 1);
    demand(&mut x, 0, 1, 1);
    x.decay = Fx::ONE - Fx::from_bits(1);
    x.demands[0].amount = Qty::from_bits(i64::MAX);
    let y = calculate(&x).unwrap();
    assert_eq!(y.provinces[&p(1)].delivered, Qty::ZERO);
    conserved(&x, &y);
}
#[test]
fn quantized_small_cases_conserve_and_terminate() {
    for cap in 0..9 {
        for d1 in 0..7 {
            for d2 in 0..7 {
                for decay in [Fx::ZERO, fx(1, 2), Fx::ONE - Fx::from_bits(1)] {
                    let mut x = base();
                    source(&mut x, 0, 0, 1);
                    x.sources[0].capacity = Qty::from_bits(cap);
                    land(&mut x, 0, 1, 1);
                    land(&mut x, 0, 2, 1);
                    demand(&mut x, 0, 1, 1);
                    demand(&mut x, 1, 2, 1);
                    x.demands[0].amount = Qty::from_bits(d1);
                    x.demands[1].amount = Qty::from_bits(d2);
                    x.decay = decay;
                    let y = calculate(&x).unwrap();
                    conserved(&x, &y);
                }
            }
        }
    }
}
fn penalties() -> PenaltyDefines {
    PenaltyDefines {
        organization_floor: fx(1, 4),
        attack_floor: fx(1, 2),
        speed_floor: fx(1, 2),
        starvation_threshold: Fx::from_bits(1288490188),
        grace_days: 5,
        attrition_at_zero: fx(1, 16),
    }
}
#[test]
fn shortage_multipliers_are_positive_time_factors() {
    let a = shortage_effects(fx(1, 2), q(1), 0, penalties()).unwrap();
    assert_eq!(a.organization, fx(5, 8));
    assert_eq!(a.attack, fx(3, 4));
    assert_eq!(a.speed, fx(3, 4));
    assert_eq!(a.movement_time, Fx::ONE.checked_div(fx(3, 4)).unwrap());
    let z = shortage_effects(Fx::ZERO, q(1), 0, penalties()).unwrap();
    assert_eq!(z.movement_time, Fx::from_num(2));
}
#[test]
fn starvation_strict_threshold_grace_and_reset() {
    let defs = penalties();
    assert_eq!(
        shortage_effects(defs.starvation_threshold, q(1), 8, defs)
            .unwrap()
            .starvation_days,
        0
    );
    let day5 = shortage_effects(fx(1, 8), q(1), 4, defs).unwrap();
    assert_eq!(day5.starvation_days, 5);
    assert_eq!(day5.attrition_rate, Fx::ZERO);
    let day6 = shortage_effects(fx(1, 8), q(1), 5, defs).unwrap();
    assert_eq!(day6.starvation_days, 6);
    assert_eq!(day6.attrition_rate, fx(7, 128));
    let empty = shortage_effects(Fx::ZERO, Qty::ZERO, u32::MAX, defs).unwrap();
    assert_eq!(empty.starvation_days, 0);
    assert_eq!(empty.movement_time, Fx::ONE);
    assert_eq!(empty.attrition_rate, Fx::ZERO);
}
#[test]
fn penalty_invalid_values_and_counter_overflow_are_rejected() {
    let defs = penalties();
    assert_eq!(
        shortage_effects(Fx::ZERO, q(1), u32::MAX, defs),
        Err(Error::Overflow)
    );
    assert_eq!(
        shortage_effects(Fx::from_bits(-1), q(1), 0, defs),
        Err(Error::InvalidValue)
    );
    let mut bad = defs;
    bad.speed_floor = Fx::ZERO;
    assert_eq!(
        shortage_effects(Fx::ONE, q(1), 0, bad),
        Err(Error::InvalidValue)
    );
    bad.speed_floor = Fx::from_bits(1);
    assert_eq!(
        shortage_effects(Fx::ZERO, q(1), 0, bad),
        Err(Error::Overflow)
    );
    assert_eq!(
        shortage_effects(Fx::ONE, Qty::from_bits(-1), 0, defs),
        Err(Error::InvalidValue)
    );
}
fn tunables() -> BTreeMap<&'static str, i64> {
    include_str!("defines.toml")
        .lines()
        .filter_map(|line| {
            let line = line.split('#').next().unwrap().trim();
            let (k, v) = line.split_once('=')?;
            Some((k.trim(), v.trim().parse::<i64>().unwrap()))
        })
        .collect()
}
#[test]
fn provisional_data_values_are_consumed_in_sensitivity_vectors() {
    let data = tunables();
    let decay = Fx::from_bits(data["decay_per_reference_hour_fx_bits"]);
    let mut x = base();
    source(&mut x, 0, 0, 1);
    x.sources[0].capacity = Qty::from_bits(data["capital_capacity_qty_bits"]);
    land(&mut x, 0, 1, 64);
    demand(&mut x, 0, 1, 20);
    x.decay = decay;
    let y = calculate(&x).unwrap();
    assert_eq!(y.provinces[&p(1)].delivered, q(10));
    conserved(&x, &y);
    x.land.get_mut(&(p(0), p(1))).unwrap().cost = Fx::from_num(128);
    assert_eq!(
        calculate(&x).unwrap().provinces[&p(1)].reason,
        Reason::OutOfRange
    );
    x.land.get_mut(&(p(0), p(1))).unwrap().cost = Fx::from_num(64);
    x.decay = decay / Fx::from_num(2);
    assert_eq!(calculate(&x).unwrap().provinces[&p(1)].delivered, q(15));
    let defs = PenaltyDefines {
        organization_floor: Fx::from_bits(data["organization_floor_fx_bits"]),
        attack_floor: Fx::from_bits(data["attack_floor_fx_bits"]),
        speed_floor: Fx::from_bits(data["speed_floor_fx_bits"]),
        starvation_threshold: Fx::from_bits(data["starvation_threshold_fx_bits"]),
        grace_days: u32::try_from(data["starvation_grace_days"]).unwrap(),
        attrition_at_zero: Fx::from_bits(data["attrition_at_zero_fx_bits"]),
    };
    assert_eq!(defs, penalties());
    assert_eq!(
        shortage_effects(Fx::ZERO, q(1), 5, defs)
            .unwrap()
            .attrition_rate,
        Fx::from_bits(data["attrition_at_zero_fx_bits"])
    );
}
#[test]
fn rails_cannot_reference_sea_nodes_and_decay_overflow_is_error() {
    let mut x = base();
    rail(&mut x, 0, 1, 1, 10);
    x.nodes.get_mut(&p(1)).unwrap().land = false;
    assert_eq!(calculate(&x), Err(Error::InvalidReference));
    let mut x = base();
    source(&mut x, 0, 0, 10);
    land(&mut x, 0, 1, 1);
    x.decay = Fx::from_bits(i64::MAX);
    x.land.get_mut(&(p(0), p(1))).unwrap().cost = Fx::from_bits(i64::MAX);
    demand(&mut x, 0, 1, 1);
    assert_eq!(calculate(&x), Err(Error::Overflow));
}
#[test]
fn qty_above_fx_integer_range_is_not_narrowed() {
    let mut x = base();
    source(&mut x, 0, 0, 1);
    x.sources[0].capacity = Qty::from_bits(i64::MAX);
    demand(&mut x, 0, 0, 1);
    x.demands[0].amount = Qty::from_bits(i64::MAX);
    let y = calculate(&x).unwrap();
    assert_eq!(y.provinces[&p(0)].ratio, Fx::ONE);
    assert_eq!(y.sources[&SourceId(0)].dispatch, Qty::from_bits(i64::MAX));
    conserved(&x, &y);
}

#[test]
fn directed_feeder_and_distance_use_each_orientation_with_shared_capacity() {
    let mut x = base();
    for (a, b) in [(0, 1), (1, 3), (0, 2), (2, 3)] {
        rail(&mut x, a, b, 1, 8);
    }
    for key in [(p(0), p(1)), (p(1), p(3))] {
        let r = x.rails.get_mut(&key).unwrap();
        r.reverse_cost = Fx::from_num(9);
        let e = x.land.get_mut(&key).unwrap();
        e.reverse_cost = Fx::from_num(9);
    }
    for key in [(p(0), p(2)), (p(2), p(3))] {
        let r = x.rails.get_mut(&key).unwrap();
        r.cost = Fx::from_num(9);
        let e = x.land.get_mut(&key).unwrap();
        e.cost = Fx::from_num(9);
    }
    source(&mut x, 3, 3, 10);
    assert_eq!(
        x.network.feeders().unwrap()[&SourceId(3)],
        Some(vec![p(0), p(1), p(3)])
    );
    assert_eq!(
        graph::distances(&x.network, p(0)).unwrap()[&p(3)],
        Fx::from_num(2)
    );
    assert_eq!(
        graph::distances(&x.network, p(3)).unwrap()[&p(0)],
        Fx::from_num(2)
    );
    x.nodes.get_mut(&p(1)).unwrap().controller = None;
    assert_eq!(
        x.network.feeders().unwrap()[&SourceId(3)],
        Some(vec![p(0), p(2), p(3)])
    );
    x.nodes.get_mut(&p(2)).unwrap().controller = Some(NationId(1));
    assert_eq!(x.network.feeders().unwrap()[&SourceId(3)], None);
    // None has not become real NationId0, and capacities remain undirected canonical keys.
    assert_eq!(x.rails.len(), 4);
    assert_eq!(x.nodes[&p(0)].controller, Some(NationId(0)));
}
#[test]
fn nonpositive_reverse_cost_is_rejected() {
    let mut x = base();
    land(&mut x, 0, 1, 1);
    x.land.get_mut(&(p(0), p(1))).unwrap().reverse_cost = Fx::ZERO;
    assert_eq!(calculate(&x), Err(Error::InvalidValue));
    let mut x = base();
    rail(&mut x, 0, 1, 1, 1);
    x.rails.get_mut(&(p(0), p(1))).unwrap().reverse_cost = Fx::from_bits(-1);
    assert_eq!(calculate(&x), Err(Error::InvalidValue));
}
