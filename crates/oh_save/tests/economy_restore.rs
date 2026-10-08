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
    assert_eq!(
        oh_save::inspect_header(&bytes, &oh_save::Limits::default())
            .unwrap()
            .packs,
        vec![context.pack().clone()],
        "original full context identity, before semantic assertion"
    );
    let before = oh_save::repro::report(&original).unwrap();
    for force in [false, true] {
        let error = match decode(&bytes, &context, force) {
            Err(error) => error,
            Ok(_) => panic!("accepted impossible current stage, force={force}"),
        };
        assert_eq!(error, "economy:TargetConflict");
        assert_eq!(oh_save::repro::report(&original).unwrap(), before);
        assert_eq!(
            std::fs::read(root.join("industry-level4.ohsave")).unwrap(),
            bytes
        );
    }
}

#[test]
fn minimum_target_is_semantic_error_in_matching_context_with_valid_control() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/repro/WP-14-M2-r3-target-range");
    let context = SaveContext::national(&root.join("packs/testland"), "m1").unwrap();
    let bad = std::fs::read(root.join("project-target-min.ohsave")).unwrap();
    let valid = std::fs::read(root.join("control-valid-target2.ohsave")).unwrap();
    assert_eq!(
        oh_save::inspect_header(&bad, &oh_save::Limits::default())
            .unwrap()
            .packs,
        vec![context.pack().clone()]
    );
    for force in [false, true] {
        let control = decode(&valid, &context, force).unwrap().simulation;
        assert_eq!(
            control
                .economy()
                .unwrap()
                .nation(oh_core::NationId(1))
                .unwrap()
                .projects()[0]
                .target,
            2
        );
        let error = match decode(&bad, &context, force) {
            Err(error) => error,
            Ok(_) => panic!("accepted MIN target"),
        };
        assert_eq!(error, "economy:InvalidValue");
    }
}
