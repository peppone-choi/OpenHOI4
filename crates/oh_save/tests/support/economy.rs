use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
static SERIAL: AtomicU64 = AtomicU64::new(0);
pub fn pack() -> PathBuf {
    fn copy(source: &Path, dest: &Path) {
        std::fs::create_dir_all(dest).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let p = entry.unwrap().path();
            let d = dest.join(p.file_name().unwrap());
            if p.is_dir() {
                copy(&p, &d);
            } else {
                std::fs::copy(p, d).unwrap();
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/evidence/WP-14-M2-r2/save-fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::SeqCst)
        ));
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        &root,
    );
    std::fs::create_dir_all(root.join("common/economy")).unwrap();
    std::fs::write(
        root.join("common/economy/synthetic.toml"),
        include_str!("../../../oh_data/tests/fixtures/economy/valid.toml"),
    )
    .unwrap();
    let scenario = root.join("scenarios/m1/scenario.toml");
    let original = std::fs::read_to_string(&scenario).unwrap();
    std::fs::write(scenario, format!("economy = \"synthetic\"\n{original}")).unwrap();
    root
}
