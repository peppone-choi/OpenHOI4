#[path = "support/military.rs"]
mod fixture;
use oh_core::{NationId, ProvinceId};
use oh_sim::{
    Command,
    military::{Action, JobStatus},
};
fn command(s: &mut oh_sim::Simulation, sequence: u64, a: Action) {
    s.enqueue(
        s.snapshot().tick(),
        NationId(1),
        sequence,
        Command::Military(a),
    )
    .unwrap();
    assert!(s.step().unwrap().commands.iter().all(|c| c.result.is_ok()));
}
#[test]
fn req_mil_v7_real_codec_pending_training_ready_deployed_cancelled_and_resume() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    command(
        &mut s,
        1,
        Action::Train {
            template: "example".into(),
        },
    );
    for target in [1, 24, 48, 72] {
        while s.snapshot().tick() < target {
            s.step().unwrap();
        }
        s.enqueue(
            target + 10,
            NationId(1),
            100 + target,
            Command::Military(Action::SetPriority {
                army: 0,
                priority: 2,
            }),
        )
        .unwrap();
        let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
        assert_eq!(
            oh_save::inspect_header(&bytes, &Default::default())
                .unwrap()
                .format_version,
            7
        );
        let mut restored = oh_save::decode(&bytes, &c, false).unwrap().simulation;
        assert_eq!(
            oh_core::canonical_bytes(&s).unwrap(),
            oh_core::canonical_bytes(&restored).unwrap()
        );
        assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
        let mut continuous = s.clone();
        for _ in 0..12 {
            continuous.step().unwrap();
            restored.step().unwrap();
        }
        assert_eq!(
            oh_core::canonical_bytes(&continuous).unwrap(),
            oh_core::canonical_bytes(&restored).unwrap()
        );
    }
    assert_eq!(s.military().unwrap().jobs()[&0].status(), JobStatus::Ready);
    command(
        &mut s,
        500,
        Action::Deploy {
            job: 0,
            army: 0,
            province: ProvinceId(10),
            allow_understrength: true,
        },
    );
    command(
        &mut s,
        501,
        Action::Train {
            template: "example".into(),
        },
    );
    command(&mut s, 502, Action::Cancel { job: 1 });
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    let restored = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(
        restored.military().unwrap().jobs()[&0].status(),
        JobStatus::Deployed
    );
    assert_eq!(
        restored.military().unwrap().jobs()[&1].status(),
        JobStatus::Cancelled
    );
    assert_eq!(restored.military().unwrap().divisions()[&0].manpower(), 8);
    assert!(s.export_save_v6().is_err());
    let dto = s.export_save_v7().unwrap();
    assert!(dto.base.queue.is_empty());
    assert!(dto.base.base.queue.is_empty());
}
#[test]
fn req_mil_v7_untrusted_bounds_immutable_definitions_and_owned_accounting_rejections() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    command(
        &mut s,
        1,
        Action::Train {
            template: "example".into(),
        },
    );
    while s.snapshot().tick() < 24 {
        s.step().unwrap();
    }
    let bytes = oh_save::encode(&s, &c, 0, vec![]).unwrap();
    for n in [0, 1, 9, bytes.len() - 1] {
        assert!(oh_save::decode(&bytes[..n], &c, false).is_err());
    }
    for limits in [
        oh_save::Limits {
            queue_max_entries: 0,
            ..Default::default()
        },
        oh_save::Limits {
            allocation_budget_bytes: 1,
            ..Default::default()
        },
        oh_save::Limits {
            body_max_bytes: 1,
            ..Default::default()
        },
    ] {
        assert!(oh_save::decode_with_limits(&bytes, &c, false, &limits).is_err());
    }
    let dto = s.export_save_v7().unwrap();
    let raw = serde_json::to_value(&dto).unwrap();
    for (pointer, value) in [
        ("/military/definitions_hash", serde_json::json!(0)),
        ("/military/jobs/0/reserved_manpower", serde_json::json!(7)),
        ("/military/jobs/0/progress_days", serde_json::json!(1)),
        ("/military/jobs/0/normal/manpower", serde_json::json!(9)),
        ("/military/jobs/0/nation", serde_json::json!(2)),
        ("/military/background/1/reserved", serde_json::json!(11)),
        ("/military/armies/0/general", serde_json::json!("changed")),
    ] {
        let mut bad = raw.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        let bad = serde_json::from_value::<oh_sim::military_save::SimulationSaveV7>(bad).unwrap();
        assert!(
            oh_sim::Simulation::from_save_v7(bad, c.restore_context()).is_err(),
            "{pointer}"
        );
    }
    let old = oh_save::SaveContext::national(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap();
    for force in [false, true] {
        assert!(oh_save::decode(&bytes, &old, force).is_err());
    }
}

