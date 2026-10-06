//! Regenerate structural schemas; does not update gameplay golden data.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("schema");
    std::fs::create_dir_all(&output)?;
    for (name, schema) in [
        ("manifest", oh_data::manifest_schema()),
        ("defines", oh_data::defines_schema()),
    ] {
        std::fs::write(
            output.join(format!("{name}.schema.json")),
            format!("{}\n", serde_json::to_string_pretty(&schema)?),
        )?;
    }
    Ok(())
}
