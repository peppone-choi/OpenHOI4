#[path = "support/economy.rs"]
mod fixture;
use oh_core::NationId;
use oh_save::{SaveContext, repro};
use oh_sim::{Command, economy::Action};
#[test]
fn req_eco_nat_actual_v5_codec_bounds_and_replay_full_state() {
    let root = fixture::pack();
    let c = SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1000).unwrap();
    s.enqueue(
        0,
        NationId(1),
        1,
        Command::Economy(Action::Construct {
            project: 9,
            state: 1,
            building: "industry".into(),
        }),
    )
    .unwrap();
    for _ in 0..24 {
        s.step().unwrap();
    }
    s.enqueue(
        60,
        NationId(1),
        2,
        Command::Economy(Action::Cancel { project: 9 }),
    )
    .unwrap();
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    assert_eq!(
        oh_save::inspect_header(&bytes, &oh_save::Limits::default())
            .unwrap()
            .format_version,
        5
    );
    let restored = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(
        repro::report(&s).unwrap(),
        repro::report(&restored).unwrap()
    );
    let mut record = repro::Recorder::new(&s, &c).unwrap();
    for _ in 0..48 {
        record.input(&mut s, repro::Input::Step {}).unwrap();
    }
    let bundle = record.finish(&s, &c).unwrap();
    let replayed = repro::replay(&bundle, &c).unwrap();
    assert_eq!(
        repro::report(&s).unwrap(),
        repro::report(&replayed).unwrap()
    );
    let out = root.parent().unwrap().join(format!(
        "out-{}",
        root.file_name().unwrap().to_str().unwrap()
    ));
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("pack-path.txt"), root.to_string_lossy().as_bytes()).unwrap();
    std::fs::write(out.join("v5.ohsave"), bytes).unwrap();
    std::fs::write(out.join("v5.zip"), bundle).unwrap();
    std::fs::write(
        out.join("expected.json"),
        serde_json::to_vec_pretty(&repro::report(&s).unwrap()).unwrap(),
    )
    .unwrap();
}
#[test]
fn req_eco_nat_v5_force_preserves_mode_and_host_identity() {
    let root = fixture::pack();
    let c = SaveContext::national(&root, "m1").unwrap();
    let s = c.simulation(7).unwrap();
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    let oldroot =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let old = SaveContext::national(&oldroot, "m1").unwrap();
    for force in [false, true] {
        assert!(
            oh_save::decode(&bytes, &old, force)
                .unwrap_err_string()
                .contains("EconomyModeMismatch")
        );
    }
    let oldbytes = oh_save::encode(&old.simulation(7).unwrap(), &old, 0, vec![1]).unwrap();
    for force in [false, true] {
        assert!(oh_save::decode(&oldbytes, &c, force).is_err());
    }
}
trait ErrorText {
    fn unwrap_err_string(self) -> String;
}
impl ErrorText for Result<oh_save::LoadedSave, String> {
    fn unwrap_err_string(self) -> String {
        match self {
            Err(e) => e,
            Ok(_) => panic!("accepted invalid save"),
        }
    }
}
#[test]
fn req_eco_01_actual_score_some_v5_file_codec_is_bounded_and_roundtrips() {
    let root = fixture::pack();
    let path = root.join("scenarios/m1/scenario.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path,format!("end_root = \"NTH\"\nend_conditions = {{ stability = {{ gte = \"0.5\" }} }}\nscore_weights = {{ victory_points = \"0\", industrial_capacity = \"1.5\", survival = \"0\", faction_victory = \"0\" }}\n{text}")).unwrap();
    let c = SaveContext::national(&root, "m1").unwrap();
    let s = c.simulation(7).unwrap();
    assert!(s.is_ended());
    assert_eq!(
        s.economy().unwrap().industrial_scores().unwrap()[&1].term,
        oh_core::Qty::from_num(15)
    );
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    let r = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(repro::report(&s).unwrap(), repro::report(&r).unwrap());
    let limits = oh_save::Limits {
        queue_max_entries: 0,
        ..oh_save::Limits::default()
    };
    assert!(oh_save::encode_with_limits(&s, &c, 0, vec![1], &limits).is_ok());
}
