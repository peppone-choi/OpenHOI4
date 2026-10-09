//! Real native processes exercise the authoritative pack, command phase and V7.
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};
#[path = "support/military.rs"]
mod military;
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        Self {
            root: military::pack(),
        }
    }
    fn run(&self, inputs: &[Value]) -> Vec<Value> {
        let output = self.invoke(&["run", "--scenario", "m1", "--seed", "1"], inputs);
        parsed(output)
    }
    fn resume(&self, path: &Path, inputs: &[Value]) -> Vec<Value> {
        let output = self.invoke(&["resume", "--load", path.to_str().unwrap()], inputs);
        parsed(output)
    }
    fn invoke(&self, args: &[&str], inputs: &[Value]) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_oh_cli"))
            .arg("military")
            .args(args)
            .arg("--pack")
            .arg(&self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        for input in inputs {
            writeln!(stdin, "{}", serde_json::to_string(input).unwrap()).unwrap();
        }
        drop(stdin);
        child.wait_with_output().unwrap()
    }
    fn alter(&self, path: &str, old: &str, new: &str) {
        let path = self.root.join(path);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains(old));
        std::fs::write(path, text.replacen(old, new, 1)).unwrap();
    }
    fn checkpoint(&self, name: &str) -> PathBuf {
        self.root.parent().unwrap().join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.root.parent().unwrap()).unwrap();
    }
}
fn parsed(output: Output) -> Vec<Value> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
fn enqueue(tick: u64, sequence: u64, command: Value) -> Value {
    json!({"op":"enqueue","nation":1,"sequence":sequence.to_string(),"tick":tick.to_string(),"command":command})
}
fn step(count: u64) -> Value {
    json!({"op":"step","count":count})
}
fn train(template: &str) -> Value {
    json!({"type":"Train","template":template})
}
fn cancel(job: &str) -> Value {
    json!({"type":"Cancel","job":job})
}
fn deploy(job: &str, army: &str, province: u16, allow_understrength: bool) -> Value {
    json!({"type":"Deploy","job":job,"army":army,"province":province,"allow_understrength":allow_understrength})
}
fn jobs(v: &Value) -> &Vec<Value> {
    v["state"]["military"]["jobs"].as_array().unwrap()
}
fn nation(v: &Value, id: u16) -> &Value {
    v["state"]["economy"]["nations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["nation"] == id)
        .unwrap()
}
fn stock(v: &Value, id: u16) -> &Value {
    &v["state"]["production"]["nations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["nation"] == id)
        .unwrap()["stock"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["model"] == "test_model_1")
        .unwrap()["available"]
}
fn held(j: &Value) -> &Value {
    &j["equipment"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["model"] == "test_model_1")
        .unwrap()["count"]
}

#[test]
fn native_actual_pending_backfill_next_day_ready_and_explicit_deploy() {
    let f = Fixture::new();
    let out = f.run(&[
        enqueue(0, 1, train("large")),
        enqueue(0, 2, train("small")),
        step(24),
        step(24),
        step(24),
        enqueue(72, 3, deploy("1", "0", 10, false)),
        step(1),
        enqueue(73, 4, deploy("1", "0", 10, true)),
        step(1),
        enqueue(74, 5, cancel("1")),
        step(1),
    ]);
    assert_eq!(jobs(&out[2])[0]["status"], "Pending");
    assert_eq!(jobs(&out[2])[0]["reserved_manpower"], "0");
    assert_eq!(jobs(&out[2])[1]["status"], "Training");
    assert_eq!(jobs(&out[2])[1]["progress_days"], 0);
    assert_eq!(jobs(&out[2])[1]["start_tick"], "24");
    assert_eq!(jobs(&out[2])[1]["reserved_manpower"], "8");
    assert_eq!(held(&jobs(&out[2])[1]), "4");
    assert_eq!(nation(&out[2], 1)["reserved"], "18");
    assert_eq!(jobs(&out[3])[1]["progress_days"], 1);
    assert_eq!(jobs(&out[4])[1]["status"], "Ready");
    assert!(
        out[4]["state"]["military"]["divisions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(out[6]["commands"][0]["ok"], false);
    assert!(
        out[6]["commands"][0]["error"]
            .as_str()
            .unwrap()
            .contains("Understrength")
    );
    assert_eq!(jobs(&out[6])[1]["status"], "Ready");
    assert_eq!(out[8]["commands"][0]["ok"], true);
    assert_eq!(jobs(&out[8])[1]["status"], "Deployed");
    assert_eq!(nation(&out[8], 1)["reserved"], "10");
    assert_eq!(nation(&out[8], 1)["committed"], "68");
    assert_eq!(out[8]["state"]["military"]["divisions"][0]["manpower"], "8");
    assert_eq!(held(&out[8]["state"]["military"]["divisions"][0]), "4");
    assert_eq!(out[10]["commands"][0]["ok"], false);
    assert!(
        out[10]["commands"][0]["error"]
            .as_str()
            .unwrap()
            .contains("Terminal")
    );
    assert_eq!(nation(&out[10], 1)["committed"], "68");
}
#[test]
fn native_cancel_exact_owned_resources_before_daily_completion() {
    let f = Fixture::new();
    let out = f.run(&[
        enqueue(0, 1, train("small")),
        step(24),
        step(47),
        enqueue(71, 2, cancel("0")),
        step(1),
        enqueue(72, 3, cancel("0")),
        step(1),
    ]);
    assert_eq!(jobs(&out[2])[0]["progress_days"], 1);
    assert_eq!(jobs(&out[4])[0]["status"], "Cancelled");
    assert_eq!(jobs(&out[4])[0]["progress_days"], 0);
    assert_eq!(jobs(&out[4])[0]["reserved_manpower"], "0");
    assert_eq!(stock(&out[4], 1), "4");
    assert_eq!(nation(&out[4], 1)["reserved"], "10");
    assert_eq!(nation(&out[4], 1)["available"], "55");
    assert_eq!(out[6]["commands"][0]["ok"], false);
    assert_eq!(stock(&out[6], 1), "4");
}
#[test]
fn native_pending_cancel_zero_and_paused_commands_no_progress() {
    let f = Fixture::new();
    let out=f.run(&[enqueue(0,1,train("large")),step(1),json!({"op":"enqueue_time","nation":1,"sequence":"2","tick":"1","command":{"type":"Pause","paused":true}}),step(1),enqueue(1,3,cancel("0")),step(5)]);
    assert_eq!(out[5]["advanced"], "0");
    assert_eq!(out[5]["state"]["state"]["tick"], "1");
    assert_eq!(jobs(&out[5])[0]["status"], "Cancelled");
    assert_eq!(nation(&out[5], 1)["committed"], "60");
    assert_eq!(nation(&out[5], 1)["reserved"], "10");
    assert_eq!(stock(&out[5], 1), "4");
}
#[test]
fn native_deploy_before_ready_midnight_rejects_and_zero_equipment_explicit_control() {
    let f = Fixture::new();
    f.alter(
        "common/production/synthetic.toml",
        "stock = { test_model_1 = 4",
        "stock = { test_model_1 = 0",
    );
    let out = f.run(&[
        enqueue(0, 1, train("small")),
        step(71),
        enqueue(71, 2, deploy("0", "0", 10, true)),
        step(1),
        enqueue(72, 3, deploy("0", "0", 20, true)),
        step(1),
        enqueue(73, 4, deploy("0", "1", 10, true)),
        enqueue(73, 5, deploy("0", "0", 50, true)),
        step(1),
        enqueue(74, 6, deploy("0", "0", 10, true)),
        step(1),
    ]);
    assert!(
        out[3]["commands"][0]["error"]
            .as_str()
            .unwrap()
            .contains("NotReady")
    );
    assert_eq!(jobs(&out[3])[0]["status"], "Ready");
    assert_eq!(out[5]["commands"][0]["ok"], false);
    assert_eq!(out[6]["ok"], false); // other nation's army rejected at enqueue.
    assert_eq!(out[8]["commands"][0]["ok"], false); // sea destination at apply.
    assert_eq!(out[10]["commands"][0]["ok"], true);
    let division = &out[10]["state"]["military"]["divisions"][0];
    assert_eq!(division["manpower"], "8");
    assert!(
        division["equipment"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["count"] == "0")
    );
}
#[test]
fn native_flat_cross_army_hamilton_and_high_priority_before_training() {
    let f = Fixture::new();
    f.alter(
        "common/military/synthetic.toml",
        "committed = 60, reserved = 10",
        "committed = 50, reserved = 10",
    );
    f.alter("common/military/synthetic.toml","divisions = []","divisions = [{id=0,army=0,template=\"small\",province=10,manpower=8,equipment={}},{id=1,army=2,template=\"tiny\",province=10,manpower=2,equipment={}}]");
    let out = f.run(&[enqueue(0, 1, train("small")), step(24)]);
    assert_eq!(held(&out[1]["state"]["military"]["divisions"][0]), "3");
    assert_eq!(held(&out[1]["state"]["military"]["divisions"][1]), "1");
    assert_eq!(held(&jobs(&out[1])[0]), "0");
    let high = f.run(&[
        enqueue(0, 1, json!({"type":"SetPriority","army":"2","priority":1})),
        step(24),
    ]);
    assert_eq!(held(&high[1]["state"]["military"]["divisions"][0]), "4");
    assert_eq!(held(&high[1]["state"]["military"]["divisions"][1]), "0");
}
#[test]
fn native_actual_v7_fresh_resume_queue_and_deterministic_checkpoint_bytes() {
    let f = Fixture::new();
    let checkpoint = f.checkpoint("training.ohsave");
    let second = f.checkpoint("training-repeat.ohsave");
    let begin = [
        enqueue(0, 1, train("small")),
        step(24),
        enqueue(71, 2, deploy("0", "0", 10, true)),
        json!({"op":"save","path":checkpoint}),
        json!({"op":"query"}),
    ];
    let a = f.run(&begin);
    assert_eq!(a[3]["format_version"], 7);
    let resumed = f.resume(&checkpoint, &[json!({"op":"query"}), step(48)]);
    assert_eq!(a[4]["state"], resumed[0]["state"]);
    let full = f.run(&[
        enqueue(0, 1, train("small")),
        step(24),
        enqueue(71, 2, deploy("0", "0", 10, true)),
        step(48),
    ]);
    assert_eq!(resumed[1]["state"], full[3]["state"]);
    assert_eq!(resumed[1]["commands"][0]["ok"], false);
    let repeat = f.run(&[
        enqueue(0, 1, train("small")),
        step(24),
        enqueue(71, 2, deploy("0", "0", 10, true)),
        json!({"op":"save","path":second}),
    ]);
    assert_eq!(a[3]["state"], repeat[3]["state"]);
    assert_eq!(
        std::fs::read(checkpoint).unwrap(),
        std::fs::read(second).unwrap()
    );
}
#[test]
fn native_cap_shrink_preserves_holdings_cancel_releases_without_capacity_mint() {
    let f = Fixture::new();
    // Explicit synthetic law, not a new default rule or campaign balance.
    f.alter(
        "common/economy/synthetic.toml",
        "conscription_ratio = \"0.25\"",
        "conscription_ratio = \"0.06\"",
    );
    let out=f.run(&[enqueue(0,1,train("small")),step(48),json!({"op":"enqueue_economy","nation":1,"sequence":"2","tick":"48","command":{"type":"ChangeLaw","law":"war"}}),step(1),step(23),enqueue(72,3,cancel("0")),step(1)]);
    assert_eq!(nation(&out[3], 1)["capacity"], "60");
    assert_eq!(nation(&out[3], 1)["available"], "0");
    assert_eq!(jobs(&out[4])[0]["status"], "Ready");
    assert_eq!(nation(&out[6], 1)["reserved"], "10");
    assert_eq!(nation(&out[6], 1)["capacity"], "60");
    assert_eq!(nation(&out[6], 1)["available"], "0");
    assert_eq!(stock(&out[6], 1), "4");
}
#[test]
fn native_strict_grammar_and_input_valid_controls() {
    let args = [
        "military",
        "run",
        "--pack",
        "p",
        "--scenario",
        "m1",
        "--seed",
        "0",
    ]
    .map(str::to_owned)
    .to_vec();
    assert!(oh_cli::military::Options::parse(&args).is_ok());
    for flag in ["--force", "--definitions", "--ticks", "--days"] {
        let mut v = args.clone();
        v.extend([flag.into(), "1".into()]);
        assert!(oh_cli::military::Options::parse(&v).is_err());
    }
    for number in ["01", "+1", "-1", "18446744073709551616"] {
        let mut v = args.clone();
        v[7] = number.into();
        assert!(oh_cli::military::Options::parse(&v).is_err());
    }
    for text in [
        "{\"op\":\"step\",\"count\":1,\"unknown\":0}",
        "{\"op\":\"step\",\"count\":1.5}",
        "{\"op\":\"step\",\"count\":-1}",
        "{\"op\":\"step\",\"count\":1,\"count\":2}",
    ] {
        assert!(serde_json::from_str::<oh_cli::military::Input>(text).is_err());
    }
    let f = Fixture::new();
    let bad = f.invoke(
        &["run", "--scenario", "m1", "--seed", "1"],
        &[enqueue(0, 1, json!({"type":"Cancel","job":"01"}))],
    );
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("noncanonical military ID"));
    let good = f.run(&[json!({"op":"query"}), step(0)]);
    assert_eq!(good[0]["state"], good[1]["state"]);
    let excessive = f.invoke(
        &["run", "--scenario", "m1", "--seed", "1"],
        &[step(oh_cli::military::REQUEST_STEP_MAX + 1)],
    );
    assert!(!excessive.status.success());
    assert!(String::from_utf8_lossy(&excessive.stderr).contains("step request limit"));
}

#[test]
fn native_deploy_army_slots_checked_fresh_and_failed_job_keeps_resources() {
    let f = Fixture::new();
    let out = f.run(&[
        enqueue(0, 1, train("small")),
        enqueue(0, 2, train("small")),
        enqueue(0, 3, train("small")),
        step(72),
        enqueue(72, 4, deploy("0", "0", 10, true)),
        enqueue(72, 5, deploy("1", "0", 10, true)),
        enqueue(72, 6, deploy("2", "0", 10, true)),
        step(1),
        enqueue(73, 7, deploy("2", "2", 10, true)),
        step(1),
    ]);
    assert_eq!(out[7]["commands"][0]["ok"], true);
    assert_eq!(out[7]["commands"][1]["ok"], true);
    assert_eq!(out[7]["commands"][2]["ok"], false);
    assert!(
        out[7]["commands"][2]["error"]
            .as_str()
            .unwrap()
            .contains("ArmyFull")
    );
    assert_eq!(jobs(&out[7])[2]["status"], "Ready");
    assert_eq!(jobs(&out[7])[2]["reserved_manpower"], "8");
    assert_eq!(held(&jobs(&out[7])[2]), "1");
    assert_eq!(out[9]["commands"][0]["ok"], true);
    assert_eq!(
        out[9]["state"]["military"]["divisions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(nation(&out[9], 1)["reserved"], "10");
    assert_eq!(nation(&out[9], 1)["committed"], "84");
}

#[test]
fn native_production_joint_return_cap_rolls_back_whole_step_including_cancel() {
    let f = Fixture::new();
    f.alter(
        "defines.toml",
        "inventory_count_limit = 9223372036854775807",
        "inventory_count_limit = 4",
    );
    f.alter(
        "defines.toml",
        "initial_efficiency = 0.25",
        "initial_efficiency = 1.0",
    );
    f.alter(
        "defines.toml",
        "stability_output_low = 0.5",
        "stability_output_low = 1.0",
    );
    f.alter(
        "defines.toml",
        "stability_output_high = 1.5",
        "stability_output_high = 1.0",
    );
    let out = f.run(&[
        enqueue(0, 1, train("small")), step(24),
        json!({"op":"enqueue_production","nation":1,"sequence":"2","tick":"24","command":{"type":"Create","model":"test_model_1","requested_ic_bits":"65536"}}),
        step(23), json!({"op":"query"}), step(1), json!({"op":"query"}),
        enqueue(47, 3, cancel("0")), step(1),
    ]);
    assert_eq!(out[5]["ok"], false);
    assert_eq!(out[5]["advanced"], "0");
    assert_eq!(out[5]["state"], out[4]["state"]);
    assert_eq!(out[6]["state"], out[4]["state"]);
    assert_eq!(jobs(&out[6])[0]["status"], "Training");
    assert_eq!(held(&jobs(&out[6])[0]), "4");
    // Cancel itself returns all holdings, but production still cannot add past L.
    // The daily cap error atomically rolls back that same-step cancellation too.
    assert_eq!(out[8]["ok"], false);
    assert_eq!(jobs(&out[8])[0]["status"], "Training");
    assert_eq!(held(&jobs(&out[8])[0]), "4");
    assert_eq!(stock(&out[8], 1), "0");
    assert_eq!(
        out[8]["state"]["military"]["pending"][0]["command"]["type"],
        "Cancel"
    );
}
