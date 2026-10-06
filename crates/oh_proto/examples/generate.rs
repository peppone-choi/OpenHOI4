fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client/src/proto");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("protocol.ts"), oh_proto::typescript()).unwrap();
}
