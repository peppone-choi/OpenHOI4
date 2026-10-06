use std::path::Path;
pub fn copy_pack(target: &Path) {
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
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        target,
    );
}
