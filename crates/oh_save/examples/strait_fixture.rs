//! Native REQUEST-0007 evidence producer with explicit trusted synthetic context.
use oh_core::{DivisionId, Fx, ProvinceId};
use oh_save::{SaveContext, decode, encode};
use oh_sim::{
    Command, Simulation,
    movement::{CrossingKind, Movement, StraitContext, StraitFactors, StraitInput, UnitInput},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn context(root: &Path) -> Result<SaveContext, String> {
    SaveContext::national(root, "m1")
}
fn initial(c: &SaveContext) -> Result<Simulation, String> {
    let base = c.simulation(7)?;
    let w = base.world().ok_or("world")?.clone();
    let n = w.inputs().nations()[0].id();
    let speed = w
        .defs()
        .map()
        .edge(10, 40)
        .ok_or("actual strait edge")?
        .distance_km;
    let unit = UnitInput {
        id: DivisionId(900),
        nation: n,
        province: ProvinceId(10),
        speed,
        allowed: BTreeSet::from([ProvinceId(10), ProvinceId(40)]),
        corrections: BTreeMap::new(),
    };
    let value = StraitContext {
        kind: CrossingKind::Strait,
        factors: StraitFactors {
            terrain: Fx::ONE,
            infrastructure: Fx::ONE,
            supply: Fx::ONE,
            strait: Fx::from_num(10),
        },
    };
    let m = Movement::with_straits(
        &w,
        vec![unit],
        vec![StraitInput {
            unit: DivisionId(900),
            corrections: BTreeMap::from([
                ((ProvinceId(10), ProvinceId(40)), value),
                ((ProvinceId(40), ProvinceId(10)), value),
            ]),
        }],
    )
    .map_err(|e| e.to_string())?;
    let mut s = Simulation::with_movement(
        "m1".into(),
        base.snapshot().date(),
        7,
        base.config().clone(),
        w,
        m,
    )
    .map_err(|e| e.to_string())?;
    for (tick, sequence, command) in [
        (
            0,
            1,
            Command::Move {
                unit: DivisionId(900),
                destination: ProvinceId(40),
            },
        ),
        (
            9,
            2,
            Command::Stop {
                unit: DivisionId(900),
            },
        ),
        (9, 3, Command::Pause(true)),
        (9, 4, Command::Pause(false)),
        (
            20,
            5,
            Command::Move {
                unit: DivisionId(900),
                destination: ProvinceId(10),
            },
        ),
    ] {
        s.enqueue(tick, n, sequence, command)
            .map_err(|e| e.to_string())?;
    }
    Ok(s)
}
fn report(s: &Simulation) -> Result<serde_json::Value, String> {
    Ok(
        serde_json::json!({"hash":format!("{:016x}",s.state_hash().map_err(|e|e.to_string())?),"dto":s.export_save_v3()?,"canonical_hex":oh_core::canonical_bytes(s).map_err(|e|e.to_string())?.iter().map(|b|format!("{b:02x}")).collect::<String>()}),
    )
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("capture") if args.len() == 2 => {
            let out = Path::new(&args[1]);
            std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
            let c = context(&root)?;
            let mut s = initial(&c)?;
            while s.snapshot().tick() < 9 {
                s.step().map_err(|e| e.to_string())?;
            }
            let split = report(&s)?;
            std::fs::write(out.join("strait-v3.ohsave"), encode(&s, &c, 0, vec![])?)
                .map_err(|e| e.to_string())?;
            s.step().map_err(|e| e.to_string())?;
            let arrival = report(&s)?;
            while s.snapshot().tick() < 32 {
                s.step().map_err(|e| e.to_string())?;
            }
            let result =
                serde_json::json!({"split":split,"arrival":arrival,"continuous":report(&s)?});
            std::fs::write(
                out.join("capture.json"),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .map_err(|e| e.to_string())?;
            println!("{}", result);
        }
        Some("resume") if args.len() == 3 => {
            let c = context(Path::new(&args[1]))?;
            let bytes = std::fs::read(&args[2]).map_err(|e| e.to_string())?;
            let mut s = decode(&bytes, &c, false)?.simulation;
            let split = report(&s)?;
            s.step().map_err(|e| e.to_string())?;
            let arrival = report(&s)?;
            while s.snapshot().tick() < 32 {
                s.step().map_err(|e| e.to_string())?;
            }
            println!(
                "{}",
                serde_json::json!({"split":split,"arrival":arrival,"continuous":report(&s)?})
            );
        }
        _ => return Err("strait_fixture capture <out> | resume <pack> <save>".into()),
    }
    Ok(())
}