#[test]
fn req_mil_v7_mixed_queue_order_and_structural_reference_rejections() {
    use oh_sim::military_save::{CommandV7, PendingV7};
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = c.simulation(1).unwrap();
    s.enqueue(10, NationId(1), 1, Command::Pause(true)).unwrap();
    s.enqueue(
        10,
        NationId(1),
        2,
        Command::Military(Action::Train {
            template: "example".into(),
        }),
    )
    .unwrap();
    let dto = s.export_save_v7().unwrap();
    let restored = oh_sim::Simulation::from_save_v7(dto.clone(), c.restore_context()).unwrap();
    assert_eq!(
        oh_core::canonical_bytes(&s).unwrap(),
        oh_core::canonical_bytes(&restored).unwrap()
    );
    assert!(matches!(&dto.queue[0].command, CommandV7::Legacy(_)));
    assert!(matches!(&dto.queue[1].command, CommandV7::Military(_)));
    let mut unordered = dto.clone();
    unordered.queue.swap(0, 1);
    assert!(oh_sim::Simulation::from_save_v7(unordered, c.restore_context()).is_err());
    let mut duplicate = dto.clone();
    duplicate.queue[1].sequence = duplicate.queue[0].sequence;
    assert!(oh_sim::Simulation::from_save_v7(duplicate, c.restore_context()).is_err());
    let mut nested = dto.clone();
    let CommandV7::Legacy(legacy) = dto.queue[0].command.clone() else {
        unreachable!()
    };
    nested.base.queue.push(oh_sim::production_save::PendingV6 {
        tick: 10,
        nation: 1,
        sequence: 1,
        command: legacy,
    });
    assert!(oh_sim::Simulation::from_save_v7(nested, c.restore_context()).is_err());
    for (nation, action) in [
        (
            1,
            Action::Train {
                template: "missing".into(),
            },
        ),
        (
            9,
            Action::Train {
                template: "example".into(),
            },
        ),
        (1, Action::Cancel { job: 99 }),
        (
            1,
            Action::SetPriority {
                army: 1,
                priority: 0,
            },
        ),
    ] {
        let mut bad = dto.clone();
        bad.queue = vec![PendingV7 {
            tick: 10,
            nation,
            sequence: 1,
            command: CommandV7::Military(action),
        }];
        assert!(oh_sim::Simulation::from_save_v7(bad, c.restore_context()).is_err());
    }
}

// Boundary fixtures extend real authority-created records, then use the same
// untrusted V7 restore path as saves. They do not perform thousands of simulated
// training days or discard history to manufacture quota capacity.
fn quota_fixture(
    s: &oh_sim::Simulation,
    c: &oh_save::SaveContext,
    extra: &[(u16, JobStatus, usize)],
) -> oh_sim::Simulation {
    let mut raw = serde_json::to_value(s.export_save_v8().unwrap()).unwrap();
    let mut next = s.military().unwrap().next_job_id();
    let mut record = raw["military"]["jobs"]["0"].clone();
    record["progress_days"] = serde_json::json!(0);
    record["start_tick"] = serde_json::Value::Null;
    record["reserved_manpower"] = serde_json::json!(0);
    record["equipment"] = serde_json::json!({});
    record["division"] = serde_json::Value::Null;
    for &(nation, status, count) in extra {
        assert!(matches!(status, JobStatus::Pending | JobStatus::Cancelled));
        for _ in 0..count {
            record["id"] = serde_json::json!(next);
            record["nation"] = serde_json::json!(nation);
            record["status"] = serde_json::to_value(status).unwrap();
            raw["military"]["jobs"][next.to_string()] = record.clone();
            next += 1;
        }
    }
    raw["military"]["next_job_id"] = serde_json::json!(next);
    let dto = serde_json::from_value(raw).unwrap();
    oh_sim::Simulation::from_save_v8(dto, c.restore_context()).unwrap()
}

