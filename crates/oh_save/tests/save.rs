use oh_save::{SaveContext, decode, encode};

fn context() -> SaveContext {
    SaveContext::national(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap()
}

#[test]
fn req_sav_01_complete_m1_roundtrip() {
    let context = context();
    let sim = context.simulation(7).unwrap();
    let bytes = encode(&sim, &context, 0, vec![]).unwrap();
    assert_eq!(&bytes[..4], b"OHSV");
    let restored = decode(&bytes, &context, false).unwrap();
    assert_eq!(
        restored.simulation.state_hash().unwrap(),
        sim.state_hash().unwrap()
    );
    assert!(restored.simulation.world().is_some());
}

#[test]
fn req_sav_02_m1_continue_after_restore() {
    let context = context();
    let mut sim = context.simulation(7).unwrap();
    for _ in 0..48 {
        sim.step().unwrap();
    }
    let bytes = encode(&sim, &context, 0, vec![]).unwrap();
    let mut restored = decode(&bytes, &context, false).unwrap().simulation;
    for _ in 0..48 {
        sim.step().unwrap();
        restored.step().unwrap();
    }
    assert_eq!(sim.state_hash().unwrap(), restored.state_hash().unwrap());
    assert_eq!(
        restored.world().unwrap().inputs().states()[0]
            .ledger()
            .tick(),
        96
    );
}
