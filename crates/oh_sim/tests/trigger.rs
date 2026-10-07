use oh_sim::{Command, Date, Simulation, TimeConfig, world::World};
fn sim(end: &str) -> Simulation {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
    loaded.scenario.start_date = "2000-02-28".into();
    loaded.scenario.end_date = Some(end.into());
    let config = TimeConfig::from_defines(&loaded.pack.defines).unwrap();
    Simulation::with_world(
        "m1".into(),
        Date::new(2000, 2, 28).unwrap(),
        1,
        config,
        World::from_loaded(&loaded).unwrap(),
    )
    .unwrap()
}
#[test]
fn req_time_04_inclusive_checkpoint_and_future_queue_preserved() {
    let mut sim = sim("2000-02-29");
    sim.enqueue(99, oh_core::NationId(0), 1, Command::SetSpeed(5))
        .unwrap();
    for _ in 0..47 {
        sim.step().unwrap();
    }
    assert!(!sim.is_ended());
    sim.step().unwrap();
    assert!(sim.is_ended());
    assert_eq!(sim.snapshot().date(), Date::new(2000, 3, 1).unwrap());
    assert_eq!(sim.snapshot().hour(), 0);
    let bytes = oh_core::canonical_bytes(&sim).unwrap();
    assert!(sim.step().is_err());
    assert!(
        sim.enqueue(48, oh_core::NationId(0), 2, Command::Pause(true))
            .is_err()
    );
    assert_eq!(bytes, oh_core::canonical_bytes(&sim).unwrap());
    assert_eq!(sim.pending_commands().len(), 1);
}
fn loaded() -> oh_data::national::LoadedNational {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
    let mut l = oh_data::national::load_scenario(&root, "m1").unwrap();
    l.nations[1].id = 42; // Independent non-contiguous typed ID input.
    l
}
fn build(l: &oh_data::national::LoadedNational) -> Simulation {
    let (y, m, d) = oh_data::trigger::date(&l.scenario.start_date).unwrap();
    Simulation::with_world(
        "m1".into(),
        Date::new(y, m, d).unwrap(),
        7,
        TimeConfig::from_defines(&l.pack.defines).unwrap(),
        World::from_loaded(l).unwrap(),
    )
    .unwrap()
}
fn program(l: &mut oh_data::national::LoadedNational, id: &str, es: Vec<oh_data::trigger::Effect>) {
    l.scenario.flag_keys = Some(vec!["x".into(), "y".into()]);
    l.scenario
        .effect_programs
        .get_or_insert_with(Default::default)
        .insert(
            id.into(),
            oh_data::trigger::EffectProgram {
                root: Some("NTH".into()),
                effects: es,
            },
        );
}
fn cmd(id: &str) -> Command {
    Command::Effects { program: id.into() }
}
#[test]
fn req_time_04_calendar_centuries_year_initial_and_legacy_none() {
    for (start, end, ticks, checkpoint) in [
        ("1900-02-28", "1900-03-01", 48, "1900-03-02"),
        ("2000-02-28", "2000-03-01", 72, "2000-03-02"),
        ("2100-02-28", "2100-03-01", 48, "2100-03-02"),
        ("2099-12-31", "2099-12-31", 24, "2100-01-01"),
        ("2000-12-30", "2000-12-31", 48, "2001-01-01"),
        ("2004-02-29", "2004-02-29", 24, "2004-03-01"),
    ] {
        let mut l = loaded();
        l.scenario.start_date = start.into();
        l.scenario.end_date = Some(end.into());
        let mut s = build(&l);
        for _ in 0..ticks - 1 {
            s.step().unwrap();
        }
        assert!(!s.is_ended());
        assert_eq!(s.snapshot().hour(), 23);
        s.step().unwrap();
        assert!(s.is_ended());
        assert_eq!(s.snapshot().date().to_string(), checkpoint);
        assert_eq!(s.snapshot().tick(), ticks);
    }
    let mut l = loaded();
    l.scenario.end_conditions =
        Some(oh_data::trigger::parse_condition(r#"{"date_gte":"2000-01-01"}"#).unwrap());
    let s = build(&l);
    assert!(s.is_ended());
    assert_eq!(s.snapshot().tick(), 0);
    assert_eq!(
        s.trigger_state().unwrap().ended.as_ref().unwrap().causes,
        vec![oh_sim::trigger::EndCause::Condition]
    );
    let s = build(&loaded());
    assert!(s.trigger_state().is_none());
    assert_eq!(
        oh_core::canonical_bytes(&s).unwrap(),
        oh_core::canonical_bytes(&(s.snapshot(), s.config(), s.pending_commands(), s.world()))
            .unwrap()
    );
}
#[test]
fn req_agd_05_actual_world_queries_and_read_only_conditions() {
    let mut l = loaded();
    l.scenario.flag_keys = Some(vec!["x".into()]);
    let s = build(&l);
    let snapshot = s.snapshot();
    let host =
        oh_sim::trigger::SimHost::new(s.world().unwrap(), &snapshot, s.trigger_state().unwrap());
    let before = oh_core::canonical_bytes(&s).unwrap();
    for (text, expected) in [
        (r#"{"nation_is":"NTH"}"#, true),
        (r#"{"controls_province":20}"#, false),
        (r#"{"owns_state":1}"#, true),
        (r#"{"owns_state":2}"#, false),
        (
            r#"{"ideology_support":{"ideology":"ideology-test-civic","value":{"eq":"0.75"}}}"#,
            true,
        ),
        (
            r#"{"all":[{"owns_state":1},{"not":{"nation_is":"NTH"}}]}"#,
            false,
        ),
        (r#"{"any":[{"owns_state":2},{"nation_is":"NTH"}]}"#, true),
    ] {
        // Typed AST validates before immutable evaluation; actual values are independently specified.
        let c = oh_data::trigger::parse_condition(text).unwrap();
        assert_eq!(
            oh_sim::trigger::evaluate(
                &host,
                &c,
                oh_sim::trigger::Scope::Nation(oh_core::NationId(1))
            )
            .unwrap(),
            expected
        );
    }
    assert_eq!(before, oh_core::canonical_bytes(&s).unwrap());
}
#[test]
fn req_agd_05_scope_restoration_idempotence_and_v4_restore() {
    let mut l = loaded();
    program(&mut l,"scoped",oh_data::trigger::parse_effects(r#"[{"set_flag":"x"},{"set_flag":"x"},{"scope":{"target":"nation:STH","effects":[{"set_flag":"x"}]}},{"clear_flag":"x"},{"clear_flag":"x"}]"#).unwrap());
    let mut s = build(&l);
    s.enqueue(0, oh_core::NationId(1), 1, cmd("scoped"))
        .unwrap();
    s.step().unwrap();
    assert_eq!(
        s.trigger_state().unwrap().flags,
        vec![(1, vec![]), (42, vec!["x".into()])]
    );
    let context = oh_sim::save_state::RestoreContext {
        scenario: "m1".into(),
        start_date: Date::new(2000, 1, 1).unwrap(),
        world: Some(World::from_loaded(&l).unwrap()),
    };
    let dto = s.export_save_v4().unwrap();
    let restored = Simulation::from_save_v4(dto.clone(), &context).unwrap();
    assert_eq!(dto, restored.export_save_v4().unwrap());
    assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
    let mut bad = dto.clone();
    bad.trigger.flags[0].1 = vec!["x".into(), "x".into()];
    assert!(Simulation::from_save_v4(bad, &context).is_err());
    let mut bad = dto;
    bad.trigger.definitions_hash ^= 1;
    assert!(Simulation::from_save_v4(bad, &context).is_err());
    assert!(s.export_save().is_err());
    assert!(s.export_save_v2().is_err());
    assert!(s.export_save_v3().is_err());
}
#[test]
fn req_agd_05_effect_1000_1001_and_mid_failure_are_full_transactions() {
    use oh_data::trigger::Effect;
    let mut l = loaded();
    program(&mut l, "budget", vec![]);
    let s = build(&l);
    let snapshot = s.snapshot();
    let mut host =
        oh_sim::trigger::SimHost::new(s.world().unwrap(), &snapshot, s.trigger_state().unwrap());
    let scope = oh_sim::trigger::Scope::Nation(oh_core::NationId(1));
    assert_eq!(
        oh_sim::trigger::execute(&mut host, &vec![Effect::SetFlag("x".into()); 1000], scope)
            .unwrap(),
        1000
    );
    let mut before = s.trigger_state().unwrap().clone();
    host.clone().commit(&mut before);
    assert_eq!(before.flags[0].1, vec!["x"]);
    assert_eq!(
        oh_sim::trigger::execute(&mut host, &vec![Effect::ClearFlag("x".into()); 1001], scope),
        Err(oh_sim::trigger::TriggerError::Budget)
    );
    let mut after = s.trigger_state().unwrap().clone();
    host.clone().commit(&mut after);
    assert_eq!(before, after);
    let failing = vec![
        Effect::ClearFlag("x".into()),
        Effect::EndScenario("a".into()),
        Effect::AddManpower(1),
    ];
    assert_eq!(
        oh_sim::trigger::execute(&mut host, &failing, scope),
        Err(oh_sim::trigger::TriggerError::Unsupported)
    );
    assert!(host.sources.is_empty());
    host.commit(&mut after);
    assert_eq!(before, after);
}
#[test]
fn req_agd_05_enqueue_preview_then_scheduled_budget_failure_is_consumed() {
    use oh_data::trigger::{Effect, IfArg};
    let mut l = loaded();
    let condition = oh_data::trigger::parse_condition(r#"{"date_gte":"2000-01-02"}"#).unwrap();
    let mut excessive = vec![Effect::SetFlag("x".into()); 1000];
    excessive.push(Effect::EndScenario("late".into()));
    program(
        &mut l,
        "later",
        vec![Effect::If(IfArg {
            condition,
            then: excessive,
            r#else: vec![Effect::ClearFlag("x".into())],
        })],
    );
    let mut s = build(&l);
    s.enqueue(24, oh_core::NationId(1), 1, cmd("later"))
        .unwrap();
    s.enqueue(99, oh_core::NationId(1), 2, cmd("later"))
        .unwrap();
    for _ in 0..24 {
        s.step().unwrap();
    }
    let flags = s.trigger_state().unwrap().clone();
    let step = s.step().unwrap();
    assert_eq!(
        step.commands[0].result,
        Err(oh_sim::Error::Trigger(
            oh_sim::trigger::TriggerError::Budget
        ))
    );
    assert_eq!(s.snapshot().tick(), 25);
    assert_eq!(s.trigger_state().unwrap(), &flags);
    assert_eq!(s.pending_commands().len(), 1);
    let before = oh_core::canonical_bytes(&s).unwrap();
    assert!(
        s.enqueue(25, oh_core::NationId(1), 3, cmd("later"))
            .is_err()
    );
    assert_eq!(before, oh_core::canonical_bytes(&s).unwrap());
}
#[test]
fn req_time_04_simultaneous_sources_sorted_and_phase_error_preserves_all() {
    use oh_data::trigger::Effect;
    use oh_sim::trigger::EndCause;
    let mut l = loaded();
    l.scenario.end_date = Some("2000-01-01".into());
    l.scenario.end_conditions =
        Some(oh_data::trigger::parse_condition(r#"{"date_gte":"2000-01-02"}"#).unwrap());
    program(
        &mut l,
        "end",
        vec![
            Effect::EndScenario("z".into()),
            Effect::EndScenario("a".into()),
            Effect::EndScenario("a".into()),
        ],
    );
    let mut s = build(&l);
    s.enqueue(23, oh_core::NationId(1), 1, cmd("end")).unwrap();
    s.enqueue(90, oh_core::NationId(1), 2, cmd("end")).unwrap();
    for _ in 0..24 {
        s.step().unwrap();
    }
    assert_eq!(
        s.trigger_state().unwrap().ended.as_ref().unwrap().causes,
        vec![
            EndCause::Date,
            EndCause::Condition,
            EndCause::Explicit("a".into()),
            EndCause::Explicit("z".into())
        ]
    );
    assert_eq!(s.pending_commands().len(), 1);
    let mut l = loaded();
    l.scenario.start_date = format!("{}-12-31", u32::MAX);
    l.scenario.flag_keys = Some(vec!["x".into()]);
    program(
        &mut l,
        "set",
        vec![Effect::SetFlag("x".into()), Effect::EndScenario("a".into())],
    );
    let mut s = build(&l);
    for _ in 0..23 {
        s.step().unwrap();
    }
    s.enqueue(23, oh_core::NationId(1), 1, cmd("set")).unwrap();
    let before = oh_core::canonical_bytes(&s).unwrap();
    assert_eq!(s.step().unwrap_err(), oh_sim::Error::ClockOverflow);
    assert_eq!(before, oh_core::canonical_bytes(&s).unwrap());
}
#[test]
fn score_arithmetic_requires_actual_inputs_and_checked_bits() {
    use oh_core::Fx;
    let result = oh_sim::formula::weighted_score(
        [Fx::from_num(2), Fx::from_num(3), Fx::ZERO, Fx::ZERO],
        [Some(Fx::from_num(4)), Some(Fx::from_num(5)), None, None],
    )
    .unwrap();
    assert_eq!(result.1.to_bits(), 23i64 << 32);
    assert_eq!(result.0[2], None);
    assert!(
        oh_sim::formula::weighted_score([Fx::ONE; 4], [Some(Fx::ONE), None, None, None]).is_err()
    );
    assert!(oh_sim::formula::weighted_score([Fx::MAX; 4], [Some(Fx::MAX); 4]).is_err());
}
#[test]
fn req_agd_05_effect_root_depth16_17_and_unselected_branch_budget() {
    use oh_data::trigger::{Effect, IfArg, ScopeArg};
    let mut l = loaded();
    program(&mut l, "empty", vec![]);
    let s = build(&l);
    let snapshot = s.snapshot();
    let mut host =
        oh_sim::trigger::SimHost::new(s.world().unwrap(), &snapshot, s.trigger_state().unwrap());
    let scope = oh_sim::trigger::Scope::Nation(oh_core::NationId(1));
    let mut es = vec![Effect::SetFlag("x".into())];
    for _ in 1..16 {
        es = vec![Effect::Scope(ScopeArg {
            target: "self".into(),
            effects: es,
        })];
    }
    assert_eq!(oh_sim::trigger::execute(&mut host, &es, scope).unwrap(), 1);
    let mut before = s.trigger_state().unwrap().clone();
    host.clone().commit(&mut before);
    es = vec![Effect::Scope(ScopeArg {
        target: "self".into(),
        effects: es,
    })];
    assert_eq!(
        oh_sim::trigger::execute(&mut host, &es, scope),
        Err(oh_sim::trigger::TriggerError::Depth)
    );
    let mut after = s.trigger_state().unwrap().clone();
    host.clone().commit(&mut after);
    assert_eq!(before, after);
    let e = Effect::If(IfArg {
        condition: oh_data::trigger::Condition::NationIs("STH".into()),
        then: vec![Effect::ClearFlag("x".into()); 1001],
        r#else: vec![],
    });
    assert_eq!(oh_sim::trigger::execute(&mut host, &[e], scope).unwrap(), 0);
}
