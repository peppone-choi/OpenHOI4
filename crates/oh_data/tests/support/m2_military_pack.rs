#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
pub const SCENARIO: &str = "m2_military";
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland_m2_military")
}
pub struct CopyPack(PathBuf, PathBuf);
impl CopyPack {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "openhoi-m2-military-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(!p.exists());
        let child = p.join("pack");
        copy(&root(), &child);
        Self(child, p)
    }
    pub fn root(&self) -> &Path {
        &self.0
    }
    pub fn alter(&self, file: &str, from: &str, to: &str) {
        let p = self.0.join(file);
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains(from));
        std::fs::write(p, s.replacen(from, to, 1)).unwrap();
    }
}
pub fn copy(source: &Path, dest: &Path) {
    std::fs::create_dir_all(dest).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let p = entry.unwrap().path();
        let d = dest.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &d)
        } else {
            std::fs::copy(p, d).unwrap();
        }
    }
}
impl Drop for CopyPack {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.1).unwrap();
    }
}
