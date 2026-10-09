#[path = "../../oh_data/tests/support/o2_production_pack.rs"]
mod fixture;
use oh_core::NationId;
use oh_server::Host;
use oh_sim::{Command, production::Action};

fn stage() -> fixture::CopyPack {
    let pack = fixture::CopyPack::new();
    let paths = std::fs::read_dir(pack.root())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    let child = pack.root().join("testland");
    std::fs::create_dir(&child).unwrap();
    for path in paths {
        std::fs::rename(&path, child.join(path.file_name().unwrap())).unwrap();
    }
    pack
}

#[test]
fn dedicated_pack_starts_and_restores_real_v6_six_nation_lines() {
    let pack = stage();
    let (_tx, shutdown) = tokio::sync::watch::channel(false);
    let host = Host::load_selected(
        pack.root(),
        shutdown.clone(),
        Some(fixture::SCENARIO),
        None,
        false,
    )
    .unwrap();
    assert_eq!(host.pack.id, "testland_m2_production");
    let context =
        oh_save::SaveContext::national(&pack.root().join("testland"), fixture::SCENARIO).unwrap();
    let mut sim = context.simulation(7).unwrap();
    let definitions = sim.world().unwrap().defs().production().unwrap().clone();
    for (nation, input) in definitions.nations {
        let requested_ic = sim
            .economy()
            .unwrap()
            .nation(NationId(nation))
            .unwrap()
            .ledger()
            .allocation[2];
        sim.enqueue(
            0,
            NationId(nation),
            1,
            Command::Production(Action::Create {
                model: input.allowed_models.first().unwrap().clone(),
                requested_ic,
            }),
        )
        .unwrap();
    }
    for _ in 0..180 * 24 {
        sim.step().unwrap();
    }
    assert_eq!(sim.production().unwrap().lines().len(), 6);
    let bytes = oh_save::encode(&sim, &context, 0, vec![1, 2, 3, 4, 5, 6]).unwrap();
    assert_eq!(
        oh_save::inspect_header(&bytes, &Default::default())
            .unwrap()
            .format_version,
        6
    );
    let file = pack.root().join("resume.ohsave");
    std::fs::write(&file, bytes).unwrap();
    let restored = Host::load_selected(
        pack.root(),
        shutdown,
        Some(fixture::SCENARIO),
        Some(&file),
        false,
    )
    .unwrap();
    assert_eq!(host.pack, restored.pack);
}

#[test]
fn server_rejects_unused_producer_and_missing_translation_with_valid_controls() {
    for mutation in ["unused", "translation"] {
        let pack = stage();
        let (_tx, shutdown) = tokio::sync::watch::channel(false);
        assert!(
            Host::load_selected(
                pack.root(),
                shutdown.clone(),
                Some(fixture::SCENARIO),
                None,
                false
            )
            .is_ok()
        );
        let root = pack.root().join("testland");
        if mutation == "unused" {
            std::fs::write(root.join("common/production/unused.toml"), "").unwrap();
        } else {
            let path = root.join("localisation/en/production.ftl");
            let source = std::fs::read_to_string(&path).unwrap();
            let reduced = source
                .lines()
                .filter(|line| !line.starts_with("o2-equipment-2 ="))
                .collect::<Vec<_>>()
                .join("\n");
            assert_ne!(source.trim(), reduced.trim());
            std::fs::write(path, reduced).unwrap();
        }
        assert!(
            Host::load_selected(pack.root(), shutdown, Some(fixture::SCENARIO), None, false)
                .is_err(),
            "{mutation}"
        );
    }
}
