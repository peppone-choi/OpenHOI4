use oh_core::{Fx, NationId, ProvinceId, Qty};
use oh_data::{DefineValue, Defines, Number, map::EdgeKind};
use oh_sim::{
    supply::world_adapter::*,
    supply::{Source, SourceId, SourceKind},
    world::World,
};
use std::collections::BTreeMap;
fn loaded() -> oh_data::national::LoadedNational {
    oh_data::national::load_scenario(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap()
}
fn defines() -> Defines {
    Defines(BTreeMap::from([(
        "movement".into(),
        BTreeMap::from([
            (
                "terrain_plains".into(),
                DefineValue::Number(Number::Fixed(Fx::ONE)),
            ),
            (
                "terrain_hills".into(),
                DefineValue::Number(Number::Fixed(Fx::from_num(2))),
            ),
            (
                "river_normal".into(),
                DefineValue::Number(Number::Fixed(Fx::ONE)),
            ),
            (
                "river_small".into(),
                DefineValue::Number(Number::Fixed(Fx::from_num(3))),
            ),
            (
                "river_large".into(),
                DefineValue::Number(Number::Fixed(Fx::from_num(4))),
            ),
        ]),
    )]))
}
fn tuning() -> TransportTuning {
    TransportTuning {
        reference_speed_kmh: Fx::from_num(4),
        decay_per_reference_hour: Fx::ONE / Fx::from_num(128),
        infrastructure_factors: BTreeMap::from([
            (Fx::ZERO, Fx::ONE),
            (Fx::ONE, Fx::ONE),
            (Fx::from_num(2), Fx::ONE),
            (Fx::from_num(3), Fx::ONE),
            (Fx::from_num(4), Fx::ONE),
            (Fx::from_num(5), Fx::ONE),
        ]),
        rail_capacity_by_level: BTreeMap::from([(1, Qty::from_num(10)), (2, Qty::from_num(20))]),
    }
}
fn capital(w: &World) -> StaticMetadata {
    let c = w.defs().nations()[0].capital;
    StaticMetadata {
        sources: vec![Source {
            id: SourceId(0),
            province: ProvinceId(c),
            kind: SourceKind::Capital,
            capacity: Qty::from_num(20),
        }],
        rails: vec![],
    }
}
#[test]
fn actual_world_capital_controller_and_refs() {
    let d = loaded();
    let w = World::from_loaded(&d).unwrap();
    let n = w.inputs().nations()[0].id();
    let result = prepare(&w, &defines(), n, &tuning(), &capital(&w)).unwrap();
    assert_eq!(result.capital, ProvinceId(d.nations[0].capital));
    for p in w.inputs().provinces() {
        assert_eq!(result.nodes[&p.id()].controller, p.controller());
    }
    assert_eq!(
        result.feeders().unwrap()[&SourceId(0)],
        Some(vec![result.capital])
    );
}
#[test]
fn actual_map_directional_terrain_river_and_infrastructure() {
    let mut d = loaded();
    d.map.edges = vec![oh_data::map::Edge {
        a: 10,
        b: 20,
        kind: EdgeKind::RiverSmall,
        distance_km: Fx::from_num(8),
    }];
    for p in &mut d.map.provinces {
        p.terrain = if p.id == 20 { "hills" } else { "plains" }.into();
    }
    d.map.states[0].infrastructure = 1;
    d.map.states[1].infrastructure = 2;
    let w = World::from_loaded(&d).unwrap();
    let mut t = tuning();
    t.infrastructure_factors.insert(Fx::ONE, Fx::from_num(2));
    t.infrastructure_factors
        .insert(Fx::from_num(2), Fx::from_num(3));
    let net = prepare(
        &w,
        &defines(),
        w.inputs().nations()[0].id(),
        &t,
        &capital(&w),
    )
    .unwrap();
    let e = &net.land[&(ProvinceId(10), ProvinceId(20))];
    let infra_to = w
        .state(w.province(ProvinceId(20)).unwrap().state().unwrap())
        .unwrap()
        .infrastructure();
    let infra_from = w
        .state(w.province(ProvinceId(10)).unwrap().state().unwrap())
        .unwrap()
        .infrastructure();
    assert_eq!(
        e.cost,
        Fx::from_num(2) * Fx::from_num(2) * t.infrastructure_factors[&infra_to] * Fx::from_num(3)
    );
    assert_eq!(
        e.reverse_cost,
        Fx::from_num(2) * t.infrastructure_factors[&infra_from] * Fx::from_num(3)
    );
    assert_ne!(e.cost, e.reverse_cost);
}
#[test]
fn missing_infrastructure_coefficient_is_not_defaulted() {
    let d = loaded();
    let w = World::from_loaded(&d).unwrap();
    let mut t = tuning();
    t.infrastructure_factors.clear();
    assert_eq!(
        prepare(
            &w,
            &defines(),
            w.inputs().nations()[0].id(),
            &t,
            &capital(&w)
        ),
        Err(AdapterError::MissingContext)
    );
}
#[test]
fn nation_zero_and_actual_uncontrolled_water_are_distinct() {
    let mut d = loaded();
    d.nations[0].id = 0;
    let w = World::from_loaded(&d).unwrap();
    let n = prepare(&w, &defines(), NationId(0), &tuning(), &capital(&w)).unwrap();
    assert_eq!(n.nodes[&n.capital].controller, Some(NationId(0)));
    let water: Vec<_> = w
        .inputs()
        .provinces()
        .iter()
        .filter(|p| p.controller().is_none())
        .collect();
    assert!(!water.is_empty());
    for p in water {
        assert_eq!(n.nodes[&p.id()].controller, None);
        assert!(!n.nodes[&p.id()].land);
    }
}

#[test]
fn static_rails_use_real_edges_and_exact_level_capacity() {
    let d = loaded();
    let w = World::from_loaded(&d).unwrap();
    let mut meta = capital(&w);
    let edge = w
        .defs()
        .map()
        .edges
        .iter()
        .find(|e| e.a == 10 && e.b == 20)
        .unwrap();
    meta.rails.push(StaticRail {
        edge: (ProvinceId(edge.a), ProvinceId(edge.b)),
        level: 2,
    });
    let net = prepare(
        &w,
        &defines(),
        w.inputs().nations()[0].id(),
        &tuning(),
        &meta,
    )
    .unwrap();
    let r = &net.rails[&meta.rails[0].edge];
    assert_eq!(r.capacity, Qty::from_num(20));
    assert_eq!(r.cost, net.land[&meta.rails[0].edge].cost);
    assert_eq!(r.reverse_cost, net.land[&meta.rails[0].edge].reverse_cost);
    meta.rails[0].level = 3;
    assert_eq!(
        prepare(
            &w,
            &defines(),
            w.inputs().nations()[0].id(),
            &tuning(),
            &meta
        ),
        Err(AdapterError::MissingContext)
    );
}
#[test]
fn hub_and_coastal_port_require_actual_instance_authority() {
    let mut d = loaded();
    d.map
        .provinces
        .iter_mut()
        .find(|p| p.id == 20)
        .unwrap()
        .coastal = false;
    let w = World::from_loaded(&d).unwrap();
    let mut meta = capital(&w);
    meta.sources.push(Source {
        id: SourceId(1),
        province: ProvinceId(20),
        kind: SourceKind::Hub,
        capacity: Qty::from_num(10),
    });
    let nation = w.inputs().nations()[0].id();
    assert_eq!(
        prepare(&w, &defines(), nation, &tuning(), &meta),
        Err(AdapterError::UnsupportedBuildingInstance)
    );
    meta.sources[1].kind = SourceKind::Port;
    assert_eq!(
        prepare(&w, &defines(), nation, &tuning(), &meta),
        Err(AdapterError::InvalidReference)
    );
    d.map
        .provinces
        .iter_mut()
        .find(|p| p.id == 20)
        .unwrap()
        .coastal = true;
    let w = World::from_loaded(&d).unwrap();
    assert_eq!(
        prepare(&w, &defines(), nation, &tuning(), &meta),
        Err(AdapterError::UnsupportedBuildingInstance)
    );
}
#[test]
fn current_controller_cut_does_not_use_ownership() {
    let mut d = loaded();
    let capital_id = d.nations[0].capital;
    d.scenario
        .control_overrides
        .insert(capital_id, d.nations[1].tag.clone());
    let w = World::from_loaded(&d).unwrap();
    let nation = w.inputs().nations()[0].id();
    assert_eq!(
        w.province(ProvinceId(capital_id)).unwrap().owner(),
        Some(nation)
    );
    assert_ne!(
        w.province(ProvinceId(capital_id)).unwrap().controller(),
        Some(nation)
    );
    let net = prepare(&w, &defines(), nation, &tuning(), &capital(&w)).unwrap();
    assert_eq!(net.feeders().unwrap()[&SourceId(0)], None);
}
#[test]
fn authoritative_fractional_infrastructure_and_expiry_need_exact_mapping() {
    let mut d = loaded();
    d.scenario.state_modifiers.insert(
        1,
        vec![oh_data::national::ModifierInput {
            source: "supply-test".into(),
            target_stat: "infrastructure".into(),
            operation: "add".into(),
            value: "0.125".into(),
            expires: Some(1),
        }],
    );
    let w = World::from_loaded(&d).unwrap();
    let nation = w.inputs().nations()[0].id();
    let mut t = tuning();
    assert_eq!(
        w.state(oh_core::StateId(1)).unwrap().infrastructure(),
        Fx::ONE + Fx::ONE / Fx::from_num(8)
    );
    assert_eq!(
        prepare(&w, &defines(), nation, &t, &capital(&w)),
        Err(AdapterError::MissingContext)
    );
    t.infrastructure_factors.insert(
        Fx::ONE + Fx::ONE / Fx::from_num(8),
        Fx::ONE / Fx::from_num(2),
    );
    let initial = prepare(&w, &defines(), nation, &t, &capital(&w)).unwrap();
    let mut sim = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&d.pack.defines).unwrap(),
        w,
    )
    .unwrap();
    sim.step().unwrap();
    let current = sim.world().unwrap();
    assert_eq!(
        current.state(oh_core::StateId(1)).unwrap().infrastructure(),
        Fx::ONE
    );
    let later = prepare(current, &defines(), nation, &t, &capital(current)).unwrap();
    assert_ne!(
        initial.land[&(ProvinceId(10), ProvinceId(20))].cost,
        later.land[&(ProvinceId(10), ProvinceId(20))].cost
    );
    t.infrastructure_factors.remove(&Fx::ONE);
    assert_eq!(
        prepare(current, &defines(), nation, &t, &capital(current)),
        Err(AdapterError::MissingContext)
    );
}
#[test]
fn invalid_coefficients_missing_defines_refs_and_static_duplicates() {
    let d = loaded();
    let w = World::from_loaded(&d).unwrap();
    let n = w.inputs().nations()[0].id();
    let meta = capital(&w);
    let t = tuning();
    assert_eq!(
        prepare(&w, &defines(), NationId(999), &t, &meta),
        Err(AdapterError::InvalidReference)
    );
    let mut bad = t.clone();
    bad.reference_speed_kmh = Fx::ZERO;
    assert_eq!(
        prepare(&w, &defines(), n, &bad, &meta),
        Err(AdapterError::InvalidValue)
    );
    let mut bad = t.clone();
    bad.infrastructure_factors.insert(Fx::ONE, Fx::ZERO);
    assert_eq!(
        prepare(&w, &defines(), n, &bad, &meta),
        Err(AdapterError::InvalidValue)
    );
    let mut bad = t.clone();
    bad.infrastructure_factors
        .insert(Fx::ZERO, Fx::from_bits(-1));
    assert_eq!(
        prepare(&w, &defines(), n, &bad, &meta),
        Err(AdapterError::InvalidValue)
    );
    let mut bad = t.clone();
    bad.decay_per_reference_hour = Fx::from_bits(-1);
    assert_eq!(
        prepare(&w, &defines(), n, &bad, &meta),
        Err(AdapterError::InvalidValue)
    );
    let mut defs = defines();
    defs.0.get_mut("movement").unwrap().remove("terrain_plains");
    assert_eq!(
        prepare(&w, &defs, n, &t, &meta),
        Err(AdapterError::MissingContext)
    );
    let mut duplicate = meta.clone();
    duplicate.sources.push(duplicate.sources[0].clone());
    assert_eq!(
        prepare(&w, &defines(), n, &t, &duplicate),
        Err(AdapterError::Graph(oh_sim::supply::Error::Duplicate))
    );
    let mut wrong = meta.clone();
    wrong.sources[0].province = ProvinceId(20);
    assert_eq!(
        prepare(&w, &defines(), n, &t, &wrong),
        Err(AdapterError::Graph(oh_sim::supply::Error::InvalidReference))
    );
    let mut missing = meta.clone();
    missing.rails.push(StaticRail {
        edge: (ProvinceId(10), ProvinceId(999)),
        level: 1,
    });
    assert_eq!(
        prepare(&w, &defines(), n, &t, &missing),
        Err(AdapterError::InvalidReference)
    );
    let mut duplicate = meta.clone();
    duplicate.rails = vec![
        StaticRail {
            edge: (ProvinceId(10), ProvinceId(20)),
            level: 1
        };
        2
    ];
    assert_eq!(
        prepare(&w, &defines(), n, &t, &duplicate),
        Err(AdapterError::Duplicate)
    );
}
#[test]
fn overflow_and_rounded_zero_costs_reject_without_world_mutation() {
    let mut d = loaded();
    d.map.edges = vec![oh_data::map::Edge {
        a: 10,
        b: 20,
        kind: EdgeKind::Normal,
        distance_km: Fx::from_bits(i64::MAX),
    }];
    let w = World::from_loaded(&d).unwrap();
    let mut t = tuning();
    t.reference_speed_kmh = Fx::from_bits(1);
    assert_eq!(
        prepare(
            &w,
            &defines(),
            w.inputs().nations()[0].id(),
            &t,
            &capital(&w)
        ),
        Err(AdapterError::Overflow)
    );
    d.map.edges[0].distance_km = Fx::from_bits(1);
    let w = World::from_loaded(&d).unwrap();
    t.reference_speed_kmh = Fx::from_num(4);
    assert_eq!(
        prepare(
            &w,
            &defines(),
            w.inputs().nations()[0].id(),
            &t,
            &capital(&w)
        ),
        Err(AdapterError::InvalidValue)
    );
    let mut sim = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&d.pack.defines).unwrap(),
        w,
    )
    .unwrap();
    let before = sim.state_hash().unwrap();
    let state = sim.snapshot();
    let w = sim.world().unwrap();
    assert!(prepare(w, &defines(), w.inputs().nations()[0].id(), &t, &capital(w)).is_err());
    assert_eq!(sim.state_hash().unwrap(), before);
    assert_eq!(sim.snapshot(), state);
    assert!(sim.pending_commands().is_empty());
    sim.step().unwrap();
}
#[test]
fn excluded_edges_are_not_supply_rails() {
    for kind in [EdgeKind::Strait, EdgeKind::Impassable] {
        let mut d = loaded();
        d.map.edges = vec![oh_data::map::Edge {
            a: 10,
            b: 20,
            kind,
            distance_km: Fx::from_num(8),
        }];
        let w = World::from_loaded(&d).unwrap();
        let n = w.inputs().nations()[0].id();
        let mut meta = capital(&w);
        let net = prepare(&w, &Defines(BTreeMap::new()), n, &tuning(), &meta).unwrap();
        assert!(net.land.is_empty());
        meta.rails.push(StaticRail {
            edge: (ProvinceId(10), ProvinceId(20)),
            level: 1,
        });
        assert_eq!(
            prepare(&w, &defines(), n, &tuning(), &meta),
            Err(AdapterError::InvalidReference)
        );
    }
}

