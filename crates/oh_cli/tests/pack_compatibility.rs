use std::path::Path;
#[test]
fn req_mod_02_pack_identity_matches_save_and_new_current_save_roundtrips() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let hash = oh_data::pack_validation::content_hash(&root).unwrap();
    assert_eq!(hash, oh_save::pack_hash(&root).unwrap());
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut sim = c.simulation(7).unwrap();
    oh_cli::advance(&mut sim, 49).unwrap();
    let bytes = oh_save::encode(&sim, &c, 0, vec![]).unwrap();
    let restored = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(sim.export_save().unwrap(), restored.export_save().unwrap());
    assert_eq!(sim.state_hash().unwrap(), restored.state_hash().unwrap());
    let old = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../oh_save/tests/fixtures/m1-v1.ohsave"),
    )
    .unwrap();
    let error = oh_save::decode(&old, &c, false)
        .err()
        .expect("changed pack identity must fail");
    assert!(error.contains("PackMismatch"), "{error}");
}
