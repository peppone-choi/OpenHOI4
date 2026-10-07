//! Shared filesystem startup inputs for the M0 CLI and server hosts.
use crate::DataPack;
use serde::Deserialize;
use std::path::Path;
pub const M0_PACK_ROOT: &str = "data/packs/examples/m0";
#[derive(Debug, Clone)]
pub struct LoadedM0 {
    pub pack: DataPack,
    pub scenario_id: String,
    pub start_date: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyScenario {
    start_date: String,
}
pub fn load_m0_scenario(root: &Path, id: &str) -> Result<LoadedM0, String> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err("scenario ID must match [a-z0-9_]+".into());
    }
    let path = root.join(id);
    let mut pack = crate::pack_validation::resolve_packs(std::slice::from_ref(&path))
        .map_err(|e| e.to_string())?
        .remove(0)
        .data;
    pack.defines = crate::scenario_defines::load(&path, id, &pack.defines, false)
        .map_err(|e| e.to_string())?;
    let scenario_path = path.join("scenarios").join(id).join("scenario.toml");
    let source = std::fs::read_to_string(&scenario_path)
        .map_err(|e| format!("{}: {e}", scenario_path.display()))?;
    let scenario: EmptyScenario =
        toml::from_str(&source).map_err(|e| format!("{}: {e}", scenario_path.display()))?;
    crate::check_present_catalogs(&path).map_err(|e| e.to_string())?;
    Ok(LoadedM0 {
        pack,
        scenario_id: id.into(),
        start_date: scenario.start_date,
    })
}
