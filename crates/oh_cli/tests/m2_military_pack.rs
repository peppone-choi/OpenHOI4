//! Real OS processes: actual ledger allocation, measured production, V7 and cancel tail.
#[path = "../../oh_data/tests/support/m2_military_pack.rs"]
mod fixture;
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
fn invoke(root: &Path, save: Option<&Path>, inputs: &[Value]) -> Vec<Value> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_oh_cli"));
    command.arg("military");
    if let Some(save) = save {
        command.args(["resume", "--load"]).arg(save);
    } else {
        command.args(["run", "--scenario", fixture::SCENARIO, "--seed", "1"]);
    }
    let mut child = command
        .arg("--pack")
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for input in inputs {
        writeln!(stdin, "{input}").unwrap();
    }
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(rows.len(), inputs.len());
    rows
}
fn nation(state: &Value, n: u16) -> &Value {
    state["economy"]["nations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["nation"] == n)
        .unwrap()
}
fn stock(state: &Value, n: u16) -> i64 {
    state["production"]["nations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["nation"] == n)
        .unwrap()["stock"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["model"] == "m2_equipment_1")
        .unwrap()["available"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}
fn step(count: u64) -> Value {
    json!({"op":"step","count":count})
}
fn creates(initial: &Value) -> Vec<Value> {
    (1..=6).map(|n|json!({"op":"enqueue_production","nation":n,"tick":"0","sequence":"1","command":{"type":"Create","model":"m2_equipment_1","requested_ic_bits":nation(initial,n)["ledger"]["allocation"][2]["bits"]}})).collect()
}
fn tail() -> Vec<Value> {
    let mut v = Vec::new();
    for n in 1..=6 {
        v.push(json!({"op":"enqueue","nation":n,"tick":"144","sequence":(100+n).to_string(),"command":{"type":"Cancel","job":(n-1).to_string()}}));
        v.push(step(1));
    }
    v
}
#[test]
fn fresh_process_measures_all_nations_then_v7_resume_equals_continuous_and_repeat() {
    let p = fixture::CopyPack::new();
    let initial = invoke(p.root(), None, &[json!({"op":"query"})]).remove(0)["state"].clone();
    assert_eq!(initial["military"]["jobs"], json!([]));
    for n in 1..=6 {
        assert_eq!(stock(&initial, n), 0);
    }
    let mut measured = creates(&initial);
    for _ in 0..365 {
        measured.push(step(24));
    }
    let output = invoke(p.root(), None, &measured);
    let mut days = [0_u64; 6];
    for (d, row) in output[6..].iter().enumerate() {
        assert_eq!(row["ok"], true);
        for n in 1..=6 {
            if days[usize::from(n - 1)] == 0 && stock(&row["state"], n) >= 6 {
                days[usize::from(n - 1)] = (d + 1) as u64;
            }
        }
    }
    assert_eq!(days, [5; 6]);
    let train_tick = days.iter().max().unwrap() * 24;
    assert_eq!(train_tick, 120);
    let mut begin = creates(&initial);
    begin.push(step(train_tick));
    for n in 1..=6 {
        begin.push(json!({"op":"enqueue","nation":n,"tick":train_tick.to_string(),"sequence":"2","command":{"type":"Train","template":"m2_small"}}));
    }
    begin.push(step(24));
    begin.push(json!({"op":"enqueue_time","nation":1,"tick":"144","sequence":"3","command":{"type":"Pause","paused":true}}));
    begin.push(step(1));
    let checkpoint = p.root().parent().unwrap().join("native-paused.ohsave");
    let repeat = p.root().parent().unwrap().join("native-repeat.ohsave");
    let final_save = p.root().parent().unwrap().join("native-tail.ohsave");
    let resumed_save = p.root().parent().unwrap().join("native-resumed.ohsave");
    let mut first = begin.clone();
    first.push(json!({"op":"save","path":checkpoint}));
    let a = invoke(p.root(), None, &first);
    let paused = &a.last().unwrap()["state"];
    assert_eq!(a.last().unwrap()["format_version"], 7);
    assert_eq!(paused["state"]["tick"], "144");
    assert_eq!(paused["state"]["paused"], true);
    for (i, j) in paused["military"]["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        assert_eq!(j["status"], "Training");
        assert_eq!(j["reserved_manpower"], "8");
        assert_eq!(j["equipment"][0]["count"], "6");
        assert_eq!(stock(paused, (i + 1) as u16), 2);
    }
    let original = std::fs::read(&checkpoint).unwrap();
    let mut again = begin.clone();
    again.push(json!({"op":"save","path":repeat}));
    let b = invoke(p.root(), None, &again);
    assert_eq!(a.last().unwrap()["state"], b.last().unwrap()["state"]);
    assert_eq!(original, std::fs::read(repeat).unwrap());
    let mut continuous = begin;
    continuous.extend(tail());
    continuous.push(json!({"op":"save","path":final_save}));
    let full = invoke(p.root(), None, &continuous);
    let mut restored = vec![json!({"op":"query"})];
    restored.extend(tail());
    restored.push(json!({"op":"save","path":resumed_save}));
    let resumed = invoke(p.root(), Some(&checkpoint), &restored);
    assert_eq!(resumed[0]["state"], *paused);
    assert_eq!(
        resumed.last().unwrap()["state"],
        full.last().unwrap()["state"]
    );
    assert_eq!(
        std::fs::read(final_save).unwrap(),
        std::fs::read(resumed_save).unwrap()
    );
    assert_eq!(std::fs::read(checkpoint).unwrap(), original);
    for n in 1..=6 {
        let after = &resumed[usize::from(n) * 2]["state"];
        let before = nation(paused, n);
        let e = nation(after, n);
        assert_eq!(e["reserved"], "0");
        assert_eq!(e["committed"], before["committed"]);
        assert_eq!(
            e["available"].as_str().unwrap().parse::<i64>().unwrap(),
            before["available"]
                .as_str()
                .unwrap()
                .parse::<i64>()
                .unwrap()
                + 8
        );
        assert_eq!(stock(after, n), 8);
        assert_eq!(after["production"]["lines"], paused["production"]["lines"]);
        let j = &after["military"]["jobs"][usize::from(n - 1)];
        assert_eq!(j["status"], "Cancelled");
        assert_eq!(j["equipment"], json!([]));
        assert_eq!(j["start_tick"], Value::Null);
        assert_eq!(j["progress_days"], 0);
        assert_eq!(after["military"]["next_job_id"], "6");
        assert_eq!(after["military"]["divisions"], json!([]));
        assert_eq!(after["military"]["pending"], json!([]));
    }
}
