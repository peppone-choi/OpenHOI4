use oh_core::{Fx, NationId, ProvinceId, Qty};
use oh_sim::{supply::config_adapter, world::World};
const GOOD: &str = include_str!("../../oh_data/tests/fixtures/supply_network/capital.toml");
fn loaded() -> oh_data::national::LoadedNational {
    let mut l = oh_data::national::load_scenario(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap();
    for k in [
        "terrain_plains",
        "terrain_hills",
        "river_normal",
        "river_small",
        "river_large",
    ] {
        l.pack
            .defines
            .0
            .entry("movement".into())
            .or_default()
            .insert(
                k.into(),
                oh_data::DefineValue::Number(oh_data::Number::Fixed(Fx::ONE)),
            );
    }
    l
}
#[test]
fn explicit_config_uses_real_world_directed_cost_capacity_and_control() {
    let l = loaded();
    let w = World::from_loaded(&l).unwrap();
    let c = oh_data::supply_network::parse(GOOD).unwrap();
    let n = config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c).unwrap();
    assert_eq!(n.capital, ProvinceId(10));
    assert_eq!(
        n.rails[&(ProvinceId(10), ProvinceId(20))].capacity,
        Qty::from_num(20)
    );
    assert_eq!(n.nodes[&ProvinceId(20)].controller, Some(NationId(2)));
    assert_eq!(
        n.feeders().unwrap()[&oh_sim::supply::SourceId(0)],
        Some(vec![ProvinceId(10)])
    );
}
#[test]
fn exact_current_modified_infrastructure_and_asymmetric_costs_preserve_world_inputs() {
    let mut l = loaded();
    l.scenario.state_modifiers.insert(
        1,
        vec![oh_data::national::ModifierInput {
            source: "supply-config-test".into(),
            target_stat: "infrastructure".into(),
            operation: "add".into(),
            value: "0.125".into(),
            expires: Some(2),
        }],
    );
    l.pack.defines.0.get_mut("movement").unwrap().insert(
        "terrain_hills".into(),
        oh_data::DefineValue::Number(oh_data::Number::Fixed(Fx::from_num(2))),
    );
    let w = World::from_loaded(&l).unwrap();
    let before = oh_core::canonical_bytes(w.inputs()).unwrap();
    let c = oh_data::supply_network::parse(GOOD).unwrap();
    assert!(
        config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c)
            .unwrap_err()
            .contains("MissingContext")
    );
    let text = GOOD.replace(
        "{ level = \"1\", factor = \"1.5\" }",
        "{ level = \"1\", factor = \"1.5\" }, { level = \"1.125\", factor = \"0.5\" }",
    );
    let c = oh_data::supply_network::parse(&text).unwrap();
    let n = config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c).unwrap();
    let edge = &n.land[&(ProvinceId(10), ProvinceId(20))];
    let d = l
        .map
        .edges
        .iter()
        .find(|e| e.a == 10 && e.b == 20)
        .unwrap()
        .distance_km;
    // Same current state-level, distinct destination terrain. Keep exact established order.
    let base = d.checked_div(Fx::from_num(4)).unwrap();
    let river = Fx::ONE;
    assert_eq!(
        edge.cost,
        base.checked_mul(Fx::from_num(2))
            .unwrap()
            .checked_mul(Fx::ONE / Fx::from_num(2))
            .unwrap()
            .checked_mul(Fx::ONE)
            .unwrap()
            .checked_mul(river)
            .unwrap()
    );
    assert_eq!(
        edge.reverse_cost,
        base.checked_mul(Fx::ONE)
            .unwrap()
            .checked_mul(Fx::ONE / Fx::from_num(2))
            .unwrap()
            .checked_mul(Fx::ONE)
            .unwrap()
            .checked_mul(river)
            .unwrap()
    );
    assert_ne!(edge.cost, edge.reverse_cost);
    assert_eq!(before, oh_core::canonical_bytes(w.inputs()).unwrap());
}
#[test]
fn configured_hub_port_stays_unsupported_and_noncoastal_port_is_invalid() {
    let mut l = loaded();
    let w = World::from_loaded(&l).unwrap();
    for kind in ["hub", "port"] {
        let text = GOOD.replace(
            "kind = \"capital\", province = 10",
            &format!("kind = \"{kind}\", province = 20"),
        );
        let c = oh_data::supply_network::parse(&text).unwrap();
        c.validate_references(&l).unwrap();
        assert!(
            config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c)
                .unwrap_err()
                .contains("UnsupportedBuildingInstance")
        );
    }
    l.map
        .provinces
        .iter_mut()
        .find(|p| p.id == 20)
        .unwrap()
        .coastal = false;
    let w = World::from_loaded(&l).unwrap();
    let c = oh_data::supply_network::parse(&GOOD.replace(
        "kind = \"capital\", province = 10",
        "kind = \"port\", province = 20",
    ))
    .unwrap();
    assert!(
        config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c)
            .unwrap_err()
            .contains("InvalidReference")
    );
}
#[test]
fn nation_zero_current_controller_and_none_water_are_preserved() {
    let mut l = loaded();
    l.nations[0].id = 0;
    let w = World::from_loaded(&l).unwrap();
    let c = oh_data::supply_network::parse(&GOOD.replace("id = 1\nsources", "id = 0\nsources"))
        .unwrap();
    let n = config_adapter::prepare(&w, &l.pack.defines, NationId(0), &c).unwrap();
    assert_eq!(n.nodes[&ProvinceId(10)].controller, Some(NationId(0)));
    assert_eq!(n.nodes[&ProvinceId(20)].controller, Some(NationId(2)));
    assert_eq!(n.nodes[&ProvinceId(50)].controller, None);
    assert!(!n.nodes[&ProvinceId(50)].land);
}
#[test]
fn empty_static_metadata_is_not_absence_or_dummy_daily_supply() {
    let l = loaded();
    let w = World::from_loaded(&l).unwrap();
    let text = GOOD
        .replace(
            "[{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }]",
            "[]",
        )
        .replace("[{ a = 10, b = 20, level = 2 }]", "[]");
    let c = oh_data::supply_network::parse(&text).unwrap();
    let n = config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c).unwrap();
    assert!(n.sources.is_empty());
    assert!(n.rails.is_empty());
    assert!(n.feeders().unwrap().is_empty());
    assert!(!n.land.is_empty());
    assert!(
        config_adapter::prepare(&w, &l.pack.defines, NationId(2), &c)
            .unwrap_err()
            .contains("MissingContext")
    );
}
#[test]
fn actual_map_refs_current_missing_coefficients_and_overflow_are_not_defaulted() {
    let mut l = loaded();
    let c = oh_data::supply_network::parse(GOOD).unwrap();
    l.map.states[0].infrastructure = 3;
    let w = World::from_loaded(&l).unwrap();
    assert!(
        config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c)
            .unwrap_err()
            .contains("MissingContext")
    );
    let mut l = loaded();
    l.map
        .edges
        .iter_mut()
        .find(|e| e.a == 10 && e.b == 20)
        .unwrap()
        .kind = oh_data::map::EdgeKind::Impassable;
    let w = World::from_loaded(&l).unwrap();
    assert!(
        config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c)
            .unwrap_err()
            .contains("InvalidReference")
    );
    let mut l = loaded();
    l.map
        .edges
        .iter_mut()
        .find(|e| e.a == 10 && e.b == 20)
        .unwrap()
        .distance_km = Fx::from_bits(i64::MAX);
    let w = World::from_loaded(&l).unwrap();
    let c = oh_data::supply_network::parse(&GOOD.replace(
        "reference_speed_kmh = \"4\"",
        "reference_speed_kmh = \"0.00000000023283064365386962890625\"",
    ))
    .unwrap();
    let before = oh_core::canonical_bytes(w.inputs()).unwrap();
    assert!(
        config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c)
            .unwrap_err()
            .contains("Overflow")
    );
    assert_eq!(before, oh_core::canonical_bytes(w.inputs()).unwrap());
}
#[test]
fn empty_static_network_requires_exact_context_even_for_isolated_actual_states() {
    let mut l = loaded();
    l.map.edges.clear();
    l.map.states[0].infrastructure = 3;
    let w = World::from_loaded(&l).unwrap();
    let text = GOOD
        .replace(
            "[{ id = 0, kind = \"capital\", province = 10, capacity = \"20\" }]",
            "[]",
        )
        .replace("[{ a = 10, b = 20, level = 2 }]", "[]");
    let c = oh_data::supply_network::parse(&text).unwrap();
    assert!(
        config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c)
            .unwrap_err()
            .contains("MissingContext")
    );
    let complete = text.replace(
        "{ level = \"1\", factor = \"1.5\" }",
        "{ level = \"1\", factor = \"1.5\" }, { level = \"3\", factor = \"0.75\" }",
    );
    let c = oh_data::supply_network::parse(&complete).unwrap();
    let n = config_adapter::prepare(&w, &l.pack.defines, NationId(1), &c).unwrap();
    assert!(n.sources.is_empty());
    assert!(n.land.is_empty());
}
