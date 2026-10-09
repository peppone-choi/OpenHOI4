//! Actual paused ready authority for separate-process WebSocket acceptance.
#[path = "../tests/support/military.rs"]
mod fixture;
fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).expect("output directory"));
    std::fs::create_dir_all(&out).unwrap();
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    s.enqueue(
        0,
        oh_core::NationId(1),
        1,
        oh_sim::Command::Military(oh_sim::military::Action::Train {
            template: "example".into(),
        }),
    )
    .unwrap();
    for _ in 0..72 {
        s.step().unwrap();
    }
    s.enqueue(72, oh_core::NationId(1), 2, oh_sim::Command::Pause(true))
        .unwrap();
    s.step().unwrap();
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    std::fs::write(out.join("ready.ohsave"), bytes).unwrap();
    std::fs::write(out.join("expected.json"),serde_json::to_vec_pretty(&serde_json::json!({"pack":root,"tick":s.snapshot().tick().to_string(),"hash":format!("{:016x}",s.state_hash().unwrap()),"format_version":7})).unwrap()).unwrap();
}
