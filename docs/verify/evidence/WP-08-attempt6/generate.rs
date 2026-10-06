fn main() {
    let root = std::path::Path::new("E:/openhoi/.orchestrator/wt/WP-08-verify6/target/wp08-verify6/generated-proto");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("protocol.ts"), oh_proto::typescript()).unwrap();
}
