//! Actual additive v5 processes used by the current save evidence capture.
use oh_core::NationId;
use oh_save::{SaveContext, decode, encode, repro};
use oh_sim::{Command, economy::Action};
use std::path::Path;
fn copy(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(source).map_err(|e| e.to_string())? {
        let p = entry.map_err(|e| e.to_string())?.path();
        let d = destination.join(p.file_name().ok_or("path")?);
        if p.is_dir() {
            copy(&p, &d)?;
        } else {
            std::fs::copy(p, d).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("capture") if args.len() == 2 => {
            let out = Path::new(&args[1]);
            if out.exists() {
                return Err("fresh capture directory required".into());
            }
            let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
            let root = out.join("pack");
            copy(&source, &root)?;
            copy(
                &source.join("scenarios/m1"),
                &root.join("scenarios/wp14_native"),
            )?;
            std::fs::create_dir_all(root.join("common/economy")).map_err(|e| e.to_string())?;
            std::fs::write(
                root.join("common/economy/wp14_native.toml"),
                include_bytes!("../../oh_data/tests/fixtures/economy/valid.toml"),
            )
            .map_err(|e| e.to_string())?;
            let path = root.join("scenarios/wp14_native/scenario.toml");
            let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            // New fixture only; original M1 source bytes stay intact.
            let text = text.replace("start_date = \"2000-01-01\"", "start_date = \"2000-02-28\"");
            let prefix = "economy = \"wp14_native\"\nflag_keys = [\"economic_test\"]\neffect_programs = { boundary = { root = \"NTH\", effects = [{ add_political_capital = \"-3\" }] } }\n";
            std::fs::write(path, format!("{prefix}{text}")).map_err(|e| e.to_string())?;
            let c = SaveContext::national(&root, "wp14_native")?;
            let mut s = c.simulation(1000)?;
            s.enqueue(
                0,
                NationId(1),
                1,
                Command::Economy(Action::Construct {
                    project: 91,
                    state: 1,
                    building: "industry".into(),
                }),
            )
            .map_err(|e| e.to_string())?;
            for _ in 0..24 {
                s.step().map_err(|e| e.to_string())?;
            }
            s.enqueue(
                60,
                NationId(1),
                2,
                Command::Economy(Action::Cancel { project: 91 }),
            )
            .map_err(|e| e.to_string())?;
            s.enqueue(
                70,
                NationId(1),
                3,
                Command::Economy(Action::Allocate {
                    ratios: [
                        oh_core::Fx::from_num(0.5),
                        oh_core::Fx::from_num(0.5),
                        oh_core::Fx::ZERO,
                        oh_core::Fx::ZERO,
                    ],
                }),
            )
            .map_err(|e| e.to_string())?;
            let split = repro::report(&s)?;
            let a = encode(&s, &c, 0, vec![1])?;
            let b = encode(&s, &c, 0, vec![1])?;
            if a != b {
                return Err("repeat save bytes differ".into());
            }
            std::fs::write(out.join("saved.ohsave"), &a).map_err(|e| e.to_string())?;
            std::fs::write(out.join("repeat.ohsave"), &b).map_err(|e| e.to_string())?;
            let mut paused = s.clone();
            paused
                .enqueue(24, NationId(1), 4, Command::Pause(true))
                .map_err(|e| e.to_string())?;
            paused.step().map_err(|e| e.to_string())?;
            let paused_report = repro::report(&paused)?;
            std::fs::write(out.join("paused.ohsave"), encode(&paused, &c, 0, vec![1])?)
                .map_err(|e| e.to_string())?;
            let mut recorder = repro::Recorder::new(&s, &c)?;
            for _ in 0..48 {
                recorder.input(&mut s, repro::Input::Step {})?;
            }
            let continued = repro::report(&s)?;
            let journal = recorder.finish(&s, &c)?;
            std::fs::write(out.join("journal.zip"), journal).map_err(|e| e.to_string())?;
            let result = serde_json::json!({"pid":std::process::id(),"pack":c.pack(),"defines_hash":c.defines_hash(),"split":split,"paused":paused_report,"continued":continued});
            std::fs::write(
                out.join("capture.json"),
                serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            println!("{result}");
        }
        Some("resume") if args.len() == 4 => {
            let c = SaveContext::national(Path::new(&args[1]), "wp14_native")?;
            let bytes = std::fs::read(&args[2]).map_err(|e| e.to_string())?;
            let loaded = decode(&bytes, &c, false)?;
            let input_header = loaded.header;
            let mut s = loaded.simulation;
            let initial = repro::report(&s)?;
            let reencoded = encode(
                &s,
                &c,
                input_header.saved_at_utc,
                input_header.player_nations.clone(),
            )?;
            if bytes != reencoded {
                return Err("native reencoded bytes differ".into());
            }
            let ticks: u64 = args[3].parse().map_err(|_| "ticks")?;
            for _ in 0..ticks {
                s.step().map_err(|e| e.to_string())?;
            }
            println!(
                "{}",
                serde_json::json!({"pid":std::process::id(),"input_header":input_header,"initial":initial,"resumed":repro::report(&s)?})
            );
        }
        Some("replay") if args.len() == 3 => {
            let c = SaveContext::national(Path::new(&args[1]), "wp14_native")?;
            let bytes = std::fs::read(&args[2]).map_err(|e| e.to_string())?;
            let s = repro::replay(&bytes, &c)?;
            println!(
                "{}",
                serde_json::json!({"pid":std::process::id(),"result":repro::report(&s)?})
            );
        }
        _ => return Err("capture OUT | resume PACK SAVE TICKS | replay PACK JOURNAL".into()),
    }
    Ok(())
}
