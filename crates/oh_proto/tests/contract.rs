use oh_proto::{ClientMessage, PROTOCOL_VERSION, ServerMessage, TimeState};
#[test]
fn req_net_01_messagepack_roundtrip() {
    let hello = ClientMessage::Hello {
        protocol_version: PROTOCOL_VERSION.into(),
    };
    assert_eq!(
        oh_proto::decode_client(&oh_proto::encode(&hello).unwrap()).unwrap(),
        hello
    );
    let snapshot = ServerMessage::Snapshot {
        state: TimeState {
            date: "2000-01-01".into(),
            hour: 0,
            tick: "0".into(),
            paused: true,
            speed: 1,
        },
    };
    assert_eq!(
        oh_proto::decode_server(&oh_proto::encode(&snapshot).unwrap()).unwrap(),
        snapshot
    );
}
#[test]
fn req_net_04_generated_types_are_current() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client/src/proto/protocol.ts");
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        oh_proto::typescript()
    );
}
