use std::path::Path;
pub fn copy_pack(target: &Path) {
    copy_from(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        target,
    );
}
/// Explicit frozen input for committed v1 fixture construction only.
#[allow(dead_code)] // Used by save_fixture; generic test targets use current packs.
pub fn copy_v1_pack(target: &Path) {
    copy_from(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/m1-pack-v1"),
        target,
    );
}
fn copy_from(source: &Path, target: &Path) {
    fn copy(source: &Path, target: &Path) {
        std::fs::create_dir_all(target).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let destination = target.join(entry.file_name());
            if path.is_dir() {
                copy(&path, &destination);
            } else {
                std::fs::copy(path, destination).unwrap();
            }
        }
    }
    copy(source, target);
}
