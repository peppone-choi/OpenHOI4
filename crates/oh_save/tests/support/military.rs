//! Synthetic opt-in pack for actual authority codec/query contracts.
#[path = "../../../oh_data/tests/support/production.rs"]
mod production;
pub fn pack() -> std::path::PathBuf {
    let root = production::pack();
    std::fs::create_dir_all(root.join("common/military")).unwrap();
    let normal = include_str!("../../../oh_data/tests/fixtures/military_templates/normal.toml")
        .replace("manpower = 1000", "manpower = 6")
        .replace("manpower = 100", "manpower = 2")
        .replace("items = 100", "items = 6")
        .replace("items = 10", "items = 2");
    let source = format!(
        "version=1\nnormal_templates=\"\"\"{normal}\"\"\"\ntraining_days={{example=2}}\nbackground={{1={{committed=60,reserved=10}},2={{committed=60,reserved=10}}}}\narmies=[{{id=0,nation=1,general='synthetic_general',capacity=4,priority=0}},{{id=1,nation=2,general='synthetic_general',capacity=4,priority=0}}]\ndivisions=[]\n"
    );
    std::fs::write(root.join("common/military/synthetic.toml"), source).unwrap();
    let path = root.join("scenarios/m1/scenario.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path, format!("military = \"synthetic\"\n{old}")).unwrap();
    root
}
