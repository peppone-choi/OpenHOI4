//! Native host for the actual opt-in military authority, never a second simulator.
use oh_proto::{EconomyCommand, MilitaryCommand, ProductionCommand, TimeCommand};
use oh_save::SaveContext;
use oh_sim::{Command, Simulation};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{BufRead, Read, Write},
    path::{Path, PathBuf},
};

pub const USAGE: &str = "military run --pack <root> --scenario <id> --seed <canonical-u64> | military resume --pack <root> --load <save>; bounded stdin JSON-lines enqueue/enqueue_time/enqueue_economy/enqueue_production/step/query/save; actual opt-in military pack required";
/// Host request limits, not training rules or game time limits.
pub const INPUT_LINE_MAX_BYTES: u64 = 1_048_576;
pub const REQUEST_STEP_MAX: u64 = 1_000_000;

#[derive(Debug, Eq, PartialEq)]
pub struct Options {
    pub pack: PathBuf,
    pub scenario: Option<String>,
    pub seed: Option<u64>,
    pub load: Option<PathBuf>,
}
fn canonical_u64(text: &str) -> Result<u64, String> {
    let value = text.parse::<u64>().map_err(|_| "canonical u64 required")?;
    if value.to_string() != text {
        return Err("canonical u64 required".into());
    }
    Ok(value)
}
impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mode = args.get(1).map(String::as_str).ok_or(USAGE)?;
        if args.first().map(String::as_str) != Some("military")
            || !["run", "resume"].contains(&mode)
        {
            return Err(USAGE.into());
        }
        let allowed = if mode == "run" {
            vec!["--pack", "--scenario", "--seed"]
        } else {
            vec!["--pack", "--load"]
        };
        let mut values = BTreeMap::new();
        let mut i = 2;
        while i < args.len() {
            let key = args[i].as_str();
            if !allowed.contains(&key) {
                return Err(format!("unsupported military argument {key}; {USAGE}"));
            }
            let value = args
                .get(i + 1)
                .filter(|v| !v.is_empty() && !v.starts_with("--"))
                .ok_or_else(|| format!("missing {key}; {USAGE}"))?;
            if values.insert(key, value.as_str()).is_some() {
                return Err(format!("duplicate {key}"));
            }
            i += 2;
        }
        let required = |key: &str| {
            values
                .get(key)
                .copied()
                .ok_or_else(|| format!("missing {key}; {USAGE}"))
        };
        let scenario = if mode == "run" {
            let id = required("--scenario")?;
            if id.len() > 64 || !oh_data::valid_id(id) {
                return Err("invalid bounded scenario ID".into());
            }
            Some(id.to_owned())
        } else {
            None
        };
        Ok(Self {
            pack: required("--pack")?.into(),
            scenario,
            seed: if mode == "run" {
                Some(canonical_u64(required("--seed")?)?)
            } else {
                None
            },
            load: if mode == "resume" {
                Some(required("--load")?.into())
            } else {
                None
            },
        })
    }
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    Enqueue {
        nation: u16,
        sequence: String,
        tick: String,
        command: MilitaryCommand,
    },
    EnqueueTime {
        nation: u16,
        sequence: String,
        tick: String,
        command: TimeCommand,
    },
    EnqueueEconomy {
        nation: u16,
        sequence: String,
        tick: String,
        command: EconomyCommand,
    },
    EnqueueProduction {
        nation: u16,
        sequence: String,
        tick: String,
        command: ProductionCommand,
    },
    Step {
        count: u64,
    },
    Query,
    Save {
        path: PathBuf,
    },
}
fn context(root: &Path, scenario: &str) -> Result<SaveContext, String> {
    let fingerprint = crate::validate_for_run(root)?;
    let context = SaveContext::national(root, scenario)?;
    if context.pack().content_hash != fingerprint.content_hash {
        return Err("PackChangedDuringLoad: military context differs from validated pack".into());
    }
    crate::verify_validation_identity(root, fingerprint)?;
    Ok(context)
}
fn status(sim: &Simulation) -> Result<Value, String> {
    Ok(
        json!({"scope":"actual_military_simulation", "state":oh_proto::TimeState::from(&sim.snapshot()), "hash":format!("{:016x}",sim.state_hash().map_err(|e|e.to_string())?), "military":oh_proto::MilitaryView::from_sim(sim).ok_or("MissingMilitaryContext")?, "economy":oh_proto::EconomyView::from_sim(sim), "production":oh_proto::ProductionView::from_sim(sim)}),
    )
}
fn enqueue(
    sim: &mut Simulation,
    nation: u16,
    sequence: String,
    tick: String,
    command: Command,
) -> Result<Value, String> {
    let sequence = canonical_u64(&sequence)?;
    let tick = canonical_u64(&tick)?;
    let result = sim.enqueue(tick, oh_core::NationId(nation), sequence, command);
    Ok(
        json!({"op":"enqueue", "ok":result.is_ok(), "error":result.err().map(|e| format!("{e:?}")), "state":status(sim)?}),
    )
}
pub fn apply(
    sim: &mut Simulation,
    context: &SaveContext,
    input: Input,
    original_load: Option<&Path>,
) -> Result<Value, String> {
    match input {
        Input::Enqueue {
            nation,
            sequence,
            tick,
            command,
        } => enqueue(
            sim,
            nation,
            sequence,
            tick,
            Command::Military(command.into_action()?),
        ),
        Input::EnqueueTime {
            nation,
            sequence,
            tick,
            command,
        } => enqueue(
            sim,
            nation,
            sequence,
            tick,
            match command {
                TimeCommand::Pause { paused } => Command::Pause(paused),
                TimeCommand::SetSpeed { speed } => Command::SetSpeed(speed),
            },
        ),
        Input::EnqueueEconomy {
            nation,
            sequence,
            tick,
            command,
        } => enqueue(
            sim,
            nation,
            sequence,
            tick,
            Command::Economy(command.into_action()?),
        ),
        Input::EnqueueProduction {
            nation,
            sequence,
            tick,
            command,
        } => enqueue(
            sim,
            nation,
            sequence,
            tick,
            Command::Production(command.into_action()?),
        ),
        Input::Query => Ok(json!({"op":"query", "ok":true, "state":status(sim)?})),
        Input::Step { count } => {
            if count > REQUEST_STEP_MAX {
                return Err("military step request limit".into());
            }
            let mut commands = Vec::new();
            let mut advanced = 0u64;
            for _ in 0..count {
                match sim.step() {
                    Ok(step) => {
                        advanced += u64::from(step.advanced);
                        for outcome in step.commands {
                            commands.push(json!({"nation":outcome.nation.0,"sequence":outcome.sequence.to_string(),"ok":outcome.result.is_ok(),"error":outcome.result.err().map(|e|format!("{e:?}"))}));
                        }
                    }
                    Err(error) => {
                        return Ok(
                            json!({"op":"step","ok":false,"error":format!("{error:?}"),"advanced":advanced.to_string(),"commands":commands,"state":status(sim)?}),
                        );
                    }
                }
            }
            Ok(
                json!({"op":"step","ok":true,"advanced":advanced.to_string(),"commands":commands,"state":status(sim)?}),
            )
        }
        Input::Save { path } => {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let target = parent
                .canonicalize()
                .map_err(|e| e.to_string())?
                .join(path.file_name().ok_or("save filename required")?);
            if original_load.is_some_and(|p| p.canonicalize().ok() == Some(target)) {
                return Err("military save aliases input checkpoint".into());
            }
            // Fixed diagnostic timestamp makes equivalent native checkpoints byte comparable.
            let bytes = oh_save::encode(sim, context, 0, vec![])?;
            let header = oh_save::inspect_header(&bytes, &oh_save::Limits::default())?;
            let outcome = oh_save::write_atomic(&path, &bytes, context)?;
            Ok(
                json!({"op":"save","ok":true,"format_version":header.format_version,"durability_warning":outcome.durability_warning,"state":status(sim)?}),
            )
        }
    }
}
pub fn execute(args: &[String]) -> Result<(), String> {
    let options = Options::parse(args)?;
    let (context, mut sim) = if let Some(path) = &options.load {
        let bytes = oh_save::read_file(path, &oh_save::Limits::default())?;
        let header = oh_save::inspect_header(&bytes, &oh_save::Limits::default())?;
        if header.definitions_hash.is_none() {
            return Err("MissingMilitaryContext: national checkpoint required".into());
        }
        let context = context(&options.pack, &header.scenario_id)?;
        let sim = oh_save::decode(&bytes, &context, false)?.simulation;
        (context, sim)
    } else {
        let context = context(
            &options.pack,
            options.scenario.as_deref().ok_or("missing scenario")?,
        )?;
        let sim = context.simulation(options.seed.ok_or("missing seed")?)?;
        (context, sim)
    };
    status(&sim)?;
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    loop {
        let mut line = Vec::new();
        (&mut stdin)
            .take(INPUT_LINE_MAX_BYTES + 1)
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if line.is_empty() {
            break;
        }
        if line.len() as u64 > INPUT_LINE_MAX_BYTES {
            return Err("military input line limit".into());
        }
        let input = serde_json::from_slice(&line).map_err(|e| format!("military input: {e}"))?;
        let output = apply(&mut sim, &context, input, options.load.as_deref())?;
        serde_json::to_writer(&mut stdout, &output).map_err(|e| e.to_string())?;
        writeln!(stdout).map_err(|e| e.to_string())?;
        stdout.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}
