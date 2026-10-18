use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

fn node(script: &str, args: &[&Path]) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let result = std::process::Command::new("node")
        .current_dir(repo)
        .args(["-e", script, env!("CARGO_BIN_EXE_oh_server")])
        .args(args)
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&result.stdout));
    assert!(
        result.status.success(),
        "pack acceptance: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn named_pack_join_snapshot_and_metadata_match_legacy_copy() {
    let root = fixture();
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland_m2_military");
    copy(&source, &root.join("legacy/testland"));
    node(
        r#"
const assert = require('node:assert/strict');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {encode, decode} = require(process.cwd() + '/client/node_modules/@msgpack/msgpack');
const [binary, root] = process.argv.slice(1);
const children = [], sockets = [];
const sleep = ms => new Promise(r => setTimeout(r, ms));
const watchdog = setTimeout(() => { for (const c of children) c.kill('SIGKILL'); process.exitCode = 1; }, 30000);
async function inspect(args) {
  const probe = net.createServer(); await new Promise(r => probe.listen(0, '127.0.0.1', r));
  const port = probe.address().port; await new Promise(r => probe.close(r));
  const child = spawn(binary, ['--port', String(port), ...args], {windowsHide: true}); children.push(child);
  let stderr = ''; child.stderr.on('data', b => stderr += b); child.stdout.resume();
  const url = `http://127.0.0.1:${port}`;
  let ready = false;
  for (let i = 0; i < 200; i++) {
    assert.equal(child.exitCode, null, stderr);
    try { if ((await fetch(url)).ok) { ready = true; break; } } catch {}
    await sleep(25);
  }
  assert.ok(ready, stderr);
  const ws = new WebSocket(url.replace('http', 'ws') + '/ws'); sockets.push(ws); ws.binaryType = 'arraybuffer';
  const messages = [];
  ws.addEventListener('message', e => messages.push(decode(new Uint8Array(e.data))));
  await new Promise((resolve, reject) => { ws.addEventListener('open', resolve, {once:true}); ws.addEventListener('error', reject, {once:true}); });
  const wait = async type => {
    for (let i = 0; i < 200; i++) { const m = messages.find(m => m.type === type); if (m) return m; await sleep(25); }
    throw Error(`no ${type}`);
  };
  ws.send(encode({type:'Hello', protocol_version:'m0-v1'})); assert.equal((await wait('Welcome')).accepted, true);
  ws.send(encode({type:'Join', session:'local', nation:'N01'}));
  const snapshot = await wait('Snapshot'); assert.ok(snapshot.world.map_id);
  const response = await fetch(`${url}/maps/${snapshot.world.map_id}/metadata`); assert.equal(response.status, 200);
  const metadata = await response.json(); assert.ok(metadata.pack_hash);
  return {map_id:snapshot.world.map_id, pack_hash:metadata.pack_hash};
}
(async () => {
  try {
    const named = await inspect(['--pack-root', 'data/packs', '--pack', 'testland_m2_military', '--scenario', 'm2_military']);
    const legacy = await inspect(['--pack-root', root + '/legacy', '--scenario', 'm2_military']);
    assert.deepEqual(named, legacy); console.log(JSON.stringify({named, legacy}));
  } finally {
    for (const ws of sockets) ws.close();
    for (const child of children) {
      if (child.exitCode === null && child.signalCode === null) {
        const exited = new Promise(r => child.once('exit', r)); child.kill('SIGKILL'); await exited;
      }
      assert.ok(child.exitCode !== null || child.signalCode !== null, 'owned server exited');
    }
    clearTimeout(watchdog);
  }
})().catch(e => {console.error(e); process.exitCode = 1;});
"#,
        &[&root],
    );
}

