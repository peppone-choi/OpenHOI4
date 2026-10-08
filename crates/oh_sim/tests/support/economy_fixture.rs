pub fn loaded() -> oh_data::national::LoadedNational {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut l = oh_data::national::load_scenario(&root, "m1").unwrap();
    l.scenario.economy = Some("synthetic".into());
    l.economy = Some(toml_definition());
    l
}
pub fn toml_definition() -> oh_data::economy::Definition {
    use oh_data::economy::*;
    let building = Building {
        name_key: "industry-name".into(),
        costs: vec!["3".into(), "20".into(), "40".into()],
        slots: vec![1, 1, 1],
        daily_cap: "6".into(),
        ic_per_level: "10".into(),
        infrastructure_factors: vec![
            InfrastructureFactor {
                infrastructure: "0".into(),
                factor: "1".into(),
            },
            InfrastructureFactor {
                infrastructure: "1".into(),
                factor: "1".into(),
            },
            InfrastructureFactor {
                infrastructure: "1.125".into(),
                factor: "1".into(),
            },
        ],
        infrastructure_levels: None,
    };
    let law = Law {
        name_key: "law-name".into(),
        category: "economy".into(),
        step: 0,
        cost: "3".into(),
        condition: oh_data::trigger::Condition::DateGte("1900-01-01".into()),
        ic_multiplier: "1".into(),
        conscription_ratio: "0.125".into(),
        consumer_base: "0".into(),
        instability_slope: "0".into(),
    };
    let mut second = law.clone();
    second.step = 1;
    second.consumer_base = "0.5".into();
    let nation = NationInput {
        laws: std::collections::BTreeMap::from([("economy".into(), "civil".into())]),
        political_capital: "0".into(),
        political_capital_cap: "100".into(),
        political_capital_daily: "3".into(),
        stability: "0.5".into(),
        mobilization: "0.5".into(),
        allocation: std::array::from_fn(|_| "0.25".into()),
        ic_multiplier: "1".into(),
        committed: 60,
        reserved: 10,
    };
    Definition {
        buildings: std::collections::BTreeMap::from([("industry".into(), building)]),
        laws: std::collections::BTreeMap::from([("civil".into(), law), ("war".into(), second)]),
        economy_category: "economy".into(),
        conscription_category: "economy".into(),
        nations: std::collections::BTreeMap::from([(1, nation.clone()), (2, nation)]),
        state_slots: std::collections::BTreeMap::from([(1, 4), (2, 4)]),
    }
}