#[test]
fn network_metadata_fresh_process_record() {
    for variant in 0..8 {
        let mut d = loaded();
        d.nations[0].id = 0;
        if variant % 2 == 1 {
            d.map
                .provinces
                .iter_mut()
                .find(|p| p.id == 20)
                .unwrap()
                .terrain = "hills".into();
        }
        let w = World::from_loaded(&d).unwrap();
        let mut t = tuning();
        t.infrastructure_factors.insert(
            Fx::ONE,
            Fx::from_bits((1i64 << 32) + variant * (1i64 << 29)),
        );
        let mut meta = capital(&w);
        meta.rails.push(StaticRail {
            edge: (ProvinceId(10), ProvinceId(20)),
            level: if variant % 2 == 0 { 1 } else { 2 },
        });
        let network = prepare(&w, &defines(), NationId(0), &t, &meta).unwrap();
        println!(
            "NETWORK,{variant},{},{},{}",
            network.nation.0,
            network.capital.0,
            network.decay.to_bits()
        );
        for (&id, node) in &network.nodes {
            println!(
                "NODE,{variant},{},{:?},{}",
                id.0,
                node.controller.map(|n| n.0),
                node.land
            );
        }
        for (&(a, b), edge) in &network.land {
            println!(
                "EDGE,{variant},{},{},{},{},{:?}",
                a.0,
                b.0,
                edge.cost.to_bits(),
                edge.reverse_cost.to_bits(),
                edge.kind
            );
        }
        for (&(a, b), rail) in &network.rails {
            println!(
                "RAIL,{variant},{},{},{},{},{}",
                a.0,
                b.0,
                rail.cost.to_bits(),
                rail.reverse_cost.to_bits(),
                rail.capacity.to_bits()
            );
        }
        for (source, path) in network.feeders().unwrap() {
            println!("FEEDER,{variant},{},{:?}", source.0, path);
        }
    }
}
