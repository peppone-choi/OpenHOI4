//! Real executable + MessagePack/WebSocket acceptance, included in cargo test (3 OS CI).
#[path = "../../oh_data/tests/support/m2_military_pack.rs"]
mod fixture;
use oh_core::{NationId, ProvinceId};
use oh_sim::{Command, Simulation, military::Action};

fn apply(sim: &mut Simulation, nation: u16, command: Command) {
    sim.enqueue(sim.snapshot().tick(), NationId(nation), 1, command)
        .unwrap();
    assert!(
        sim.step()
            .unwrap()
            .commands
            .iter()
            .all(|c| c.result.is_ok())
    );
}
#[test]
fn s1_native_movement_wire_authority_rejections_stop_and_arrival() {
    let pack = fixture::CopyPack::new();
    let paths = std::fs::read_dir(pack.root())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    let root = pack.root().join("testland");
    std::fs::create_dir(&root).unwrap();
    for path in paths {
        std::fs::rename(&path, root.join(path.file_name().unwrap())).unwrap();
    }
    let context = oh_save::SaveContext::national(&root, fixture::SCENARIO).unwrap();
    let mut sim = context.simulation(1).unwrap();
    for nation in 1..=2 {
        apply(
            &mut sim,
            nation,
            Command::Military(Action::Train {
                template: "m2_small".into(),
            }),
        );
    }
    while sim.snapshot().tick() < 80 {
        sim.step().unwrap();
    }
    apply(&mut sim, 1, Command::Pause(true));
    apply(
        &mut sim,
        2,
        Command::Military(Action::Deploy {
            job: 1,
            army: 1,
            province: ProvinceId(3),
            allow_understrength: true,
        }),
    );
    let ready = pack.root().join("ready.ohsave");
    let ready_bytes = oh_save::encode(&sim, &context, 0, vec![1, 2]).unwrap();
    std::fs::write(&ready, &ready_bytes).unwrap();
    // Synthetic disconnected allowed snapshot: no product pack or travel rule is changed.
    apply(
        &mut sim,
        1,
        Command::Military(Action::Deploy {
            job: 0,
            army: 0,
            province: ProvinceId(1),
            allow_understrength: true,
        }),
    );
    let mut dto = sim.export_save_v7().unwrap();
    let unit = dto
        .base
        .base
        .base
        .base
        .units
        .iter_mut()
        .find(|unit| unit.nation == 1)
        .unwrap();
    unit.allowed = vec![1, 2, 110];
    unit.corrections
        .retain(|edge| unit.allowed.contains(&edge.from) && unit.allowed.contains(&edge.to));
    let isolated = Simulation::from_save_v7(dto, context.restore_context()).unwrap();
    let isolated_path = pack.root().join("isolated.ohsave");
    let isolated_bytes = oh_save::encode(&isolated, &context, 0, vec![1, 2]).unwrap();
    std::fs::write(&isolated_path, &isolated_bytes).unwrap();
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let result = std::process::Command::new("node")
        .current_dir(repo)
        .args([
            "crates/oh_server/tests/movement_query.cjs",
            env!("CARGO_BIN_EXE_oh_server"),
        ])
        .arg(pack.root())
        .arg(&ready)
        .arg(&isolated_path)
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&result.stdout));
    assert!(
        result.status.success(),
        "native movement failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(std::fs::read(ready).unwrap(), ready_bytes);
    assert_eq!(std::fs::read(isolated_path).unwrap(), isolated_bytes);
}
