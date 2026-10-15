use oh_proto::ClientMessage;

#[test]
fn movement_command_wire_decodes_for_joined_nation_dispatch() {
    let raw = r#"{"type":"MovementCommand","sequence":"1","command":{"type":"Move","division":"0","destination":2}}"#;
    let result = serde_json::from_str::<ClientMessage>(raw);
    assert!(result.is_ok(), "S1 movement wire must decode: {result:?}");
}
