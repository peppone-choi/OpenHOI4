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

// Exercise the wire directly: a received client Close must be acknowledged
// before TCP EOF, independently of any browser/Node error handling.
#[test]
fn req_net_01_client_close_handshake() {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::time::{Duration, Instant};

    struct Server(std::process::Child);
    impl Drop for Server {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn send(socket: &mut TcpStream, opcode: u8, payload: &[u8]) {
        assert!(payload.len() < 126);
        let mask = [1, 2, 3, 4];
        let mut bytes = vec![0x80 | opcode, 0x80 | payload.len() as u8];
        bytes.extend(mask);
        bytes.extend(payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
        socket.write_all(&bytes).unwrap();
    }
    fn receive(socket: &mut TcpStream) -> (u8, Vec<u8>) {
        let mut header = [0; 2];
        socket
            .read_exact(&mut header)
            .expect("frame before TCP EOF");
        assert_eq!(header[1] & 0x80, 0, "server frames are unmasked");
        let length = match header[1] {
            126 => {
                let mut length = [0; 2];
                socket.read_exact(&mut length).unwrap();
                u16::from_be_bytes(length) as usize
            }
            127 => panic!("unexpected large lifecycle frame"),
            length => length as usize,
        };
        let mut payload = vec![0; length];
        socket.read_exact(&mut payload).unwrap();
        (header[0] & 15, payload)
    }

    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut server = Server(
        Command::new(env!("CARGO_BIN_EXE_oh_server"))
            .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .args(["--port", &port.to_string()])
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    for stage in 0..=2 {
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut socket = loop {
            if let Ok(socket) = TcpStream::connect(("127.0.0.1", port)) {
                break socket;
            }
            assert!(server.0.try_wait().unwrap().is_none());
            assert!(Instant::now() < deadline, "server startup deadline");
            std::thread::sleep(Duration::from_millis(10));
        };
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket.write_all(format!("GET /ws HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n").as_bytes()).unwrap();
        let mut headers = Vec::new();
        while !headers.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            socket.read_exact(&mut byte).unwrap();
            headers.push(byte[0]);
        }
        assert!(headers.starts_with(b"HTTP/1.1 101"));
        if stage >= 1 {
            send(
                &mut socket,
                2,
                &oh_proto::encode(&oh_proto::ClientMessage::Hello {
                    protocol_version: oh_proto::PROTOCOL_VERSION.into(),
                })
                .unwrap(),
            );
            let (_, welcome) = receive(&mut socket);
            assert!(matches!(
                oh_proto::decode_server(&welcome).unwrap(),
                oh_proto::ServerMessage::Welcome { accepted: true, .. }
            ));
        }
        if stage == 2 {
            send(
                &mut socket,
                2,
                &oh_proto::encode(&oh_proto::ClientMessage::Join {
                    session: "local".into(),
                    nation: None,
                })
                .unwrap(),
            );
            let (_, snapshot) = receive(&mut socket);
            assert!(matches!(
                oh_proto::decode_server(&snapshot).unwrap(),
                oh_proto::ServerMessage::Snapshot { .. }
            ));
        }
        let close = [1000u16.to_be_bytes().as_slice(), b"acceptance"].concat();
        send(&mut socket, 8, &close);
        loop {
            let (opcode, payload) = receive(&mut socket);
            if opcode == 8 {
                assert_eq!(payload, close, "echo close status and reason");
                break;
            }
            assert_eq!(opcode, 2, "only pending binary updates before Close");
        }
        assert_eq!(socket.read(&mut [0]).unwrap(), 0, "TCP EOF after Close");
    }
}