fn paused_job(c: &oh_save::SaveContext, tick: u64) -> oh_sim::Simulation {
    let mut s = c.simulation(1).unwrap();
    command(
        &mut s,
        1,
        Action::Train {
            template: "example".into(),
        },
    );
    while s.snapshot().tick() < tick {
        s.step().unwrap();
    }
    s.enqueue(tick, NationId(1), 2, Command::Pause(true))
        .unwrap();
    s.step().unwrap();
    assert!(s.snapshot().paused());
    s
}

fn quota_command(
    s: &mut oh_sim::Simulation,
    nation: u16,
    sequence: u64,
    a: Action,
) -> Result<(), oh_sim::Error> {
    s.enqueue(
        s.snapshot().tick(),
        NationId(nation),
        sequence,
        Command::Military(a),
    )
    .unwrap();
    let result = s.step().unwrap();
    assert_eq!(result.commands.len(), 1);
    result.commands[0].result
}

fn assert_military_resume(s: &oh_sim::Simulation, c: &oh_save::SaveContext) -> oh_sim::Simulation {
    let bytes = oh_save::encode(s, c, 0, vec![1, 2]).unwrap();
    assert_eq!(
        oh_save::inspect_header(&bytes, &Default::default())
            .unwrap()
            .format_version,
        if s.military().unwrap().jobs().len() > 4096 {
            8
        } else {
            7
        }
    );
    let restored = oh_save::decode(&bytes, c, false).unwrap().simulation;
    assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
    assert_eq!(
        oh_core::canonical_bytes(s).unwrap(),
        oh_core::canonical_bytes(&restored).unwrap()
    );
    assert_eq!(oh_save::encode(&restored, c, 0, vec![1, 2]).unwrap(), bytes);
    restored
}

#[test]
fn req_mil_quota_terminal_history_over_4096_preserves_ids_replay_and_resume() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = paused_job(&c, 72);
    command(
        &mut s,
        3,
        Action::Deploy {
            job: 0,
            army: 0,
            province: ProvinceId(10),
            allow_understrength: true,
        },
    );
    let mut s = quota_fixture(&s, &c, &[(1, JobStatus::Cancelled, 4096)]);
    assert_eq!(s.military().unwrap().jobs().len(), 4097);
    let history = s.military().unwrap().clone();
    let train = Command::Military(Action::Train {
        template: "example".into(),
    });
    let tick = s.snapshot().tick();
    s.enqueue(tick, NationId(1), 10, train.clone()).unwrap();
    let mut restored = assert_military_resume(&s, &c);
    let before = restored.state_hash().unwrap();
    assert_eq!(
        restored.enqueue(tick, NationId(1), 10, train),
        Err(oh_sim::Error::DuplicateCommand)
    );
    assert_eq!(restored.state_hash().unwrap(), before);
    assert!(s.step().unwrap().commands[0].result.is_ok());
    assert!(restored.step().unwrap().commands[0].result.is_ok());
    assert_eq!(s.state_hash().unwrap(), restored.state_hash().unwrap());
    assert_eq!(restored.military().unwrap().next_job_id(), 4098);
    assert_eq!(restored.military().unwrap().next_division_id(), 1);
    assert_eq!(
        restored.military().unwrap().jobs()[&4097].status(),
        JobStatus::Pending
    );
    for (&id, record) in history.jobs() {
        assert_eq!(&restored.military().unwrap().jobs()[&id], record);
    }
    quota_command(&mut restored, 1, 11, Action::Cancel { job: 4097 }).unwrap();
    let mut restored = assert_military_resume(&restored, &c);
    quota_command(
        &mut restored,
        1,
        12,
        Action::Train {
            template: "example".into(),
        },
    )
    .unwrap();
    assert_eq!(restored.military().unwrap().next_job_id(), 4099);
    assert_eq!(
        restored.military().unwrap().jobs()[&4097].status(),
        JobStatus::Cancelled
    );
    assert_eq!(
        restored.military().unwrap().jobs()[&4098].status(),
        JobStatus::Pending
    );
    for action in [
        Action::Cancel { job: 4097 },
        Action::Deploy {
            job: 0,
            army: 0,
            province: ProvinceId(10),
            allow_understrength: true,
        },
    ] {
        let before = restored.state_hash().unwrap();
        assert_eq!(
            quota_command(&mut restored, 1, 20, action),
            Err(oh_sim::Error::Military(
                oh_sim::military::MilitaryError::Terminal
            ))
        );
        assert_eq!(restored.state_hash().unwrap(), before);
    }
}

