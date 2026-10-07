use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
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
