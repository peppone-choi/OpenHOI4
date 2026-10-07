//! Native WP-17 evidence producer. Synthetic host inputs, not UI/game content.
use oh_core::{DivisionId, Fx, ProvinceId};
use oh_save::{SaveContext, decode, encode};
use oh_sim::{
    Command, Simulation,
    movement::{Factors, Movement, UnitInput},
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
    let allowed = w
        .defs()
        .map()
        .provinces
        .iter()
        .filter(|p| p.kind == oh_data::map::ProvinceKind::Land)
        .map(|p| ProvinceId(p.id))
        .collect::<BTreeSet<_>>();
    let mut corrections = BTreeMap::new();
    for e in &w.defs().map().edges {
        for (a, b) in [(e.a, e.b), (e.b, e.a)] {
            if allowed.contains(&ProvinceId(a))
                && allowed.contains(&ProvinceId(b))
                && matches!(
                    e.kind,
                    oh_data::map::EdgeKind::Normal
                        | oh_data::map::EdgeKind::RiverSmall
                        | oh_data::map::EdgeKind::RiverLarge
                )
            {
                corrections.insert(
                    (ProvinceId(a), ProvinceId(b)),
                    Factors {
                        terrain: Fx::from_num(1.5),
                        infrastructure: Fx::ONE,
                        supply: Fx::ONE,
                        river: Fx::ONE,
                    },
                );
            }
        }
    }
    let movement = Movement::new(
        &w,
        vec![UnitInput {
            id: DivisionId(900),
            nation: n,
            province: ProvinceId(10),
            speed: Fx::from_num(2),
            allowed,
            corrections,
        }],
    )
    .map_err(|e| e.to_string())?;
    let mut s = Simulation::with_movement(
        "m1".into(),
        base.snapshot().date(),
        7,
        base.config().clone(),
        w,
        movement,
    )
    .map_err(|e| e.to_string())?;
    for (tick, sequence, command) in [
        (
            0,
            1,
            Command::Move {
                unit: DivisionId(900),
                destination: ProvinceId(20),
            },
        ),
        (
            2,
            2,
            Command::Stop {
                unit: DivisionId(900),
            },
        ),
        (
            10,
            3,
            Command::Move {
                unit: DivisionId(900),
                destination: ProvinceId(10),
            },
        ),
        (3, 4, Command::Pause(true)),
        (3, 5, Command::Pause(false)),
    ] {
        s.enqueue(tick, n, sequence, command)
            .map_err(|e| e.to_string())?;
    }
    Ok(s)
}
fn report(s: &Simulation) -> Result<serde_json::Value, String> {
    Ok(
        serde_json::json!({"hash":format!("{:016x}",s.state_hash().map_err(|e|e.to_string())?),"dto":s.export_save_v2()?,"canonical_hex":oh_core::canonical_bytes(s).map_err(|e|e.to_string())?.iter().map(|b|format!("{b:02x}")).collect::<String>()}),
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
            s.step().map_err(|e| e.to_string())?;
            let split = report(&s)?;
            let bytes = encode(&s, &c, 0, vec![])?;
            std::fs::write(out.join("movement-v2.ohsave"), bytes).map_err(|e| e.to_string())?;
            while s.snapshot().tick() < 24 {
                s.step().map_err(|e| e.to_string())?;
            }
            let result = serde_json::json!({"split":split,"continuous":report(&s)?});
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
            while s.snapshot().tick() < 24 {
                s.step().map_err(|e| e.to_string())?;
            }
            println!(
                "{}",
                serde_json::json!({"split":split,"continuous":report(&s)?})
            );
        }
        _ => return Err("movement_fixture capture <out> | resume <pack> <save>".into()),
    }
    Ok(())
}