#[test]
fn req_mil_quota_pending_training_ready_count_cancel_and_deploy_release() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    for (tick, status) in [
        (1, JobStatus::Pending),
        (24, JobStatus::Training),
        (72, JobStatus::Ready),
    ] {
        let mut s = quota_fixture(&paused_job(&c, tick), &c, &[(1, JobStatus::Pending, 4095)]);
        assert_eq!(s.military().unwrap().jobs()[&0].status(), status);
        let before = s.state_hash().unwrap();
        assert_eq!(
            quota_command(
                &mut s,
                1,
                10,
                Action::Train {
                    template: "example".into()
                }
            ),
            Err(oh_sim::Error::Military(
                oh_sim::military::MilitaryError::ActiveLimit
            ))
        );
        assert_eq!(s.state_hash().unwrap(), before);
        quota_command(
            &mut s,
            2,
            11,
            Action::Train {
                template: "example".into(),
            },
        )
        .unwrap();
        assert_eq!(s.military().unwrap().jobs()[&4096].nation(), NationId(2));
        let mut s = assert_military_resume(&s, &c);
        let release = if status == JobStatus::Ready {
            Action::Deploy {
                job: 0,
                army: 0,
                province: ProvinceId(10),
                allow_understrength: true,
            }
        } else {
            Action::Cancel { job: 0 }
        };
        quota_command(&mut s, 1, 12, release).unwrap();
        quota_command(
            &mut s,
            1,
            13,
            Action::Train {
                template: "example".into(),
            },
        )
        .unwrap();
        assert_eq!(s.military().unwrap().next_job_id(), 4098);
        assert_eq!(s.military().unwrap().jobs()[&4097].nation(), NationId(1));
        let before = s.state_hash().unwrap();
        assert_eq!(
            quota_command(
                &mut s,
                1,
                14,
                Action::Train {
                    template: "example".into()
                }
            ),
            Err(oh_sim::Error::Military(
                oh_sim::military::MilitaryError::ActiveLimit
            ))
        );
        assert_eq!(s.state_hash().unwrap(), before);
        assert_military_resume(&s, &c);
    }
}

