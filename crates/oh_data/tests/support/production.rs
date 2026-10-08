//! Generated test input; original M1 files are copied, never rewritten in place.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
static SERIAL: AtomicU64 = AtomicU64::new(0);
pub fn pack() -> PathBuf {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = repo
        .join("target/wp15-fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::SeqCst)
        ))
        .join("testland");
    pub fn copy(source: &Path, dest: &Path) {
        std::fs::create_dir_all(dest).unwrap();
        for e in std::fs::read_dir(source).unwrap() {
            let p = e.unwrap().path();
            let d = dest.join(p.file_name().unwrap());
            if p.is_dir() {
                copy(&p, &d)
            } else {
                std::fs::copy(p, d).unwrap();
            }
        }
    }
    copy(&repo.join("data/packs/testland"), &root);
    std::fs::create_dir_all(root.join("common/economy")).unwrap();
    std::fs::create_dir_all(root.join("common/production")).unwrap();
    std::fs::write(
        root.join("common/economy/synthetic.toml"),
        include_str!("../fixtures/economy/valid.toml"),
    )
    .unwrap();
    std::fs::write(
        root.join("common/production/synthetic.toml"),
        include_str!("../fixtures/production/valid.toml"),
    )
    .unwrap();
    for (path, append) in [
        (
            "scenarios/m1/scenario.toml",
            "production = \"synthetic\"\neconomy = \"synthetic\"\n",
        ),
        (
            "defines.toml",
            include_str!("../fixtures/production/defines.toml"),
        ),
        (
            "localisation/en/national.ftl",
            include_str!("../fixtures/production/en.ftl"),
        ),
        (
            "localisation/ko/national.ftl",
            include_str!("../fixtures/production/ko.ftl"),
        ),
    ] {
        let p = root.join(path);
        let old = std::fs::read_to_string(&p).unwrap();
        let text = if path.ends_with("scenario.toml") {
            format!("{append}{old}")
        } else {
            format!("{old}\n{append}")
        };
        std::fs::write(p, text).unwrap();
    }
    root
}
