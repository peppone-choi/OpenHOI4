use oh_data::military::{parse, schema};
#[path = "support/production.rs"]
mod production;

fn input(armies: &str, divisions: &str) -> String {
    let normal = include_str!("fixtures/military_templates/normal.toml");
    format!(
        "version=1\nnormal_templates='''{normal}'''\ntraining_days={{example=2}}\nbackground={{1={{committed=60,reserved=10}},2={{committed=60,reserved=10}}}}\narmies=[{armies}]\ndivisions=[{divisions}]\n"
    )
}
const ARMY: &str = "{id=0,nation=1,general='synthetic_general',capacity=4096,priority=0}";

#[test]
fn military_schema_snapshot_and_scenario_presence_are_current() {
    for (name, generated) in [
        ("military", schema()),
        ("scenario", oh_data::national::scenario_schema()),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("schema/{name}.schema.json"));
        let stored: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(stored, serde_json::to_value(generated).unwrap());
    }
}

#[test]
fn strict_military_structure_limits_and_raw_duplicates() {
    let good = input(ARMY, "");
    assert!(parse(&good).is_ok());
    for bad in [
        good.replace("version=1", "version=2"),
        good.replace("version=1", "version=1\nunknown=1"),
        good.replace("priority=0", "priority=0,unknown=1"),
        good.replace("committed=60", "committed=-1"),
        good.replace("reserved=10", "reserved=-1"),
        good.replace("priority=0", "priority=65536"),
        good.replace("capacity=4096", "capacity=-1"),
        good.replace("training_days={example=2}", "training_days={example=1.5}"),
        input(&format!("{ARMY},{ARMY}"), ""),
        input(
            ARMY,
            "{id=0,army=0,template='example',province=10,manpower=0,equipment={}}, {id=0,army=0,template='example',province=10,manpower=0,equipment={}}",
        ),
        " ".repeat(oh_data::military_templates::MAX_BYTES + 1),
    ] {
        assert!(
            parse(&bad).is_err(),
            "unexpected accepted malformed military input"
        );
    }
    let divisions = (0..4096)
        .map(|id| {
            format!("{{id={id},army=0,template='example',province=10,manpower=0,equipment={{}}}}")
        })
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        parse(&input(ARMY, &divisions)).unwrap().divisions.len(),
        4096
    );
    assert!(parse(&input(ARMY, &format!("{divisions},{{id=4096,army=0,template='example',province=10,manpower=0,equipment={{}}}}"))).is_err());
}

#[test]
fn military_pack_authority_and_reference_rejections() {
    let pack = production::pack();
    std::fs::create_dir_all(pack.join("common/military")).unwrap();
    let path = pack.join("common/military/synthetic.toml");
    let scenario = pack.join("scenarios/m1/scenario.toml");
    let old = std::fs::read_to_string(&scenario).unwrap();
    std::fs::write(&scenario, format!("military='synthetic'\n{old}")).unwrap();
    let good = input(ARMY, "");
    std::fs::write(&path, &good).unwrap();
    let loaded = oh_data::national::load_scenario(&pack, "m1").unwrap();
    assert_eq!(loaded.military.unwrap().armies.len(), 1);
    for bad in [
        good.replace("example=2", "example=0"),
        good.replace("example=2", "missing=2"),
        good.replace("nation=1", "nation=3"),
        good.replace("capacity=4096", "capacity=0"),
        good.replace("synthetic_general", "Invalid General"),
        good.replace(
            "2={committed=60,reserved=10}",
            "3={committed=60,reserved=10}",
        ),
        input(
            ARMY,
            "{id=0,army=9,template='example',province=10,manpower=0,equipment={}}",
        ),
        input(
            ARMY,
            "{id=0,army=0,template='example',province=65535,manpower=0,equipment={}}",
        ),
    ] {
        std::fs::write(&path, bad).unwrap();
        assert!(oh_data::national::load_scenario(&pack, "m1").is_err());
    }
    std::fs::write(&path, &good).unwrap();
    std::fs::write(
        &scenario,
        format!(
            "military='synthetic'\n{}",
            old.replace("production = \"synthetic\"\n", "")
        ),
    )
    .unwrap();
    assert!(oh_data::national::load_scenario(&pack, "m1").is_err());
    std::fs::remove_dir_all(pack.parent().unwrap()).unwrap();
}
