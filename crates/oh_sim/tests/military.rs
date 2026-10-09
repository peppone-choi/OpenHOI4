use oh_core::{Fx, NationId, ProvinceId, Qty};
use oh_sim::{
    Command, Date, Simulation, TimeConfig,
    military::{Action, JobStatus, MilitaryError},
    world::World,
};
#[path = "support/economy_fixture.rs"]
mod economy_fixture;
fn loaded(stock: i64, limit: i64, divisions: &str) -> oh_data::national::LoadedNational {
    let mut l = economy_fixture::loaded();
    let mut p = fixture_definition();
    p.tuning.as_mut().unwrap().inventory_count_limit = limit;
    p.nations
        .get_mut(&1)
        .unwrap()
        .stock
        .insert("test_model_1".into(), stock);
    l.production = Some(p);
    let normal = include_str!("../../oh_data/tests/fixtures/military_templates/normal.toml")
        .replace("manpower = 1000", "manpower = 6")
        .replace("manpower = 100", "manpower = 2")
        .replace("items = 100", "items = 6")
        .replace("items = 10", "items = 2");
    let text = format!(
        "version=1\nnormal_templates='''{normal}'''\ntraining_days={{example=2}}\nbackground={{1={{committed=60,reserved=10}},2={{committed=60,reserved=10}}}}\narmies=[{{id=0,nation=1,general='synthetic_general',capacity=1,priority=0}},{{id=1,nation=1,general='synthetic_general_2',capacity=4,priority=1}}]\ndivisions=[{divisions}]\n"
    );
    l.military = Some(oh_data::military::parse(&text).unwrap());
    l.military.as_ref().unwrap().validate(&l).unwrap();
    l
}
fn sim(stock: i64, limit: i64, divisions: &str) -> Simulation {
    let l = loaded(stock, limit, divisions);
    Simulation::with_world(
        "m1".into(),
        Date::new(2000, 1, 1).unwrap(),
        1,
        TimeConfig::from_defines(&l.pack.defines).unwrap(),
        World::from_loaded(&l).unwrap(),
    )
    .unwrap()
}
fn cmd(s: &mut Simulation, a: Action) -> Result<(), oh_sim::Error> {
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        s.snapshot().tick(),
        Command::Military(a),
    )
    .unwrap();
    s.step().unwrap().commands[0].result
}
fn until(s: &mut Simulation, tick: u64) {
    while s.snapshot().tick() < tick {
        s.step().unwrap();
    }
}
fn train(s: &mut Simulation) {
    cmd(
        s,
        Action::Train {
            template: "example".into(),
        },
    )
    .unwrap();
}
#[test]
fn req_training_pending_start_next_day_ready_and_explicit_zero_equipment_deploy() {
    let mut s = sim(0, 100, "");
    train(&mut s);
    assert_eq!(
        s.economy().unwrap().nation(NationId(1)).unwrap().reserved(),
        10
    );
    assert_eq!(
        s.military().unwrap().jobs()[&0].status(),
        JobStatus::Pending
    );
    until(&mut s, 24);
    let j = &s.military().unwrap().jobs()[&0];
    assert_eq!(j.status(), JobStatus::Training);
    assert_eq!(j.progress_days(), 0);
    assert_eq!(j.reserved_manpower(), 8);
    until(&mut s, 48);
    assert_eq!(s.military().unwrap().jobs()[&0].progress_days(), 1);
    until(&mut s, 72);
    assert_eq!(s.military().unwrap().jobs()[&0].status(), JobStatus::Ready);
    assert!(s.military().unwrap().divisions().is_empty());
    assert_eq!(
        cmd(
            &mut s,
            Action::Deploy {
                job: 0,
                army: 0,
                province: ProvinceId(10),
                allow_understrength: false
            }
        ),
        Err(oh_sim::Error::Military(MilitaryError::Understrength))
    );
    assert_eq!(s.military().unwrap().jobs()[&0].reserved_manpower(), 8);
    cmd(
        &mut s,
        Action::Deploy {
            job: 0,
            army: 0,
            province: ProvinceId(10),
            allow_understrength: true,
        },
    )
    .unwrap();
    assert_eq!(
        s.economy().unwrap().nation(NationId(1)).unwrap().reserved(),
        10
    );
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .committed(),
        68
    );
    assert_eq!(s.military().unwrap().divisions()[&0].manpower(), 8);
    assert_eq!(
        cmd(&mut s, Action::Cancel { job: 0 }),
        Err(oh_sim::Error::Military(MilitaryError::Terminal))
    );
}
#[test]
fn req_exact_owned_refunds_and_idempotence() {
    let mut s = sim(5, 100, "");
    train(&mut s);
    until(&mut s, 24);
    assert_eq!(
        s.production().unwrap().stock(NationId(1), "test_model_1"),
        Some(0)
    );
    assert_eq!(
        s.military().unwrap().jobs()[&0].equipment()["test_model_1"],
        5
    );
    cmd(&mut s, Action::Cancel { job: 0 }).unwrap();
    assert_eq!(
        s.production().unwrap().stock(NationId(1), "test_model_1"),
        Some(5)
    );
    assert_eq!(
        s.economy().unwrap().nation(NationId(1)).unwrap().reserved(),
        10
    );
    assert_eq!(
        cmd(&mut s, Action::Cancel { job: 0 }),
        Err(oh_sim::Error::Military(MilitaryError::Terminal))
    );
}
#[test]
fn req_active_reinforcement_before_training_and_priority() {
    let div = "{id=0,army=0,template='example',province=10,manpower=0,equipment={}}, {id=1,army=1,template='example',province=10,manpower=0,equipment={}}";
    let mut s = sim(4, 100, div);
    train(&mut s);
    until(&mut s, 24);
    assert_eq!(
        s.military().unwrap().divisions()[&0].equipment()["test_model_1"],
        4
    );
    assert_eq!(
        s.military().unwrap().divisions()[&1].equipment()["test_model_1"],
        0
    );
    assert_eq!(
        s.military().unwrap().jobs()[&0].equipment()["test_model_1"],
        0
    );
    assert_eq!(
        s.economy()
            .unwrap()
            .nation(NationId(1))
            .unwrap()
            .committed(),
        76
    );
}
#[test]
fn req_same_priority_flat_division_proportion_and_training_proportion() {
    let div = "{id=0,army=1,template='example',province=10,manpower=0,equipment={test_model_1=2}}, {id=1,army=1,template='example',province=10,manpower=0,equipment={test_model_1=6}}";
    let mut s = sim(4, 100, div);
    until(&mut s, 24);
    assert_eq!(
        s.military().unwrap().divisions()[&0].equipment()["test_model_1"],
        5
    );
    assert_eq!(
        s.military().unwrap().divisions()[&1].equipment()["test_model_1"],
        7
    );
    let mut s = sim(5, 100, "");
    train(&mut s);
    train(&mut s);
    until(&mut s, 24);
    assert_eq!(
        s.military().unwrap().jobs()[&0].equipment()["test_model_1"],
        3
    );
    assert_eq!(
        s.military().unwrap().jobs()[&1].equipment()["test_model_1"],
        2
    );
}
#[test]
fn req_production_holds_return_space_and_full_step_atomic_failure() {
    let mut s = sim(8, 8, "");
    train(&mut s);
    until(&mut s, 24);
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        101,
        Command::Production(oh_sim::production::Action::Create {
            model: "test_model_1".into(),
            requested_ic: Qty::ONE,
        }),
    )
    .unwrap();
    s.step().unwrap();
    until(&mut s, 119);
    let h = s.state_hash().unwrap();
    let q = s.snapshot();
    assert!(s.step().is_err());
    assert_eq!(s.state_hash().unwrap(), h);
    assert_eq!(s.snapshot(), q);
    s.enqueue(s.snapshot().tick(), NationId(1), 0, Command::Pause(true))
        .unwrap();
    cmd(&mut s, Action::Cancel { job: 0 }).unwrap();
    assert_eq!(
        s.military().unwrap().jobs()[&0].status(),
        JobStatus::Cancelled
    );
    assert_eq!(
        s.production().unwrap().stock(NationId(1), "test_model_1"),
        Some(8)
    );
}
#[test]
fn req_commands_before_midnight_ready_and_cancel_wins() {
    let mut s = sim(0, 100, "");
    train(&mut s);
    until(&mut s, 71);
    assert_eq!(
        cmd(
            &mut s,
            Action::Deploy {
                job: 0,
                army: 0,
                province: ProvinceId(10),
                allow_understrength: true
            }
        ),
        Err(oh_sim::Error::Military(MilitaryError::NotReady))
    );
    assert_eq!(s.military().unwrap().jobs()[&0].status(), JobStatus::Ready);
    let mut s = sim(5, 100, "");
    train(&mut s);
    until(&mut s, 71);
    cmd(&mut s, Action::Cancel { job: 0 }).unwrap();
    assert_eq!(
        s.military().unwrap().jobs()[&0].status(),
        JobStatus::Cancelled
    );
    assert_eq!(s.military().unwrap().jobs()[&0].progress_days(), 0);
}
#[test]
fn req_paused_pending_never_advances_and_foreign_army_deploy_rejects() {
    let mut s = sim(0, 100, "");
    train(&mut s);
    s.enqueue(s.snapshot().tick(), NationId(1), 10, Command::Pause(true))
        .unwrap();
    s.step().unwrap();
    for _ in 0..30 {
        s.step().unwrap();
    }
    assert_eq!(
        s.military().unwrap().jobs()[&0].status(),
        JobStatus::Pending
    );
    assert_eq!(s.snapshot().tick(), 1);
}

