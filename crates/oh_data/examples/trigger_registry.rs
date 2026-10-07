//! Registry-derived syntax/capability reference, separate from gameplay data.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string_pretty(&oh_data::trigger::registry())?
    );
    Ok(())
}
