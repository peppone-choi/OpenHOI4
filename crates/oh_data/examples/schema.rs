//! Regenerate structural schemas; does not update gameplay golden data.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("schema");
    std::fs::create_dir_all(&output)?;
    for (name, schema) in [
        ("economy", oh_data::economy::schema()),
        ("manifest", oh_data::manifest_schema()),
        ("defines", oh_data::defines_schema()),
        ("states", oh_data::map::states_schema()),
        ("regions", oh_data::map::regions_schema()),
        ("nation", oh_data::national::nation_schema()),
        ("scenario", oh_data::national::scenario_schema()),
        ("visuals", oh_data::national::visuals_schema()),
        ("province", oh_data::map::province_schema()),
    ] {
        std::fs::write(
            output.join(format!("{name}.schema.json")),
            format!("{}\n", serde_json::to_string_pretty(&schema)?),
        )?;
    }
    Ok(())
}