#[test]
fn named_pack_errors_precede_port_binding() {
    let root = fixture();
    copy(&root.join("testland"), &root.join("renamed"));
    node(
        r#"
const assert = require('node:assert/strict');
const net = require('node:net');
const {spawn} = require('node:child_process');
const [binary, root] = process.argv.slice(1);
(async () => {
  const probe = net.createServer(); await new Promise(r => probe.listen(0, '127.0.0.1', r));
  try {
    const cases = [
      [['--pack', '../x'], /invalid pack ID/],
      [['--pack', 'Testland'], /invalid pack ID/],
      [['--pack', 'absent_pack'], /absent_pack/],
      [['--pack', 'testland', '--pack', 'testland'], /duplicate option --pack/],
      [['--pack', 'testland', '--scenario', 'absent_scenario'], /InvalidSelectedScenario/],
      [['--pack-root', root, '--pack', 'renamed'], /pack ID mismatch/],
      [['--pack'], /missing value for --pack/],
    ];
    for (const [args, expected] of cases) {
      const child = spawn(binary, ['--port', String(probe.address().port), ...args], {windowsHide:true});
      let stderr = ''; child.stderr.on('data', b => stderr += b); child.stdout.resume();
      const timer = setTimeout(() => child.kill('SIGKILL'), 10000);
      const code = await new Promise((resolve, reject) => { child.once('error', reject); child.once('close', resolve); });
      clearTimeout(timer); assert.notEqual(code, 0); assert.match(stderr, expected); assert.doesNotMatch(stderr, /cannot bind/);
      console.log(JSON.stringify({args, code, stderr}));
    }
  } finally { await new Promise(r => probe.close(r)); }
})().catch(e => {console.error(e); process.exitCode = 1;});
"#,
        &[&root],
    );
}
fn fixture() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/evidence/WP-24-P06/server-fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &root.join("testland"),
    );
    root
}
fn copy(a: &Path, b: &Path) {
    fs::create_dir_all(b).unwrap();
    for e in fs::read_dir(a).unwrap() {
        let p = e.unwrap().path();
        let t = b.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &t)
        } else {
            fs::copy(p, t).unwrap();
        }
    }
}
fn change(root: &Path, file: &str, old: &str, new: &str) {
    let p = root.join("testland").join(file);
    let s = fs::read_to_string(&p).unwrap();
    assert!(s.contains(old));
    fs::write(p, s.replace(old, new)).unwrap();
}
#[test]
fn p06_server_load_rejects_missing_dependency() {
    let root = fixture();
    change(
        &root,
        "manifest.toml",
        "depends = []",
        "depends = [{ id = 'missing', version = '*' }]",
    );
    let (_, s) = tokio::sync::watch::channel(false);
    let result = oh_server::Host::load(&root, s);
    assert!(
        result.is_err(),
        "authority was created for a missing dependency"
    );
    assert!(result.err().unwrap().contains("missing dependency"));
}
#[test]
fn p06_server_load_rejects_broken_ftl() {
    let root = fixture();
    fs::write(
        root.join("testland/localisation/en/pack.ftl"),
        "broken = {\n",
    )
    .unwrap();
    let (_, s) = tokio::sync::watch::channel(false);
    let result = oh_server::Host::load(&root, s);
    assert!(result.is_err(), "authority was created for invalid Fluent");
    assert!(result.err().unwrap().contains("invalid Fluent"));
}
#[test]
fn p06_server_restore_force_does_not_bypass_active_validation() {
    for (file, old, new, message) in [
        (
            "manifest.toml",
            "engine = \">=0.1, <0.2\"",
            "engine = \">=0.2\"",
            "incompatible",
        ),
        (
            "manifest.toml",
            "conflicts = []",
            "conflicts = ['testland']",
            "self conflicts",
        ),
        (
            "scenarios/m1/nations/NTH.toml",
            "capital = 10",
            "capital = 65535",
            "capital",
        ),
        (
            "scenarios/m1/scenario.toml",
            "map = \"testland\"",
            "unknown = \"testland\"",
            "unknown field",
        ),
        (
            "scenarios/m1/defines.toml",
            "max_message_bytes = 65536",
            "",
            "max_message_bytes",
        ),
    ] {
        let root = fixture();
        let c = oh_save::SaveContext::national(&root.join("testland"), "m1").unwrap();
        let bytes = oh_save::encode(&c.simulation(7).unwrap(), &c, 0, vec![]).unwrap();
        let save = root.join("old.ohsave");
        fs::write(&save, &bytes).unwrap();
        change(&root, file, old, new);
        for force in [false, true] {
            let (_, s) = tokio::sync::watch::channel(false);
            let r = oh_server::Host::load_with_save(&root, s, Some(&save), force);
            assert!(r.is_err());
            assert!(r.err().unwrap().contains(message));
        }
        assert_eq!(fs::read(save).unwrap(), bytes);
    }
}

#[test]
fn p06_frozen_v1_codec_remains_readable_but_is_not_an_active_localised_pack() {
    let root = fixture();
    let frozen = Path::new(env!("CARGO_MANIFEST_DIR")).join("../oh_save/tests/fixtures/m1-pack-v1");
    // Independent sibling preserves the original current fixture created above.
    let parent = root.join("frozen");
    copy(&frozen, &parent.join("testland"));
    assert!(oh_save::SaveContext::national(&parent.join("testland"), "m1").is_ok());
    let (_, s) = tokio::sync::watch::channel(false);
    let r = oh_server::Host::load(&parent, s);
    assert!(r.is_err());
    assert!(r.err().unwrap().contains("testland_name"));
}
