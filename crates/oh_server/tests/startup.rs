use std::process::Command;
#[test]
fn req_net_06_help_and_invalid_options() {
    let bin = env!("CARGO_BIN_EXE_oh_server");
    let help = Command::new(bin).arg("--help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("--open") && text.contains("127.0.0.1") && text.contains("Ctrl+C"));
    for args in [
        vec!["--bad"],
        vec!["--port", "bad"],
        vec!["--port"],
        vec!["--port", "0"],
        vec!["--open", "--open"],
    ] {
        let result = Command::new(bin).args(args).output().unwrap();
        assert!(!result.status.success());
        assert!(
            String::from_utf8(result.stderr)
                .unwrap()
                .contains("startup error")
        );
    }
}
#[test]
fn req_net_06_port_conflict() {
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_oh_server"))
        .args(["--port", &socket.local_addr().unwrap().port().to_string()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot bind")
    );
}
