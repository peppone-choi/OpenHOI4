//! Temporary authored acceptance pack; shipped M1 sources stay untouched.
use std::path::PathBuf;
#[path = "../../../oh_data/tests/support/production.rs"]
mod production;
pub const MILITARY: &str = include_str!("../fixtures/military/training.toml");
pub fn pack() -> PathBuf {
    let root = production::pack();
    std::fs::create_dir_all(root.join("common/military")).unwrap();
    std::fs::write(root.join("common/military/synthetic.toml"), MILITARY).unwrap();
    let scenario = root.join("scenarios/m1/scenario.toml");
    let text = std::fs::read_to_string(&scenario).unwrap();
    std::fs::write(scenario, format!("military = \"synthetic\"\n{text}")).unwrap();
    let production = root.join("common/production/synthetic.toml");
    let text = std::fs::read_to_string(&production).unwrap();
    std::fs::write(
        production,
        text.replacen(
            "stock = { test_model_1 = 0",
            "stock = { test_model_1 = 4",
            1,
        ),
    )
    .unwrap();
    root
}