#[test]
fn req_mil_quota_restore_per_country_boundary_and_history_integrity() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let s = quota_fixture(
        &paused_job(&c, 1),
        &c,
        &[(1, JobStatus::Pending, 4095), (2, JobStatus::Pending, 4096)],
    );
    assert_eq!(s.military().unwrap().jobs().len(), 8192);
    let s = assert_military_resume(&s, &c);
    let raw = serde_json::to_value(s.export_save_v8().unwrap()).unwrap();
    for mutation in 0..4 {
        let mut bad = raw.clone();
        match mutation {
            0 => bad["military"]["jobs"]["4096"]["nation"] = serde_json::json!(1), // 4097 active in one nation.
            1 => {
                bad["military"]["jobs"].as_object_mut().unwrap().remove("0");
            } // History pruning prohibited.
            2 => bad["military"]["next_job_id"] = serde_json::json!(8191),
            _ => bad["military"]["jobs"]["8191"]["id"] = serde_json::json!(8190),
        }
        assert!(
            oh_sim::Simulation::from_save_v8(
                serde_json::from_value(bad).unwrap(),
                c.restore_context()
            )
            .is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn req_mil_quota_apply_order_rollback_and_ready_cancel_release() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let mut s = quota_fixture(&paused_job(&c, 72), &c, &[(1, JobStatus::Pending, 4094)]);
    let tick = s.snapshot().tick();
    for sequence in [10, 11] {
        s.enqueue(
            tick,
            NationId(1),
            sequence,
            Command::Military(Action::Train {
                template: "example".into(),
            }),
        )
        .unwrap();
    }
    let mut s = assert_military_resume(&s, &c);
    let result = s.step().unwrap();
    assert_eq!(result.commands.len(), 2);
    assert!(result.commands[0].result.is_ok());
    assert_eq!(
        result.commands[1].result,
        Err(oh_sim::Error::Military(
            oh_sim::military::MilitaryError::ActiveLimit
        ))
    );
    assert_eq!(s.military().unwrap().next_job_id(), 4096);
    let before = s.state_hash().unwrap();
    assert_eq!(
        quota_command(
            &mut s,
            1,
            12,
            Action::Deploy {
                job: 0,
                army: 0,
                province: ProvinceId(20),
                allow_understrength: true
            }
        ),
        Err(oh_sim::Error::Military(
            oh_sim::military::MilitaryError::NotOwner
        ))
    );
    assert_eq!(s.state_hash().unwrap(), before);
    // The full-quota future Train is structurally valid because cancellation
    // can release its slot before it applies; restore must preserve that order.
    s.enqueue(
        tick,
        NationId(1),
        13,
        Command::Military(Action::Cancel { job: 0 }),
    )
    .unwrap();
    s.enqueue(
        tick,
        NationId(1),
        14,
        Command::Military(Action::Train {
            template: "example".into(),
        }),
    )
    .unwrap();
    let mut s = assert_military_resume(&s, &c);
    assert!(s.step().unwrap().commands.iter().all(|c| c.result.is_ok()));
    assert_eq!(
        s.military().unwrap().jobs()[&0].status(),
        JobStatus::Cancelled
    );
    assert_eq!(s.military().unwrap().next_job_id(), 4097);
    assert_eq!(
        s.economy().unwrap().nation(NationId(1)).unwrap().reserved(),
        10
    );
    assert_military_resume(&s, &c);
}

#[test]
fn req_mil_extended_history_v8_is_distinct_and_v7_rejection_is_preserved() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let small = paused_job(&c, 1);
    let old = oh_save::encode(&small, &c, 0, vec![1]).unwrap();
    assert_eq!(
        oh_save::inspect_header(&old, &Default::default())
            .unwrap()
            .format_version,
        7
    );
    assert_eq!(
        oh_save::encode(
            &oh_save::decode(&old, &c, false).unwrap().simulation,
            &c,
            0,
            vec![1]
        )
        .unwrap(),
        old
    );
    let s = quota_fixture(&small, &c, &[(1, JobStatus::Cancelled, 4096)]);
    assert!(s.export_save_v7().is_err());
    assert!(
        oh_sim::Simulation::from_save_v7(s.export_save_v8().unwrap(), c.restore_context()).is_err()
    );
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    assert_eq!(&bytes[4..6], &8u16.to_le_bytes());
    let length = u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
    let mut header: oh_save::HeaderV1 =
        oh_core::from_canonical_bytes(&bytes[10..10 + length]).unwrap();
    header.format_version = 7;
    let header = oh_core::canonical_bytes(&header).unwrap();
    let mut relabelled = b"OHSV".to_vec();
    relabelled.extend_from_slice(&7u16.to_le_bytes());
    relabelled.extend_from_slice(&(header.len() as u32).to_le_bytes());
    relabelled.extend(header);
    relabelled.extend_from_slice(&bytes[10 + length..]);
    for force in [false, true] {
        assert!(
            oh_save::decode(&relabelled, &c, force)
                .err()
                .unwrap()
                .contains("InvalidV7: job record limit")
        );
    }
    assert_military_resume(&s, &c);
}

#[test]
fn req_mil_v8_encode_limits_preserve_previous_readable_atomic_save() {
    let root = fixture::pack();
    let c = oh_save::SaveContext::national(&root, "m1").unwrap();
    let small = paused_job(&c, 1);
    let good = oh_save::encode(&small, &c, 0, vec![1]).unwrap();
    let destination = tempfile::tempdir().unwrap();
    let path = destination.path().join("previous.ohsave");
    oh_save::write_atomic(&path, &good, &c).unwrap();
    let s = quota_fixture(&small, &c, &[(1, JobStatus::Cancelled, 4096)]);
    let valid = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    oh_save::decode(&valid, &c, false).unwrap();
    for limits in [
        oh_save::Limits {
            queue_max_entries: 4096,
            ..Default::default()
        },
        oh_save::Limits {
            body_max_bytes: 1,
            ..Default::default()
        },
        oh_save::Limits {
            allocation_budget_bytes: 1,
            ..Default::default()
        },
        oh_save::Limits {
            file_max_bytes: 1,
            ..Default::default()
        },
    ] {
        let result = oh_save::encode_with_limits(&s, &c, 0, vec![1], &limits)
            .and_then(|bytes| oh_save::write_atomic(&path, &bytes, &c));
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), good);
        oh_save::decode(&std::fs::read(&path).unwrap(), &c, false).unwrap();
        assert_eq!(std::fs::read_dir(destination.path()).unwrap().count(), 1);
    }
}
