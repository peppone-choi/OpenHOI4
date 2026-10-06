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
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/wp05");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("wire-fixtures.json"),
        serde_json::to_vec_pretty(&fixtures).unwrap(),
    )
    .unwrap();
}
