//! Reproducible generated production input and separate-process save acceptance.
#[path = "../../oh_data/tests/support/production.rs"]
mod fixture;
use oh_core::{NationId, Qty};
use oh_save::{SaveContext, decode, encode, repro};
use oh_sim::{Command, production::Action};
use std::path::Path;
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("capture") if (2..=3).contains(&args.len()) => {
            let ticks: u64 = args
                .get(2)
                .map(|s| s.parse())
                .transpose()
                .map_err(|_| "ticks")?
                .unwrap_or(240);
            let out = Path::new(&args[1]);
            if out.exists() {
                return Err("fresh output directory required".into());
            }
            std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
            let root = fixture::pack();
            let c = SaveContext::national(&root, "m1")?;
            let mut s = c.simulation(1)?;
            s.enqueue(
                0,
                NationId(1),
                1,
                Command::Production(Action::Create {
                    model: "test_model_1".into(),
                    requested_ic: Qty::ONE,
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
                Command::Production(Action::Switch {
                    line: 0,
                    model: "test_model_2".into(),
                }),
            )
            .map_err(|e| e.to_string())?;
            let bytes = encode(&s, &c, 0, vec![1])?;
            if bytes != encode(&s, &c, 0, vec![1])? {
                return Err("repeat bytes differ".into());
            }
            std::fs::write(out.join("saved.ohsave"), &bytes).map_err(|e| e.to_string())?;
            let split = repro::report(&s)?;
            let mut paused = s.clone();
            paused
                .enqueue(24, NationId(1), 3, Command::Pause(true))
                .map_err(|e| e.to_string())?;
            paused.step().map_err(|e| e.to_string())?;
            std::fs::write(out.join("paused.ohsave"), encode(&paused, &c, 0, vec![1])?)
                .map_err(|e| e.to_string())?;
            for _ in 0..ticks {
                s.step().map_err(|e| e.to_string())?;
            }
            let result = serde_json::json!({"pid":std::process::id(),"pack_path":root,"pack":c.pack(),"continued_ticks":ticks,"split":split,"continued":repro::report(&s)?,"paused":repro::report(&paused)?});
            std::fs::write(
                out.join("capture.json"),
                serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            println!("{result}");
        }
        Some("resume") if args.len() == 4 => {
            let c = SaveContext::national(Path::new(&args[1]), "m1")?;
            let bytes = std::fs::read(&args[2]).map_err(|e| e.to_string())?;
            let loaded = decode(&bytes, &c, false)?;
            let mut s = loaded.simulation;
            if bytes
                != encode(
                    &s,
                    &c,
                    loaded.header.saved_at_utc,
                    loaded.header.player_nations,
                )?
            {
                return Err("native repeat bytes differ".into());
            }
            let initial = repro::report(&s)?;
            let ticks: u64 = args[3].parse().map_err(|_| "ticks")?;
            for _ in 0..ticks {
                s.step().map_err(|e| e.to_string())?;
            }
            println!(
                "{}",
                serde_json::json!({"pid":std::process::id(),"initial":initial,"resumed":repro::report(&s)?})
            );
        }
        _ => return Err("capture OUT [TICKS] | resume PACK SAVE TICKS".into()),
    }
    Ok(())
}
