//! M0 headless `run` boundary: filesystem loading lives here, outside oh_sim.
use oh_sim::{Date, Simulation, TimeConfig, formula};
use std::{collections::BTreeMap, path::Path};

/// Shared M0 host startup path; relative to the repository working directory.
pub use oh_data::m0::M0_PACK_ROOT;
pub const USAGE: &str = "usage: oh_cli run --scenario <id> (--ticks <n> | --days <n>) --seed <u64> [--hash-out]\nM1: run --pack data/packs/testland --scenario m1 --days 365 --seed 1 --hash-out\nSave: run ... --save-out <file>; resume --load <file> --pack <root> (--ticks <n> | --days <n>) [--force] [--save-out <file>] [--hash-out]. M0 resume --pack uses its parent examples/m0 root.\nvalidate, ai-bench and repro belong to later work packages.\nM0 --scenario testland resolves data/packs/examples/m0/testland (manifest ID m0_testland).\nInputs: manifest.toml, defines.toml, scenarios/testland/scenario.toml under that pack.\nThis self-contained empty example is distinct from data/packs/testland (WP-03 skeleton / later WP-23 content).";

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

/// Explicit M1 pack path; M0 RunOptions/run remain byte-for-byte compatible.
pub fn parse_invocation(
    args: &[String],
) -> Result<(RunOptions, Option<std::path::PathBuf>), String> {
    let mut filtered = Vec::new();
    let mut pack = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--pack" {
            if pack.is_some() {
                return Err("duplicate --pack".into());
            }
            i += 1;
            let value = args
                .get(i)
                .filter(|s| !s.starts_with("--"))
                .ok_or("missing --pack")?;
            pack = Some(value.into());
        } else {
            filtered.push(args[i].clone());
        }
        i += 1;
    }
    Ok((RunOptions::parse(&filtered)?, pack))
}
pub fn load_national(
    root: &Path,
    id: &str,
) -> Result<(LoadedScenario, oh_sim::world::World), String> {
    let loaded = oh_data::national::load_scenario(root, id).map_err(|e| e.to_string())?;
    let world = oh_sim::world::World::from_loaded(&loaded)?;
    let time = TimeConfig::from_defines(&loaded.pack.defines).map_err(|e| e.to_string())?;
    let start_date = parse_date(&loaded.scenario.start_date)?;
    Ok((
        LoadedScenario {
            pack: loaded.pack,
            scenario_id: id.into(),
            start_date,
            time,
        },
        world,
    ))
}
pub fn run_national(root: &Path, options: &RunOptions) -> Result<Simulation, String> {
    let (loaded, world) = load_national(root, &options.scenario)?;
    let mut sim = Simulation::with_world(
        loaded.scenario_id,
        loaded.start_date,
        options.seed,
        loaded.time,
        world,
    )
    .map_err(|e| e.to_string())?;
    for _ in 0..options.ticks {
        sim.step().map_err(|e| e.to_string())?;
    }
    Ok(sim)
}

/// Minimal native checkpoint path. Existing RunOptions and M0 golden stay intact.
pub fn execute_invocation(args: &[String]) -> Result<(Simulation, bool), String> {
    let mut filtered = Vec::new();
    let mut save = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--save-out" {
            if save.is_some() {
                return Err("duplicate --save-out".into());
            }
            i += 1;
            save = Some(std::path::PathBuf::from(
                args.get(i)
                    .filter(|s| !s.starts_with("--"))
                    .ok_or("missing --save-out")?,
            ));
        } else {
            filtered.push(args[i].clone());
        }
        i += 1;
    }
    let (sim, hash_out, context) = if filtered.first().is_some_and(|s| s == "resume") {
        let mut values = BTreeMap::new();
        let mut force = false;
        let mut hash_out = false;
        let mut i = 1;
        while i < filtered.len() {
            let key = filtered[i].as_str();
            if key == "--force" || key == "--hash-out" {
                let flag = if key == "--force" {
                    &mut force
                } else {
                    &mut hash_out
                };
                if *flag {
                    return Err(format!("duplicate {key}"));
                }
                *flag = true;
                i += 1;
                continue;
            }
            if !["--load", "--pack", "--ticks", "--days"].contains(&key) {
                return Err(format!("unknown resume argument {key}"));
            }
            let value = filtered
                .get(i + 1)
                .filter(|s| !s.starts_with("--"))
                .ok_or_else(|| format!("missing {key}"))?;
            if values.insert(key, value.as_str()).is_some() {
                return Err(format!("duplicate {key}"));
            }
            i += 2;
        }
        let get = |key: &str| {
            values
                .get(key)
                .copied()
                .ok_or_else(|| format!("missing {key}"))
        };
        let count = match (values.get("--ticks"), values.get("--days")) {
            (Some(n), None) => n.parse::<u64>().map_err(|_| "invalid --ticks")?,
            (None, Some(n)) => formula::ticks_for_days(n.parse().map_err(|_| "invalid --days")?)
                .map_err(|e| e.to_string())?,
            _ => return Err("provide exactly one of --ticks or --days".into()),
        };
        let limits = oh_save::Limits::default();
        let bytes = oh_save::read_file(Path::new(get("--load")?), &limits)?;
        let header = oh_save::inspect_header(&bytes, &limits)?;
        let root = Path::new(get("--pack")?);
        let context = if header.definitions_hash.is_some() {
            oh_save::SaveContext::national(root, &header.scenario_id)?
        } else {
            oh_save::SaveContext::m0(root, &header.scenario_id)?
        };
        let loaded = oh_save::decode(&bytes, &context, force)?;
        for warning in loaded.warnings {
            eprintln!("oh_cli: {warning}");
        }
        let mut sim = loaded.simulation;
        advance(&mut sim, count)?;
        (sim, hash_out, Some(context))
    } else {
        let (options, pack) = parse_invocation(&filtered)?;
        if save.is_some() {
            let context = match &pack {
                Some(root) => oh_save::SaveContext::national(root, &options.scenario)?,
                None => oh_save::SaveContext::m0(Path::new(M0_PACK_ROOT), &options.scenario)?,
            };
            let mut sim = context.simulation(options.seed)?;
            advance(&mut sim, options.ticks)?;
            (sim, options.hash_out, Some(context))
        } else {
            let sim = match pack {
                Some(root) => run_national(&root, &options)?,
                None => run(Path::new(M0_PACK_ROOT), &options)?,
            };
            (sim, options.hash_out, None)
        }
    };
    if let Some(path) = save {
        let context = context.ok_or("missing save context")?;
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        let bytes = oh_save::encode(
            &sim,
            &context,
            i64::try_from(seconds).map_err(|_| "UTC timestamp overflow")?,
            vec![],
        )?;
        let outcome = oh_save::write_atomic(&path, &bytes, &context)?;
        if let Some(warning) = outcome.durability_warning {
            eprintln!("oh_cli: {warning}");
        }
    }
    Ok((sim, hash_out))
}
pub fn advance(sim: &mut Simulation, ticks: u64) -> Result<(), String> {
    let mut candidate = sim.clone();
    for _ in 0..ticks {
        if !candidate.step().map_err(|e| e.to_string())?.advanced {
            return Err("PausedCannotAdvance: no current-tick resume".into());
        }
    }
    *sim = candidate;
    Ok(())
}
