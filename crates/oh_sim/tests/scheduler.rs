use oh_core::NationId;
use oh_sim::{Command, Date, Phase, Simulation, TimeConfig};

fn sim(date: Date) -> Simulation {
    Simulation::new(
        "testland".into(),
        date,
        1,
        TimeConfig::new([500, 200, 80, 25, 0], 1).unwrap(),
    )
    .unwrap()
}

#[test]
fn req_gen_01_headless_state_and_hash_are_repeatable() {
    let mut a = sim(Date::new(2000, 1, 1).unwrap());
    let mut b = sim(Date::new(2000, 1, 1).unwrap());
    let initial = a.state_hash().unwrap();
    for _ in 0..1000 {
        a.step().unwrap();
        b.step().unwrap();
    }
    assert_eq!(a.snapshot().tick(), 1000);
    assert_eq!(a.snapshot().date(), Date::new(2000, 2, 11).unwrap());
    assert_eq!(a.snapshot().hour(), 16);
    assert_eq!(a.state_hash().unwrap(), b.state_hash().unwrap());
    assert_ne!(initial, a.state_hash().unwrap());
}

#[test]
fn req_gen_05_commands_are_sorted_and_future_commands_wait() {
    let mut a = sim(Date::new(2000, 1, 1).unwrap());
    let mut b = sim(Date::new(2000, 1, 1).unwrap());
    let commands = [
        (0, NationId(2), 1, Command::SetSpeed(5)),
        (0, NationId(1), 3, Command::SetSpeed(2)),
        (0, NationId(1), 2, Command::SetSpeed(4)),
        (1, NationId(0), 4, Command::Pause(true)),
    ];
    for (tick, nation, seq, command) in commands.clone() {
        a.enqueue(tick, nation, seq, command).unwrap();
    }
    for (tick, nation, seq, command) in commands.into_iter().rev() {
        b.enqueue(tick, nation, seq, command).unwrap();
    }
    assert_eq!(a.state_hash().unwrap(), b.state_hash().unwrap());
    let step = a.step().unwrap();
    assert_eq!(
        step.commands
            .iter()
            .map(|c| (c.nation, c.sequence))
            .collect::<Vec<_>>(),
        vec![(NationId(1), 2), (NationId(1), 3), (NationId(2), 1)]
    );
    assert_eq!(a.snapshot().speed(), 5);
    assert_eq!(a.snapshot().tick(), 1);
    assert!(!a.snapshot().paused());
    assert!(!a.step().unwrap().advanced);
    assert!(a.snapshot().paused());
    assert!(a.enqueue(0, NationId(0), 5, Command::Pause(false)).is_err());
    b.step().unwrap();
    b.step().unwrap();
    assert_eq!(a.state_hash().unwrap(), b.state_hash().unwrap());
}

#[test]
fn req_time_01_pause_resume_and_five_data_driven_speeds() {
    let mut a = sim(Date::new(2000, 1, 1).unwrap());
    a.enqueue(0, NationId(0), 1, Command::Pause(true)).unwrap();
    let paused = a.step().unwrap();
    assert!(!paused.advanced);
    assert_eq!(paused.phases, vec![Phase::Commands, Phase::Snapshot]);
    assert_eq!(a.snapshot().tick(), 0);
    assert_eq!(a.snapshot().hour(), 0);
    a.enqueue(0, NationId(0), 2, Command::Pause(false)).unwrap();
    assert!(a.step().unwrap().advanced);
    for (index, duration) in [500, 200, 80, 25, 0].into_iter().enumerate() {
        a.enqueue(
            a.snapshot().tick(),
            NationId(0),
            index as u64 + 3,
            Command::SetSpeed(index as u8 + 1),
        )
        .unwrap();
        a.step().unwrap();
        assert_eq!(a.ms_per_tick(), duration);
    }
    for invalid in [0, 6, u8::MAX] {
        a.enqueue(
            a.snapshot().tick(),
            NationId(0),
            u64::from(invalid) + 100,
            Command::SetSpeed(invalid),
        )
        .unwrap();
        assert!(a.step().unwrap().commands[0].result.is_err());
        assert_eq!(a.snapshot().speed(), 5);
    }
    assert_eq!(a.snapshot().hour(), 9);
}

#[test]
fn req_time_02_tick_day_month_snapshot_order() {
    let mut a = sim(Date::new(2000, 1, 31).unwrap());
    for _ in 0..23 {
        a.step().unwrap();
    }
    let boundary = a.step().unwrap();
    assert_eq!(
        boundary.phases,
        vec![
            Phase::Commands,
            Phase::Movement,
            Phase::Combat,
            Phase::RetreatAndControl,
            Phase::Supply,
            Phase::Economy,
            Phase::Production,
            Phase::Construction,
            Phase::ManpowerTrainingReinforcement,
            Phase::Research,
            Phase::Politics,
            Phase::AgendaAndDecisions,
            Phase::Events,
            Phase::Diplomacy,
            Phase::Ai,
            Phase::Ledger,
            Phase::MonthlyStatistics,
            Phase::Snapshot
        ]
    );
    assert_eq!(boundary.snapshot.date(), Date::new(2000, 2, 1).unwrap());
    assert_eq!(boundary.snapshot.hour(), 0);
    let normal = a.step().unwrap();
    assert_eq!(
        normal.phases,
        vec![
            Phase::Commands,
            Phase::Movement,
            Phase::Combat,
            Phase::RetreatAndControl,
            Phase::Snapshot
        ]
    );
    for _ in 0..22 {
        a.step().unwrap();
    }
    let daily = a.step().unwrap();
    assert!(daily.phases.contains(&Phase::Supply));
    assert!(!daily.phases.contains(&Phase::MonthlyStatistics));
    assert_eq!(daily.phases.last(), Some(&Phase::Snapshot));
}

