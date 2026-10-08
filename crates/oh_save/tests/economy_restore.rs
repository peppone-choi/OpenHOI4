use oh_save::{SaveContext, decode};
use std::path::Path;

#[test]
fn req_eco_06_req_sav_02_original_p05_stage_overflow_is_rejected_normal_and_force() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/repro/WP-14-M2-r3-restore-stage");
    let context = SaveContext::national(&root.join("packs/testland"), "m1").unwrap();
    let bytes = std::fs::read(root.join("industry-level4.ohsave")).unwrap();
    assert_eq!(bytes.len(), 458);
    let original = context.simulation(1000).unwrap();
    let before = oh_save::repro::report(&original).unwrap();
    for force in [false, true] {
        assert!(
            decode(&bytes, &context, force).is_err(),
            "accepted impossible current stage, force={force}"
        );
        assert_eq!(oh_save::repro::report(&original).unwrap(), before);
        assert_eq!(
            std::fs::read(root.join("industry-level4.ohsave")).unwrap(),
            bytes
        );
    }
}
