//! Local authoritative host. All I/O and clocks stay here, never in oh_sim.
mod session;
use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
};
use oh_data::{DefineValue, Number};
use oh_proto::{ClientMessage, PROTOCOL_VERSION, PackInfo, ServerMessage, TimeState};
use oh_sim::{Date, Simulation, TimeConfig};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{oneshot, watch};
mod embedded {
    include!(concat!(env!("OUT_DIR"), "/assets.rs"));
}
pub const USAGE: &str = "OpenHOI4 local server\nUsage: oh_server [--open] [--port <1..65535>] [--pack-root <directory>]\nDefault: http://127.0.0.1:8080/ (loopback only)\nDefault pack-root: data/packs (M1 testland scenario m1); explicit data/packs/examples/m0 preserves M0\nBuild first: npm --prefix client ci; npm --prefix client run build; cargo build -p oh_server\nThe executable serves the built client; no Node/Vite server is needed to play.\n--open opens the address in your default browser. Ctrl+C shuts down normally.\n--help prints this help.";
#[derive(Debug)]
pub struct Options {
    pub port: u16,
    pub open: bool,
    pub pack_root: PathBuf,
}
impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            port: 8080,
            open: false,
            pack_root: "data/packs".into(),
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut i = 0;
        while i < args.len() {
            let key = args[i].as_str();
            if !seen.insert(key) {
                return Err(format!("duplicate option {key}"));
            }
            match key {
                "--open" => options.open = true,
                "--port" | "--pack-root" => {
                    i += 1;
                    let value = args
                        .get(i)
                        .filter(|v| !v.starts_with("--"))
                        .ok_or_else(|| format!("missing value for {key}"))?;
                    if key == "--port" {
                        options.port = value
                            .parse()
                            .ok()
                            .filter(|p| *p != 0)
                            .ok_or("--port must be an integer in 1..65535")?;
                    } else {
                        options.pack_root = value.into();
                    }
                }
                _ => return Err(format!("unknown option {key}\n{USAGE}")),
            }
            i += 1;
        }
        Ok(options)
    }
}
#[derive(Clone)]
pub struct Host {
    loaded: Arc<oh_data::m0::LoadedM0>,
    world: Option<oh_sim::world::World>,
    root: PathBuf,
    date: Date,
    time: TimeConfig,
    pub pack: PackInfo,
    delta_ms: u64,
    capacity: usize,
    max_message: usize,
    handshake_ms: u64,
    default_seed: u64,
    shutdown: watch::Receiver<bool>,
}
fn positive(defines: &oh_data::Defines, key: &str) -> Result<u64, String> {
    match defines.get(key) {
        Some(DefineValue::Number(Number::Integer(n))) if *n > 0 => Ok(*n as u64),
        _ => Err(format!("{key} must be a positive integer in defines.toml")),
    }
}
impl Host {
    pub fn load(root: &Path, shutdown: watch::Receiver<bool>) -> Result<Self, String> {
        let (loaded, world) = if root.join("testland/scenarios/m1/scenario.toml").exists() {
            let national = oh_data::national::load_scenario(&root.join("testland"), "m1")
                .map_err(|e| e.to_string())?;
            let world = oh_sim::world::World::from_loaded(&national)?;
            (
                oh_data::m0::LoadedM0 {
                    pack: national.pack,
                    scenario_id: "m1".into(),
                    start_date: national.scenario.start_date,
                },
                Some(world),
            )
        } else {
            (oh_data::m0::load_m0_scenario(root, "testland")?, None)
        };
        let fields = loaded.start_date.split('-').collect::<Vec<_>>();
        if fields.len() != 3
            || fields
                .iter()
                .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err("invalid start_date YYYY-MM-DD".into());
        }
        let date = Date::new(
            fields[0].parse().map_err(|_| "invalid year")?,
            fields[1].parse().map_err(|_| "invalid month")?,
            fields[2].parse().map_err(|_| "invalid day")?,
        )
        .map_err(|e| e.to_string())?;
        let time = TimeConfig::from_defines(&loaded.pack.defines).map_err(|e| e.to_string())?;
        let delta_ms = positive(&loaded.pack.defines, "network.delta_ms")?;
        if delta_ms < 100 {
            return Err("network.delta_ms must be at least 100 (02 section 4.2)".into());
        }
        let capacity = usize::try_from(positive(&loaded.pack.defines, "network.command_capacity")?)
            .map_err(|_| "command capacity overflow")?;
        let max_message =
            usize::try_from(positive(&loaded.pack.defines, "network.max_message_bytes")?)
                .map_err(|_| "message size overflow")?;
        let default_seed = positive(&loaded.pack.defines, "network.default_seed")?;
        let handshake_ms = positive(&loaded.pack.defines, "network.handshake_timeout_ms")?;
        let path = root.join("testland");
        let scenario_file = if world.is_some() {
            "scenarios/m1/scenario.toml"
        } else {
            "scenarios/testland/scenario.toml"
        };
        let files = ["manifest.toml", "defines.toml", scenario_file]
            .into_iter()
            .map(|p| std::fs::read(path.join(p)).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let hash = if world.is_some() {
            pack_hash(&path)?
        } else {
            oh_core::state_hash(&files).map_err(|e| e.to_string())?
        };
        let pack = PackInfo {
            id: loaded.pack.manifest.id.clone(),
            version: loaded.pack.manifest.version.clone(),
            hash: format!("{hash:016x}"),
        };
        Ok(Self {
            loaded: Arc::new(loaded),
            world,
            root: path,
            date,
            time,
            pack,
            delta_ms,
            capacity,
            max_message,
            handshake_ms,
            default_seed,
            shutdown,
        })
    }
    fn simulation(&self, seed: u64) -> Result<Simulation, String> {
        let mut sim = Simulation::new(
            self.loaded.scenario_id.clone(),
            self.date,
            seed,
            self.time.clone(),
        )
        .map_err(|e| e.to_string())?;
        if let Some(world) = self.world.clone() {
            sim = Simulation::with_world(
                self.loaded.scenario_id.clone(),
                self.date,
                seed,
                self.time.clone(),
                world,
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(sim)
    }
}
fn pack_hash(root: &Path) -> Result<u64, String> {
    fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        let mut paths = std::fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .map(|e| e.map(|e| e.path()).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort();
        for p in paths {
            if p.is_symlink() {
                return Err("pack symlinks unsupported".into());
            }
            if p.is_dir() {
                collect(root, &p, out)?;
            } else {
                out.push((
                    p.strip_prefix(root)
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .replace('\\', "/"),
                    std::fs::read(&p).map_err(|e| e.to_string())?,
                ));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    collect(root, root, &mut files)?;
    oh_core::state_hash(&files).map_err(|e| e.to_string())
}
pub fn router(host: Host) -> Router {
    Router::new()
        .route("/ws", get(upgrade))
        .route("/pack/localisation/{language}/{file}", get(pack_locale))
        .fallback(get(static_file))
        .with_state(host)
}
async fn pack_locale(
    State(host): State<Host>,
    axum::extract::Path((language, file)): axum::extract::Path<(String, String)>,
) -> Response {
    if !["en", "ko"].contains(&language.as_str())
        || !["map.ftl", "national.ftl"].contains(&file.as_str())
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    match std::fs::read(host.root.join("localisation").join(language).join(file)) {
        Ok(data) => ([("content-type", "text/plain; charset=utf-8")], data).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
async fn static_file(uri: Uri) -> Response {
    let path = if uri.path() == "/" {
        "/index.html"
    } else {
        uri.path()
    };
    match embedded::asset(path) {
        Some((mime, data)) => (
            [("content-type", mime), ("cache-control", "no-cache")],
            data,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }
}
async fn upgrade(State(host): State<Host>, ws: WebSocketUpgrade) -> Response {
    ws.max_message_size(host.max_message)
        .on_upgrade(move |socket| connection(socket, host))
}
async fn send(socket: &mut WebSocket, message: ServerMessage) -> bool {
    let Ok(bytes) = oh_proto::encode(&message) else {
        return false;
    };
    socket.send(Message::Binary(bytes.into())).await.is_ok()
}
async fn notice(socket: &mut WebSocket, key: &str) {
    let _ = send(socket, ServerMessage::Notice { key: key.into() }).await;
}
async fn connection(mut socket: WebSocket, mut host: Host) {
    let hello = tokio::select! {
        result = tokio::time::timeout(Duration::from_millis(host.handshake_ms), socket.recv()) => result.ok().flatten(),
        _ = host.shutdown.changed() => None,
    };
    let version = match hello {
        Some(Ok(Message::Binary(bytes))) => match oh_proto::decode_client(&bytes) {
            Ok(ClientMessage::Hello { protocol_version }) => protocol_version,
            _ => {
                notice(&mut socket, "hello-required").await;
                let _ = socket.send(Message::Close(None)).await;
                return;
            }
        },
        _ => {
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };
    let accepted = version == PROTOCOL_VERSION;
    if !send(
        &mut socket,
        ServerMessage::Welcome {
            engine_version: env!("CARGO_PKG_VERSION").into(),
            protocol_version: PROTOCOL_VERSION.into(),
            accepted,
            reason_key: (!accepted).then(|| "protocol-version".into()),
            packs: vec![host.pack.clone()],
            sessions: vec!["local".into()],
        },
    )
    .await
    {
        return;
    }
    if !accepted {
        let _ = socket.send(Message::Close(None)).await;
        return;
    }
    let mut active: Option<session::Session> = None;
    let mut client_sequence = 0u64;
    let mut delta_sequence = 0u64;
    let mut current_state: Option<TimeState> = None;
    loop {
        // Before Join, wait for frames; after Join also wait for thread output.
        let update = async {
            if let Some(session) = active.as_mut() {
                session
                    .states
                    .changed()
                    .await
                    .map(|()| session.states.borrow().clone())
                    .ok()
            } else {
                std::future::pending::<Option<TimeState>>().await
            }
        };
        tokio::select! {
            _ = host.shutdown.changed() => { let _ = socket.send(Message::Close(None)).await; break; }
            state = update => {
                let Some(state) = state else { notice(&mut socket, "simulation-error").await; break; };
                let Some(next) = delta_sequence.checked_add(1) else { break; }; delta_sequence = next;
                current_state = Some(state.clone());
                if !send(&mut socket, ServerMessage::Delta { sequence: delta_sequence.to_string(), state }).await { break; }
            }
            frame = socket.recv() => {
                let message = match frame {
                    Some(Ok(Message::Binary(bytes))) => match oh_proto::decode_client(&bytes) { Ok(m) => m, Err(_) => { notice(&mut socket, "invalid-message").await; continue; } },
                    Some(Ok(Message::Ping(bytes))) => { if socket.send(Message::Pong(bytes)).await.is_err() { break; } continue; },
                    Some(Ok(Message::Pong(_))) => continue,
                    Some(Ok(Message::Text(_))) => { notice(&mut socket, "invalid-message").await; continue; },
                    _ => break,
                };
                match message {
                    ClientMessage::Hello { .. } => notice(&mut socket, "invalid-message").await,
                    ClientMessage::Create { scenario, seed, mode } => {
                        if active.is_some() { notice(&mut socket, "already-joined").await; continue; }
                        let Ok(seed) = seed.parse::<u64>() else { notice(&mut socket, "unsupported-create").await; continue; };
                        if scenario != host.loaded.scenario_id || mode != "single" { notice(&mut socket, "unsupported-create").await; continue; }
                        let Ok(sim) = host.simulation(seed) else { notice(&mut socket, "simulation-error").await; break; };
                        let state = TimeState::from(&sim.snapshot()); current_state = Some(state.clone()); active = Some(session::Session::start(sim, host.capacity, host.delta_ms));
                        if !send(&mut socket, ServerMessage::Snapshot { state }).await { break; }
                    }
                    ClientMessage::Join { session, nation } => {
                        if active.is_some() { notice(&mut socket, "already-joined").await; continue; }
                        if session != "local" || nation.is_some() { notice(&mut socket, "unsupported-session").await; continue; }
                        let Ok(sim) = host.simulation(host.default_seed) else { notice(&mut socket, "simulation-error").await; break; };
                        let state = TimeState::from(&sim.snapshot()); current_state = Some(state.clone()); active = Some(session::Session::start(sim, host.capacity, host.delta_ms));
                        if !send(&mut socket, ServerMessage::Snapshot { state }).await { break; }
                    }
                    ClientMessage::Command { sequence, command } => {
                        let parsed = sequence.parse::<u64>();
                        let reason = if !parsed.as_ref().is_ok_and(|s| *s > client_sequence) { Some("invalid-sequence") } else if active.is_none() { Some("not-joined") } else { None };
                        if let Some(reason) = reason {
                            if !send(&mut socket, ServerMessage::CommandResult { sequence, accepted: false, reason_key: Some(reason.into()) }).await { break; } continue;
                        }
                        client_sequence = parsed.unwrap();
                        let (reply, received) = oneshot::channel();
                        if active.as_ref().unwrap().commands.try_send(session::Request::Command { command, reply }).is_err() { notice(&mut socket, "session-closed").await; break; }
                        let result = tokio::select! { result = received => result.unwrap_or(Err("session-closed")), _ = host.shutdown.changed() => break };
                        let reason_key = result.as_ref().err().map(|r| (*r).into());
                        if !send(&mut socket, ServerMessage::CommandResult { sequence, accepted: result.is_ok(), reason_key }).await { break; }
                        if let Ok(state) = result {
                            // Discard a pending older publication before sending the command snapshot.
                            active.as_mut().unwrap().states.borrow_and_update();
                            current_state = Some(state.clone());
                            if !send(&mut socket, ServerMessage::Snapshot { state }).await { break; }
                        }
                    }
                    ClientMessage::Query { request, kind } => {
                        if kind=="world" || kind.starts_with("nation:") || kind.starts_with("state:") {
                            let mut reason=Some("not-joined"); let mut world=None;
                            if let Some(session)=active.as_ref(){
                                let (reply,received)=oneshot::channel();
                                if session.commands.try_send(session::Request::World{reply}).is_ok(){
                                    world=tokio::select!{value=received=>value.ok().flatten(),_=host.shutdown.changed()=>break};
                                    reason=if world.is_some(){None}else{Some("unsupported-query")};
                                    if let Some(view)=world.as_mut(){
                                        if let Some(id)=kind.strip_prefix("nation:"){let id=id.parse::<u16>().ok();view.nations.retain(|n|Some(n.id)==id);if view.nations.is_empty(){reason=Some("unknown-entity");}}
                                        if let Some(id)=kind.strip_prefix("state:"){let id=id.parse::<u16>().ok();view.states.retain(|s|Some(s.id)==id);if view.states.is_empty(){reason=Some("unknown-entity");}}
                                    }
                                }else{reason=Some("session-closed");}
                            }
                            if reason.is_some(){world=None;}
                            if !send(&mut socket,ServerMessage::WorldResult{request,supported:world.is_some(),reason_key:reason.map(str::to_owned),world}).await{break;}continue;
                        }
                        let state = current_state.clone().filter(|_| kind == "time");
                        let reason_key = if kind != "time" { Some("unsupported-query".into()) } else if state.is_none() { Some("not-joined".into()) } else { None };
                        if !send(&mut socket, ServerMessage::QueryResult { request, supported: state.is_some(), reason_key, state }).await { break; }
                    }
                }
            }
        }
    }
    if let Some(session) = active {
        session.stop().await;
    }
}
/// Platform browser launcher. Arguments are passed separately, never via a shell.
pub fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut c = std::process::Command::new("rundll32.exe");
        c.arg("url.dll,FileProtocolHandler");
        c
    };
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");
    let status = command
        .arg(url)
        .status()
        .map_err(|e| format!("cannot open browser: {e}; open {url} manually"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "browser launcher failed ({status}); open {url} manually"
        ))
    }
}

#[cfg(test)]
mod national_tests {
    use super::*;
    #[test]
    fn default_host_loads_m1_and_m0_requires_its_explicit_path() {
        let (_, shutdown) = watch::channel(false);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs");
        let host = Host::load(&root, shutdown.clone()).unwrap();
        let sim = host.simulation(1).unwrap();
        assert_eq!(sim.snapshot().scenario(), "m1");
        assert!(sim.world().is_some());
        let m0 = Host::load(&root.join("examples/m0"), shutdown)
            .unwrap()
            .simulation(1)
            .unwrap();
        assert!(m0.world().is_none());
        assert_eq!(m0.snapshot().scenario(), "testland");
        assert_eq!(
            Options::parse(&[]).unwrap().pack_root,
            PathBuf::from("data/packs")
        );
    }
}
