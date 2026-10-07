//! Historical source97f pack only. No live-pack fallback or future fixture assumption.
use std::path::{Path, PathBuf};
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/movement-pack-97f")
}
pub fn context() -> oh_save::SaveContext {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/movement-pack-97f.source.json")).unwrap();
    assert_eq!(
        manifest["source"],
        "97f688718dabed46bc477fa249798ac09e999581"
    );
    assert_eq!(manifest["count"], 19);
    let source = root();
    for record in manifest["files"].as_array().unwrap() {
        let path = source.join(record["path"].as_str().unwrap());
        assert_eq!(
            std::fs::read(&path).unwrap().len() as u64,
            record["bytes"].as_u64().unwrap()
        );
    }
    let c = oh_save::SaveContext::national(&source, "m1").unwrap();
    assert_eq!(c.pack().content_hash, 13318001328374612931);
    assert_eq!(
        c.pack().content_hash,
        manifest["pack_hash"].as_u64().unwrap()
    );
    c
}
