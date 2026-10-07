//! Actual headless Host: stdin inputs drive authority, stdout reports actual results.
use oh_save::{
    SaveContext,
    repro::{self, Bundle, Input, Recorder},
    repro_zip::{self, Policy},
};
use std::{
    collections::BTreeMap,
    io::{BufRead, Read, Write},
    path::{Path, PathBuf},
};

pub const USAGE: &str = "repro record --out <zip> [--pack <national-root> --scenario <id> --seed <n> | --load <save> --pack <root>]; stdin JSON enqueue/pump/step lines, EOF ends recording. repro run <zip> [--pack <root>]";
fn context(root: Option<&Path>, scenario: &str, national: bool) -> Result<SaveContext, String> {
    if !oh_data::valid_id(scenario) {
        return Err("Repro: invalid scenario ID".into());
    }
    let root = root.unwrap_or_else(|| {
        Path::new(if national {
            "data/packs/testland"
        } else {
            crate::M0_PACK_ROOT
        })
    });
    let actual = if national {
        root.to_path_buf()
    } else {
        root.join(scenario)
    };
    // Active is fixed by the Host. No caller-selected historic policy or force.
    let fingerprint = crate::validate_for_run(&actual)?;
    let c = if national {
        SaveContext::national(root, scenario)?
    } else {
        SaveContext::m0(root, scenario)?
    };
    if c.pack().content_hash != fingerprint.content_hash {
        return Err("Repro: pack changed during load".into());
    }
    crate::verify_validation_identity(&actual, fingerprint)?;
    Ok(c)
}
fn get_path(values: &BTreeMap<&str, &str>, key: &str) -> Option<PathBuf> {
    values.get(key).map(PathBuf::from)
}
pub fn execute(args: &[String]) -> Result<(), String> {
    let mode = args.get(1).ok_or(USAGE)?;
    let mut values = BTreeMap::new();
    let mut i = if mode == "run" { 3 } else { 2 };
    while i < args.len() {
        let key = args[i].as_str();
        let allowed = if mode == "run" {
            vec!["--pack"]
        } else {
            vec!["--out", "--pack", "--scenario", "--seed", "--load"]
        };
        if !allowed.contains(&key) {
            return Err(format!("unknown repro argument {key}"));
        }
        let value = args
            .get(i + 1)
            .filter(|s| !s.starts_with("--"))
            .ok_or("missing repro argument")?;
        if values.insert(key, value.as_str()).is_some() {
            return Err("duplicate repro argument".into());
        }
        i += 2;
    }
    let pack = get_path(&values, "--pack");
    if mode == "run" {
        let path = Path::new(args.get(2).filter(|s| !s.starts_with("--")).ok_or(USAGE)?);
        let bytes = repro_zip::read(path, Policy::default().archive_max_bytes)?;
        let b: Bundle = repro::inspect(&bytes)?;
        let c = context(pack.as_deref(), &b.scenario, b.national)?;
        let sim = repro::replay(&bytes, &c)?;
        println!(
            "{}",
            serde_json::to_string(&repro::report(&sim)?).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if mode != "record" {
        return Err(USAGE.into());
    }
    let out = get_path(&values, "--out").ok_or("missing --out")?;
    let (c, mut sim) = if let Some(load) = get_path(&values, "--load") {
        if values.contains_key("--seed") || values.contains_key("--scenario") {
            return Err("--load excludes --seed/--scenario".into());
        }
        let bytes = repro_zip::read(&load, oh_save::Limits::default().file_max_bytes)?;
        let header = oh_save::inspect_header(&bytes, &oh_save::Limits::default())?;
        let c = context(
            pack.as_deref(),
            &header.scenario_id,
            header.definitions_hash.is_some(),
        )?;
        let sim = oh_save::decode(&bytes, &c, false)?.simulation;
        (c, sim)
    } else {
        let scenario = values.get("--scenario").copied().unwrap_or("m1");
        let seed = values
            .get("--seed")
            .copied()
            .unwrap_or("1")
            .parse::<u64>()
            .map_err(|_| "invalid seed")?;
        let c = context(pack.as_deref(), scenario, true)?;
        let sim = c.simulation(seed)?;
        (c, sim)
    };
    // Reject aliases which would replace an input save or any file within the pack.
    let parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let target = parent
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(out.file_name().ok_or("missing output filename")?);
    if target.starts_with(c.root())
        || get_path(&values, "--load")
            .is_some_and(|p| p.canonicalize().ok() == Some(target.clone()))
    {
        return Err("Repro: output aliases input save/pack".into());
    }
    let mut recorder = Recorder::new(&sim, &c)?;
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    loop {
        let mut line = Vec::new();
        (&mut stdin)
            .take(Policy::default().input_line_max_bytes + 1)
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if line.is_empty() {
            break;
        }
        if line.len() as u64 > Policy::default().input_line_max_bytes {
            return Err("Repro: input line limit".into());
        }
        let input: Input =
            serde_json::from_slice(&line).map_err(|e| format!("Repro input: {e}"))?;
        let event = recorder.input(&mut sim, input)?;
        serde_json::to_writer(&mut stdout, &event).map_err(|e| e.to_string())?;
        writeln!(stdout).map_err(|e| e.to_string())?;
        stdout.flush().map_err(|e| e.to_string())?;
    }
    let bytes = recorder.finish(&sim, &c)?;
    repro_zip::write(&out, &bytes, &c)?;
    serde_json::to_writer(&mut stdout, &repro::report(&sim)?).map_err(|e| e.to_string())?;
    writeln!(stdout).map_err(|e| e.to_string())?;
    Ok(())
}
