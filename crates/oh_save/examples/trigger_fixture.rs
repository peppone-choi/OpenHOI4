//! Native full-state probe. All synthetic context is explicit and test-only.
use oh_core::{DivisionId, Fx, ProvinceId};
use oh_save::{SaveContext, decode, encode};
use oh_sim::{
    Command, Simulation,
    movement::{Movement, StraitInput, UnitInput},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn copy(source: &Path, target: &Path) -> Result<(), String> {
    std::fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for e in std::fs::read_dir(source).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let path = e.path();
        let out = target.join(e.file_name());
        if e.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy(&path, &out)?;
        } else {
            std::fs::copy(path, out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
fn report(s: &Simulation) -> Result<serde_json::Value, String> {
    Ok(
        serde_json::json!({"dto":s.export_save_v4()?,"canonical_hex":oh_core::canonical_bytes(s).map_err(|e|e.to_string())?.iter().map(|b|format!("{b:02x}")).collect::<String>(),"hash":format!("{:016x}",s.state_hash().map_err(|e|e.to_string())?),"ended":s.is_ended()}),
    )
}
fn put(
    out: &Path,
    name: &str,
    s: &Simulation,
    c: &SaveContext,
) -> Result<serde_json::Value, String> {
    let bytes = encode(s, c, 0, vec![])?;
    std::fs::write(out.join(format!("{name}.ohsave")), &bytes).map_err(|e| e.to_string())?;
    let restored = decode(&bytes, c, false)?;
    if encode(&restored.simulation, c, 0, vec![])? != bytes {
        return Err("reencode differs".into());
    }
    let mut value = report(s)?;
    value["pack"] = serde_json::to_value(c.pack()).map_err(|e| e.to_string())?;
    value["effective_defines_hash"] = serde_json::json!(c.defines_hash());
    value["format"] = serde_json::json!(restored.header.format_version);
    Ok(value)
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("capture") if args.len() == 2 => {
            let out = Path::new(&args[1]);
            std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
            let root = out.join("pack");
            copy(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
                &root,
            )?;
            std::fs::write(
                root.join("scenarios/m1/scenario.toml"),
                include_str!("../tests/fixtures/trigger/scenario.toml"),
            )
            .map_err(|e| e.to_string())?;
            let c = SaveContext::national(&root, "m1")?;
            let base = c.simulation(7)?;
            let world = base.world().ok_or("world")?.clone();
            let n = world.inputs().nations()[0].id();
            let unit = UnitInput {
                id: DivisionId(900),
                nation: n,
                province: ProvinceId(10),
                speed: Fx::ONE,
                allowed: BTreeSet::from([ProvinceId(10)]),
                corrections: BTreeMap::new(),
            };
            let movement = Movement::with_straits(
                &world,
                vec![unit],
                vec![StraitInput {
                    unit: DivisionId(900),
                    corrections: BTreeMap::new(),
                }],
            )
            .map_err(|e| e.to_string())?;
            let mut s = Simulation::with_movement(
                "m1".into(),
                base.snapshot().date(),
                7,
                base.config().clone(),
                world,
                movement,
            )
            .map_err(|e| e.to_string())?;
            s.enqueue(
                0,
                n,
                1,
                Command::Effects {
                    program: "prepare".into(),
                },
            )
            .map_err(|e| e.to_string())?;
            s.enqueue(
                47,
                n,
                2,
                Command::Effects {
                    program: "finish".into(),
                },
            )
            .map_err(|e| e.to_string())?;
            for (sequence, command) in [
                (
                    3,
                    Command::Move {
                        unit: DivisionId(900),
                        destination: ProvinceId(10),
                    },
                ),
                (
                    4,
                    Command::Stop {
                        unit: DivisionId(900),
                    },
                ),
                (
                    5,
                    Command::Effects {
                        program: "prepare".into(),
                    },
                ),
                (6, Command::Pause(true)),
                (7, Command::SetSpeed(5)),
            ] {
                s.enqueue(99, n, sequence, command)
                    .map_err(|e| e.to_string())?;
            }
            let mut cases = BTreeMap::new();
            cases.insert("active", put(out, "active", &s, &c)?);
            while s.snapshot().tick() < 47 {
                s.step().map_err(|e| e.to_string())?;
            }
            cases.insert("split", put(out, "split", &s, &c)?);
            s.step().map_err(|e| e.to_string())?;
            if !s.is_ended() || s.pending_commands().len() != 5 {
                return Err("checkpoint/future queue".into());
            }
            cases.insert("ended", put(out, "ended", &s, &c)?);
            let before = oh_core::canonical_bytes(&s).map_err(|e| e.to_string())?;
            if s.step().is_ok()
                || s.enqueue(48, n, 99, Command::Pause(true)).is_ok()
                || before != oh_core::canonical_bytes(&s).map_err(|e| e.to_string())?
            {
                return Err("ended refusal mutated".into());
            }
            for (name, prefix) in [
                ("empty", "flag_keys = []\neffect_programs = {}\n"),
                ("initial", "end_conditions = { date_gte = '2000-01-01' }\n"),
                (
                    "paused_condition",
                    "end_root = 'NTH'\nend_conditions = { has_flag = 'x' }\nflag_keys = ['x']\n[effect_programs.set]\nroot = 'NTH'\neffects = [{set_flag = 'x'}]\n",
                ),
            ] {
                let root = out.join(format!("pack-{name}"));
                copy(
                    &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
                    &root,
                )?;
                let path = root.join("scenarios/m1/scenario.toml");
                let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                // Program tables must follow existing top-level fields and ownership tables.
                let source = if name == "paused_condition" {
                    let (fields, tables) = prefix.split_once("[effect_programs").ok_or("prefix")?;
                    format!("{fields}{source}\n[effect_programs{tables}")
                } else {
                    format!("{prefix}{source}")
                };
                std::fs::write(&path, source).map_err(|e| e.to_string())?;
                let c = SaveContext::national(&root, "m1")?;
                let mut s = c.simulation(7)?;
                if name == "paused_condition" {
                    s.enqueue(0, n, 1, Command::Pause(true))
                        .map_err(|e| e.to_string())?;
                    s.enqueue(
                        0,
                        n,
                        2,
                        Command::Effects {
                            program: "set".into(),
                        },
                    )
                    .map_err(|e| e.to_string())?;
                    s.step().map_err(|e| e.to_string())?;
                    if s.is_ended() {
                        return Err("paused condition re-evaluated".into());
                    }
                }
                cases.insert(name, put(out, name, &s, &c)?);
            }
            println!(
                "{}",
                serde_json::json!({"pid":std::process::id(),"cases":cases})
            );
        }
        Some("resume") if args.len() == 5 => {
            let c = SaveContext::national(Path::new(&args[1]), "m1")?;
            let bytes = std::fs::read(&args[2]).map_err(|e| e.to_string())?;
            let mut loaded = decode(&bytes, &c, false)?;
            let split = put(Path::new(&args[4]), "reencoded", &loaded.simulation, &c)?;
            let requested: u64 = args[3].parse().map_err(|_| "ticks")?;
            for _ in 0..requested {
                if loaded.simulation.is_ended() {
                    break;
                }
                loaded.simulation.step().map_err(|e| e.to_string())?;
            }
            let result = put(Path::new(&args[4]), "resumed", &loaded.simulation, &c)?;
            println!(
                "{}",
                serde_json::json!({"pid":std::process::id(),"split":split,"result":result})
            );
        }
        _ => return Err("capture <out> | resume <pack> <save> <ticks> <out>".into()),
    }
    Ok(())
}
