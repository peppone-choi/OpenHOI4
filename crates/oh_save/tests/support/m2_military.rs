#![allow(dead_code)]
use oh_core::NationId;
use oh_sim::{
    Command, Simulation,
    military::{Action, JobStatus},
    production,
};
use serde_json::{Value, json};
pub fn stock(s: &Simulation, n: u16) -> i64 {
    s.production()
        .unwrap()
        .stock(NationId(n), "m2_equipment_1")
        .unwrap()
}
pub fn checkpoint(c: &oh_save::SaveContext) -> (Simulation, Value) {
    let mut s = c.simulation(1).unwrap();
    assert!(s.military().unwrap().jobs().is_empty());
    assert_eq!(s.military().unwrap().next_job_id(), 0);
    assert_eq!(s.military().unwrap().next_division_id(), 0);
    assert!(s.military().unwrap().divisions().is_empty());
    for n in 1..=6 {
        let ic = s
            .economy()
            .unwrap()
            .nation(NationId(n))
            .unwrap()
            .ledger()
            .allocation[2];
        s.enqueue(
            0,
            NationId(n),
            1,
            Command::Production(production::Action::Create {
                model: "m2_equipment_1".into(),
                requested_ic: ic,
            }),
        )
        .unwrap();
    }
    let mut measured = [0_u64; 6];
    for day in 1..=365 {
        for _ in 0..24 {
            let result = s.step().unwrap();
            assert!(result.commands.iter().all(|r| r.result.is_ok()));
        }
        for n in 1..=6 {
            if measured[usize::from(n - 1)] == 0 && stock(&s, n) >= 6 {
                measured[usize::from(n - 1)] = day;
            }
        }
        if measured.iter().all(|d| *d > 0) {
            break;
        }
    }
    assert!(
        measured.iter().all(|d| *d > 0),
        "stock threshold not reached in365days: {measured:?}"
    );
    let t = s.snapshot().tick();
    for n in 1..=6 {
        s.enqueue(
            t,
            NationId(n),
            2,
            Command::Military(Action::Train {
                template: "m2_small".into(),
            }),
        )
        .unwrap();
    }
    for _ in 0..24 {
        let result = s.step().unwrap();
        assert!(result.commands.iter().all(|r| r.result.is_ok()));
    }
    assert!(
        s.military()
            .unwrap()
            .jobs()
            .values()
            .all(|j| j.status() == JobStatus::Training
                && j.reserved_manpower() == 8
                && j.equipment().values().sum::<i64>() > 0)
    );
    let tick = s.snapshot().tick();
    s.enqueue(tick, NationId(1), 3, Command::Pause(true))
        .unwrap();
    s.step().unwrap();
    assert!(s.snapshot().paused());
    let rows=s.military().unwrap().jobs().values().map(|j|{let n=j.nation().0;let e=s.economy().unwrap().nation(NationId(n)).unwrap();json!({"nation":n,"job":j.id().to_string(),"held":j.equipment()["m2_equipment_1"].to_string(),"stock":stock(&s,n).to_string(),"reserved":e.reserved().to_string(),"available":e.available().to_string(),"committed":e.committed().to_string()})}).collect::<Vec<_>>();
    let summary = json!({"days":measured,"train_enqueued_tick":t.to_string(),"tick":s.snapshot().tick().to_string(),"paused_hash":format!("{:016x}",s.state_hash().unwrap()),"nations":rows});
    println!("OBSERVATIONS {summary}");
    (s, summary)
}
pub fn cancel_tail(s: &mut Simulation) {
    for n in 1..=6 {
        let job = u64::from(n - 1);
        s.enqueue(
            s.snapshot().tick(),
            NationId(n),
            100 + u64::from(n),
            Command::Military(Action::Cancel { job }),
        )
        .unwrap();
        let result = s.step().unwrap();
        assert!(result.commands[0].result.is_ok());
    }
}
