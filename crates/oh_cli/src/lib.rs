//! M0 headless `run` boundary: filesystem loading lives here, outside oh_sim.
use oh_sim::{Date, Simulation, TimeConfig, formula};
use std::{collections::BTreeMap, path::Path};

/// Shared M0 host startup path; relative to the repository working directory.
pub use oh_data::m0::M0_PACK_ROOT;
pub const USAGE: &str = "usage: oh_cli run --scenario <id> (--ticks <n> | --days <n>) --seed <u64> [--hash-out]\nM0 supports run only; validate, ai-bench and repro belong to later work packages.\nM0 --scenario testland resolves data/packs/examples/m0/testland (manifest ID m0_testland).\nInputs: manifest.toml, defines.toml, scenarios/testland/scenario.toml under that pack.\nThis self-contained empty example is distinct from data/packs/testland (WP-03 skeleton / later WP-23 content).";

#[derive(Debug, Eq, PartialEq)]
pub struct RunOptions {
    pub scenario: String,
    pub ticks: u64,
    pub seed: u64,
    pub hash_out: bool,
}
impl RunOptions {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        if args.first().map(String::as_str) != Some("run") {
            return Err(USAGE.into());
        }
        let mut values = BTreeMap::new();
        let mut hash_out = false;
        let mut index = 1;
        while index < args.len() {
            let key = args[index].as_str();
            if key == "--hash-out" {
                if hash_out {
                    return Err("duplicate --hash-out".into());
                }
                hash_out = true;
                index += 1;
                continue;
            }
            if !["--scenario", "--ticks", "--days", "--seed"].contains(&key) {
                return Err(format!("unknown argument {key}\n{USAGE}"));
            }
            let value = args
                .get(index + 1)
                .filter(|v| !v.starts_with("--"))
                .ok_or_else(|| format!("missing value for {key}"))?;
            if values.insert(key, value.as_str()).is_some() {
                return Err(format!("duplicate {key}"));
            }
            index += 2;
        }
        let required = |key: &str| {
            values
                .get(key)
                .copied()
                .ok_or_else(|| format!("missing {key}\n{USAGE}"))
        };
        let number = |key: &str| {
            required(key)?
                .parse::<u64>()
                .map_err(|_| format!("{key} must be an unsigned 64-bit integer"))
        };
        let scenario = required("--scenario")?;
        if scenario.is_empty()
            || !scenario
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err("scenario ID must match [a-z0-9_]+".into());
        }
        let ticks = match (
            values.contains_key("--ticks"),
            values.contains_key("--days"),
        ) {
            (true, false) => number("--ticks")?,
            (false, true) => {
                formula::ticks_for_days(number("--days")?).map_err(|err| err.to_string())?
            }
            _ => return Err("provide exactly one of --ticks or --days".into()),
        };
        Ok(Self {
            scenario: scenario.into(),
            ticks,
            seed: number("--seed")?,
            hash_out,
        })
    }
}

fn parse_date(value: &str) -> Result<Date, String> {
    let fields: Vec<_> = value.split('-').collect();
    if fields.len() != 3
        || fields
            .iter()
            .any(|v| v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("start_date must be a Gregorian YYYY-MM-DD string".into());
    }
    Date::new(
        fields[0].parse().map_err(|_| "invalid start year")?,
        fields[1].parse().map_err(|_| "invalid start month")?,
        fields[2].parse().map_err(|_| "invalid start day")?,
    )
    .map_err(|err| err.to_string())
}

/// Host-loaded startup inputs before constructing a simulation session.
/// Pack metadata remains available for the server handshake. No file loading
/// happens inside Simulation. Hosts may reuse this boundary with zero ticks.
#[derive(Debug, Clone)]
pub struct LoadedScenario {
    pub pack: oh_data::DataPack,
    pub scenario_id: String,
    pub start_date: Date,
    pub time: TimeConfig,
}
impl LoadedScenario {
    pub fn simulation(&self, seed: u64) -> Result<Simulation, oh_sim::Error> {
        Simulation::new(
            self.scenario_id.clone(),
            self.start_date,
            seed,
            self.time.clone(),
        )
    }
}

/// pack_root contains scenario-directory names, not necessarily manifest IDs.
/// M0 resolves one empty scenario in a self-contained pack (no merging).
/// WP-05 can reuse these startup inputs and oh_sim's public session API.
pub fn load_scenario(pack_root: &Path, scenario_id: &str) -> Result<LoadedScenario, String> {
    let loaded = oh_data::m0::load_m0_scenario(pack_root, scenario_id)?;
    let config = TimeConfig::from_defines(&loaded.pack.defines).map_err(|err| {
        format!(
            "{}: {err}",
            pack_root.join(scenario_id).join("defines.toml").display()
        )
    })?;
    let path = pack_root
        .join(scenario_id)
        .join("scenarios")
        .join(scenario_id)
        .join("scenario.toml");
    let date =
        parse_date(&loaded.start_date).map_err(|err| format!("{}: {err}", path.display()))?;
    Ok(LoadedScenario {
        pack: loaded.pack,
        scenario_id: loaded.scenario_id,
        start_date: date,
        time: config,
    })
}

pub fn run(pack_root: &Path, options: &RunOptions) -> Result<Simulation, String> {
    let mut sim = load_scenario(pack_root, &options.scenario)?
        .simulation(options.seed)
        .map_err(|err| err.to_string())?;
    for _ in 0..options.ticks {
        sim.step().map_err(|err| err.to_string())?;
    }
    Ok(sim)
}
