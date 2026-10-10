#[path = "../../oh_data/tests/support/m2_military_pack.rs"]
mod fixture;
use oh_core::NationId;
use oh_server::Host;
use oh_sim::{Command, military::Action, production};
fn stage() -> fixture::CopyPack {
    let p = fixture::CopyPack::new();
    let paths = std::fs::read_dir(p.root())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    let child = p.root().join("testland");
    std::fs::create_dir(&child).unwrap();
    for path in paths {
        std::fs::rename(&path, child.join(path.file_name().unwrap())).unwrap();
    }
    p
}
#[test]
fn host_accepts_fresh_and_real_paused_v7() {
    let p = stage();
    let root = p.root().join("testland");
    let (_, shutdown) = tokio::sync::watch::channel(false);
    let fresh = Host::load_selected(
        p.root(),
        shutdown.clone(),
        Some(fixture::SCENARIO),
        None,
        false,
    )
    .unwrap();
    assert_eq!(fresh.pack.id, "testland_m2_military");
    let context = oh_save::SaveContext::national(&root, fixture::SCENARIO).unwrap();
    let mut sim = context.simulation(1).unwrap();
    for n in 1..=6 {
        let ic = sim
            .economy()
            .unwrap()
            .nation(NationId(n))
            .unwrap()
            .ledger()
            .allocation[2];
        sim.enqueue(
            0,
            NationId(n),
            1,
            Command::Production(production::Action::Create {
                model: "m2_equipment_1".into(),
                requested_ic: ic,
            }),
        )
        .unwrap();
    }
    for _ in 0..120 {
        sim.step().unwrap();
    }
    for n in 1..=6 {
        sim.enqueue(
            120,
            NationId(n),
            2,
            Command::Military(Action::Train {
                template: "m2_small".into(),
            }),
        )
        .unwrap();
    }
    for _ in 0..24 {
        sim.step().unwrap();
    }
    sim.enqueue(144, NationId(1), 3, Command::Pause(true))
        .unwrap();
    sim.step().unwrap();
    assert!(
        sim.military()
            .unwrap()
            .jobs()
            .values()
            .all(|j| j.status() == oh_sim::military::JobStatus::Training)
    );
    let save = p.root().join("paused.ohsave");
    std::fs::write(
        &save,
        oh_save::encode(&sim, &context, 0, vec![1, 2, 3, 4, 5, 6]).unwrap(),
    )
    .unwrap();
    let resumed = Host::load_selected(
        p.root(),
        shutdown,
        Some(fixture::SCENARIO),
        Some(&save),
        false,
    )
    .unwrap();
    assert_eq!(fresh.pack, resumed.pack);
}
#[test]
fn host_rejects_unused_military_missing_translation_and_missing_producer_with_controls() {
    for mutation in ["unused", "translation", "production"] {
        let p = stage();
        let (_, shutdown) = tokio::sync::watch::channel(false);
        assert!(
            Host::load_selected(
                p.root(),
                shutdown.clone(),
                Some(fixture::SCENARIO),
                None,
                false
            )
            .is_ok()
        );
        let root = p.root().join("testland");
        match mutation {
            "unused" => std::fs::write(root.join("common/military/unused.toml"), "").unwrap(),
            "translation" => {
                let path = root.join("localisation/en/pack.ftl");
                let text = std::fs::read_to_string(&path).unwrap();
                assert!(text.contains("m2-military-pack-name ="));
                std::fs::write(path, "").unwrap();
            }
            "production" => {
                let path = root.join("scenarios/m2_military/scenario.toml");
                let text = std::fs::read_to_string(&path).unwrap();
                assert!(text.contains("production ="));
                std::fs::write(
                    path,
                    text.lines()
                        .filter(|l| !l.starts_with("production ="))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
                .unwrap();
            }
            _ => unreachable!(),
        };
        assert!(
            Host::load_selected(p.root(), shutdown, Some(fixture::SCENARIO), None, false).is_err(),
            "{mutation}"
        );
    }
}
