#[path = "../../oh_save/tests/support/military.rs"]
mod military_fixture;
#[path = "../../oh_data/tests/support/production.rs"]
mod production_fixture;
use oh_proto::*;
use serde::Serialize;
fn add<T: Serialize>(fixtures: &mut Vec<serde_json::Value>, name: &str, message: T) {
    fixtures.push(
        serde_json::json!({"name": name, "bytes": encode(&message).unwrap(), "expected": message}),
    );
}
fn main() {
    let state = TimeState {
        date: "2000-02-29".into(),
        hour: 23,
        tick: u64::MAX.to_string(),
        paused: true,
        speed: 5,
    };
    let mut fixtures = Vec::new();
    add(
        &mut fixtures,
        "Hello",
        ClientMessage::Hello {
            protocol_version: PROTOCOL_VERSION.into(),
        },
    );
    add(
        &mut fixtures,
        "Create",
        ClientMessage::Create {
            scenario: "testland".into(),
            seed: u64::MAX.to_string(),
            mode: "single".into(),
        },
    );
    add(
        &mut fixtures,
        "Join",
        ClientMessage::Join {
            session: "local".into(),
            nation: None,
        },
    );
    add(
        &mut fixtures,
        "Pause",
        ClientMessage::Command {
            sequence: "1".into(),
            command: TimeCommand::Pause { paused: true },
        },
    );
    add(
        &mut fixtures,
        "SetSpeed",
        ClientMessage::Command {
            sequence: "2".into(),
            command: TimeCommand::SetSpeed { speed: 5 },
        },
    );
    add(
        &mut fixtures,
        "Query",
        ClientMessage::Query {
            request: "q".into(),
            kind: "time".into(),
        },
    );
    add(
        &mut fixtures,
        "Welcome",
        ServerMessage::Welcome {
            engine_version: "0.1.0".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            accepted: true,
            reason_key: None,
            packs: vec![PackInfo {
                id: "m0_testland".into(),
                version: "0.1.0".into(),
                hash: "0123".into(),
            }],
            sessions: vec!["local".into()],
        },
    );
    add(
        &mut fixtures,
        "CommandResult",
        ServerMessage::CommandResult {
            sequence: "2".into(),
            accepted: false,
            reason_key: Some("invalid-speed".into()),
        },
    );
    add(
        &mut fixtures,
        "Snapshot",
        ServerMessage::Snapshot {
            state: state.clone(),
        },
    );
    add(
        &mut fixtures,
        "Delta",
        ServerMessage::Delta {
            sequence: u64::MAX.to_string(),
            state: state.clone(),
        },
    );
    add(
        &mut fixtures,
        "QueryResult",
        ServerMessage::QueryResult {
            request: "q".into(),
            supported: true,
            reason_key: None,
            state: Some(state),
        },
    );
    add(
        &mut fixtures,
        "Notice",
        ServerMessage::Notice {
            key: "unsupported-query".into(),
        },
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    let sim = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap();
    let mut national = Vec::new();
    add(
        &mut national,
        "WorldResult",
        ServerMessage::WorldResult {
            request: "world".into(),
            supported: true,
            reason_key: None,
            world: WorldView::from_sim(&sim),
        },
    );
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/wp09");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("national-wire-fixtures.json"),
        serde_json::to_vec_pretty(&national).unwrap(),
    )
    .unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/wp05");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("wire-fixtures.json"),
        serde_json::to_vec_pretty(&fixtures).unwrap(),
    )
    .unwrap();
    let mut economic = loaded.clone();
    economic.scenario.economy = Some("synthetic".into());
    economic.economy = Some(
        serde_json::from_str(include_str!(
            "../../oh_data/tests/fixtures/economy/valid.json"
        ))
        .unwrap(),
    );
    let mut sim = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1000,
        oh_sim::TimeConfig::from_defines(&economic.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&economic).unwrap(),
    )
    .unwrap();
    sim.enqueue(
        0,
        oh_core::NationId(1),
        1,
        oh_sim::Command::Economy(oh_sim::economy::Action::Construct {
            project: 7,
            state: 1,
            building: "industry".into(),
        }),
    )
    .unwrap();
    for _ in 0..24 {
        sim.step().unwrap();
    }
    let mut economic_wire = Vec::new();
    add(
        &mut economic_wire,
        "EconomyResult",
        ServerMessage::EconomyResult {
            request: "economy".into(),
            supported: true,
            reason_key: None,
            economy: EconomyView::from_sim(&sim),
        },
    );
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/wp14");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("economy-wire-fixtures.json"),
        serde_json::to_vec_pretty(&economic_wire).unwrap(),
    )
    .unwrap();
    let production_root = production_fixture::pack();
    let l = oh_data::national::load_scenario(&production_root, "m1").unwrap();
    let mut p = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&l.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&l).unwrap(),
    )
    .unwrap();
    p.enqueue(
        0,
        oh_core::NationId(1),
        1,
        oh_sim::Command::Production(oh_sim::production::Action::Create {
            model: "test_model_1".into(),
            requested_ic: oh_core::Qty::from_num(1),
        }),
    )
    .unwrap();
    for _ in 0..24 {
        p.step().unwrap();
    }
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/wp15");
    std::fs::create_dir_all(&output).unwrap();
    let message = ServerMessage::ProductionResult {
        request: "production".into(),
        supported: true,
        reason_key: None,
        production: ProductionView::from_sim(&p),
    };
    std::fs::write(
        output.join("production-wire-fixture.json"),
        serde_json::to_vec_pretty(&message).unwrap(),
    )
    .unwrap();
    std::fs::write(
        output.join("fixture-pack-path.txt"),
        production_root.to_string_lossy().as_bytes(),
    )
    .unwrap();
    let root = military_fixture::pack();
    let loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    let mut military = oh_sim::Simulation::with_world(
        "m1".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap();
    military
        .enqueue(
            0,
            oh_core::NationId(1),
            1,
            oh_sim::Command::Military(oh_sim::military::Action::Train {
                template: "example".into(),
            }),
        )
        .unwrap();
    for _ in 0..72 {
        military.step().unwrap();
    }
    military
        .enqueue(
            100,
            oh_core::NationId(1),
            2,
            oh_sim::Command::Military(oh_sim::military::Action::SetPriority {
                army: 0,
                priority: 2,
            }),
        )
        .unwrap();
    let message = ServerMessage::MilitaryResult {
        request: "military".into(),
        supported: true,
        reason_key: None,
        military: MilitaryView::from_sim(&military),
    };
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/wp16");
    std::fs::create_dir_all(&output).unwrap();
    std::fs::write(
        output.join("military-wire-fixture.json"),
        serde_json::to_vec_pretty(&message).unwrap(),
    )
    .unwrap();
}