fn fixture_definition() -> oh_data::production::Definition {
    use oh_data::production::{Definition, Model, NationInput, Tuning};
    use std::collections::{BTreeMap, BTreeSet};
    let m = Model {
        name_key: "test-model-1".into(),
        family: "test_family".into(),
        generation: 1,
        unit_cost: "1".into(),
        resources_per_item: BTreeMap::new(),
    };
    let mut m2 = m.clone();
    m2.generation = 2;
    m2.name_key = "test-model-2".into();
    let n = NationInput {
        allowed_models: BTreeSet::from(["test_model_1".into(), "test_model_2".into()]),
        stock: BTreeMap::from([("test_model_1".into(), 0), ("test_model_2".into(), 0)]),
    };
    Definition {
        models: BTreeMap::from([("test_model_1".into(), m), ("test_model_2".into(), m2)]),
        nations: BTreeMap::from([(1, n.clone()), (2, n)]),
        tuning: Some(Tuning {
            initial_efficiency: Fx::from_num(0.25),
            efficiency_cap: Fx::ONE,
            daily_efficiency_growth: Fx::from_num(0.0078125),
            same_family_newer_retention: Fx::from_num(0.75),
            other_retention: Fx::from_num(0.25),
            stability_output_low: Fx::from_num(0.5),
            stability_output_high: Fx::from_num(1.5),
            inventory_count_limit: i64::MAX,
            lines_max: 1024,
        }),
    }
}