#[test]
fn calendar_leap_year_and_year_boundaries() {
    for (year, day, next) in [
        (2000, 28, Date::new(2000, 2, 29).unwrap()),
        (1900, 28, Date::new(1900, 3, 1).unwrap()),
        (2000, 29, Date::new(2000, 3, 1).unwrap()),
    ] {
        let mut a = sim(Date::new(year, 2, day).unwrap());
        for _ in 0..24 {
            a.step().unwrap();
        }
        assert_eq!(a.snapshot().date(), next);
    }
    let mut a = sim(Date::new(2000, 12, 31).unwrap());
    for _ in 0..24 {
        a.step().unwrap();
    }
    assert_eq!(a.snapshot().date(), Date::new(2001, 1, 1).unwrap());
    assert!(Date::new(1900, 2, 29).is_err());
    assert!(Date::new(2000, 0, 1).is_err());
    assert!(Date::new(2000, 1, 0).is_err());
}

#[test]
fn hash_includes_seed_config_and_pending_commands() {
    let a = sim(Date::new(2000, 1, 1).unwrap());
    let mut b = sim(Date::new(2000, 1, 1).unwrap());
    b.enqueue(2, NationId(0), 1, Command::Pause(true)).unwrap();
    assert_ne!(a.state_hash().unwrap(), b.state_hash().unwrap());
    assert!(b.enqueue(2, NationId(0), 1, Command::Pause(false)).is_err());
    let seed = Simulation::new(
        "testland".into(),
        Date::new(2000, 1, 1).unwrap(),
        2,
        TimeConfig::new([500, 200, 80, 25, 0], 1).unwrap(),
    )
    .unwrap();
    assert_ne!(a.state_hash().unwrap(), seed.state_hash().unwrap());
    assert!(TimeConfig::new([500, 200, 80, 25, 0], 0).is_err());
}

#[test]
fn time_defines_are_required_checked_and_change_host_pacing() {
    use oh_data::{DefineValue, Defines, Number};
    use std::collections::BTreeMap;
    let define = |speeds: Vec<Number>, initial: i64| {
        Defines(BTreeMap::from([(
            "time".into(),
            BTreeMap::from([
                ("speed_ms_per_tick".into(), DefineValue::Array(speeds)),
                (
                    "initial_speed".into(),
                    DefineValue::Number(Number::Integer(initial)),
                ),
            ]),
        )]))
    };
    assert!(TimeConfig::from_defines(&Defines(BTreeMap::new())).is_err());
    for speeds in [
        vec![],
        vec![Number::Integer(10); 4],
        vec![Number::Integer(10); 6],
        vec![Number::Integer(-1); 5],
        vec![Number::Fixed(oh_core::Fx::from_num(1)); 5],
    ] {
        assert!(TimeConfig::from_defines(&define(speeds, 1)).is_err());
    }
    for initial in [-1, 0, 6, 256] {
        assert!(TimeConfig::from_defines(&define(vec![Number::Integer(10); 5], initial)).is_err());
    }
    let config = TimeConfig::from_defines(&define(vec![Number::Integer(7); 5], 3)).unwrap();
    let changed =
        Simulation::new("testland".into(), Date::new(2000, 1, 1).unwrap(), 1, config).unwrap();
    assert_eq!(changed.ms_per_tick(), 7);
    assert_eq!(changed.snapshot().speed(), 3);
    assert_ne!(
        changed.state_hash().unwrap(),
        sim(Date::new(2000, 1, 1).unwrap()).state_hash().unwrap()
    );
}

#[test]
fn overflow_preserves_state_and_queued_commands_and_pause_still_works() {
    let mut a = sim(Date::new(u32::MAX, 12, 31).unwrap());
    for _ in 0..23 {
        a.step().unwrap();
    }
    a.enqueue(23, NationId(0), 1, Command::SetSpeed(5)).unwrap();
    let before = a.state_hash().unwrap();
    assert_eq!(a.step().unwrap_err(), oh_sim::Error::ClockOverflow);
    assert_eq!(before, a.state_hash().unwrap());
    assert_eq!(a.snapshot().hour(), 23);
    assert_eq!(a.snapshot().speed(), 1);
    a.enqueue(23, NationId(0), 2, Command::Pause(true)).unwrap();
    let paused = a.step().unwrap();
    assert!(!paused.advanced);
    assert_eq!(paused.commands.len(), 2);
    assert_eq!(a.snapshot().speed(), 5);
}
