//! Print a synthetic core hash for external OS comparisons. No game rules are run.
#[path = "../tests/common/mod.rs"]
mod common;

fn main() -> Result<(), oh_core::SerializationError> {
    println!(
        "{:016x}",
        oh_core::state_hash(&common::diagnostic_snapshot())?
    );
    Ok(())
}
