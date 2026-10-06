mod support;
use oh_core::{NationId, canonical_bytes};
use oh_save::{
    HeaderV1, Limits, SaveContext, decode, decode_with_limits, encode, encode_with_limits,
    inspect_header,
};
use oh_sim::{Command, Simulation, save_state::*};
use std::io::Write;

fn context() -> SaveContext {
    SaveContext::national(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
        "m1",
    )
    .unwrap()
}
fn mutate_body(bytes: &[u8], change: impl FnOnce(&mut SimulationSaveV1)) -> Vec<u8> {
    let end = 10 + u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
    let body = zstd::stream::decode_all(&bytes[end..]).unwrap();
    let mut dto: SimulationSaveV1 = oh_core::from_canonical_bytes(&body).unwrap();
    change(&mut dto);
    let mut result = bytes[..end].to_vec();
    result.extend(zstd::stream::encode_all(canonical_bytes(&dto).unwrap().as_slice(), 3).unwrap());
    result
}
fn mutate_header(bytes: &[u8], change: impl FnOnce(&mut HeaderV1)) -> Vec<u8> {
    let end = 10 + u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
    let mut h = inspect_header(bytes, &Limits::default()).unwrap();
    change(&mut h);
    let header = canonical_bytes(&h).unwrap();
    let mut result = bytes[..6].to_vec();
    result.extend((header.len() as u32).to_le_bytes());
    result.extend(header);
    result.extend(&bytes[end..]);
    result
}
#[test]
fn req_sav_02_actual_expiry_queue_order_and_pause() {
    let dir = tempfile::tempdir().unwrap();
    support::mutable_pack(&dir.path().join("pack"));
    let c = SaveContext::national(&dir.path().join("pack"), "m1").unwrap();
    let mut sim = support::scheduled(&c);
    support::advance(&mut sim, 48);
    support::assert_expiry(&sim, 12884901890);
    assert_eq!(sim.snapshot().date().to_string(), "2000-03-01");
    let bytes = encode(&sim, &c, 0, vec![]).unwrap();
    let mut resumed = decode(&bytes, &c, false).unwrap().simulation;
    let step = resumed.step().unwrap();
    assert_eq!(
        step.commands
            .iter()
            .map(|c| (c.nation.0, c.sequence))
            .collect::<Vec<_>>(),
        [(0, 1), (0, 9), (42, 1), (42, 3)]
    );
    assert_eq!(resumed.snapshot().speed(), 5);
    support::assert_expiry(&resumed, 12884901888);
    resumed.step().unwrap();
    assert_eq!(resumed.snapshot().speed(), 3);
    support::assert_expiry(&resumed, 12884901888);
    support::advance(&mut sim, 48);
    support::advance(&mut resumed, 46);
    assert_eq!(sim.state_hash().unwrap(), resumed.state_hash().unwrap());
    let mut paused = c.simulation(7).unwrap();
    paused
        .enqueue(0, NationId(0), 1, Command::Pause(true))
        .unwrap();
    assert!(!paused.step().unwrap().advanced);
    paused
        .enqueue(0, NationId(0), 2, Command::Pause(false))
        .unwrap();
    let bytes = encode(&paused, &c, 0, vec![]).unwrap();
    let mut loaded = decode(&bytes, &c, false).unwrap().simulation;
    assert!(loaded.snapshot().paused());
    assert_eq!(
        loaded.world().unwrap().inputs().states()[0].ledger().tick(),
        0
    );
    assert!(loaded.step().unwrap().advanced);
}
#[test]
fn req_sav_02_rawbits_fractional_floor_and_all_speeds() {
    let c = context();
    let original = c.simulation(7).unwrap();
    let mut dto = original.export_save().unwrap();
    dto.config = TimeV1 {
        speed_ms_per_tick: [0, 7, 11, 13, 17],
        initial_speed: 2,
    };
    let s = &mut dto.world.as_mut().unwrap().inputs.states[0];
    let q = 4294967296i64;
    s.base = 2 * q + 1;
    s.modifiers = vec![
        ModifierV1 {
            source: "a".into(),
            target_stat: "infrastructure".into(),
            op: ModifierOpV1::Add,
            value: q,
            expires: Some(24),
        },
        ModifierV1 {
            source: "b".into(),
            target_stat: "infrastructure".into(),
            op: ModifierOpV1::Mul,
            value: 3 * q / 2,
            expires: None,
        },
    ];
    s.infrastructure = 19327352833;
    s.ledger = LedgerV1 {
        target_stat: "infrastructure".into(),
        tick: 0,
        value: s.infrastructure,
        entries: vec![
            EntryV1 {
                source: None,
                op: LedgerOpV1::Base,
                value: 2 * q + 1,
                accumulated: 2 * q + 1,
            },
            EntryV1 {
                source: Some("a".into()),
                op: LedgerOpV1::Add,
                value: q,
                accumulated: 3 * q + 1,
            },
            EntryV1 {
                source: Some("b".into()),
                op: LedgerOpV1::Mul,
                value: 3 * q / 2,
                accumulated: 19327352833,
            },
        ],
    };
    for speed in 1..=5 {
        dto.state.speed = speed;
        let mut sim = Simulation::from_save(dto.clone(), c.restore_context()).unwrap();
        support::advance(&mut sim, 23);
        assert_eq!(
            sim.world().unwrap().inputs().states()[0]
                .infrastructure()
                .to_bits(),
            19327352833
        );
        let mut loaded = decode(&encode(&sim, &c, 0, vec![]).unwrap(), &c, false)
            .unwrap()
            .simulation;
        assert_eq!(loaded.config().speed_ms_per_tick(), [0, 7, 11, 13, 17]);
        assert_eq!(loaded.config().initial_speed(), 2);
        assert_eq!(loaded.snapshot().speed(), speed);
        for _ in 0..2 {
            loaded.step().unwrap();
            let s = &loaded.world().unwrap().inputs().states()[0];
            assert_eq!(s.infrastructure().to_bits(), 12884901889);
            assert_eq!(s.modifiers().len(), 2);
            assert_eq!(s.ledger().entries().len(), 2);
            let bytes = encode(&loaded, &c, 0, vec![]).unwrap();
            loaded = decode(&bytes, &c, false).unwrap().simulation;
        }
    }
}
#[test]
fn req_sav_01_invalid_inputs_and_ledger_never_change_live_or_queue() {
    let c = context();
    let mut sim = c.simulation(7).unwrap();
    sim.enqueue(20, NationId(65535), 0, Command::Pause(false))
        .unwrap();
    let bytes = encode(&sim, &c, 0, vec![]).unwrap();
    let before = canonical_bytes(&sim).unwrap();
    type Mutation = fn(&mut SimulationSaveV1);
    let changes: &[Mutation] = &[
        |d| d.state.date.year = 0,
        |d| d.state.date.month = 13,
        |d| d.state.hour = 24,
        |d| d.state.tick = 1,
        |d| d.state.speed = 0,
        |d| d.queue[0].command = CommandV1::SetSpeed(6),
        |d| d.queue.push(d.queue[0].clone()),
        |d| d.world = None,
        |d| d.world.as_mut().unwrap().inputs.nations[0].support[0].1 += 1,
        |d| {
            d.world.as_mut().unwrap().inputs.nations[0]
                .government
                .clear()
        },
        |d| d.world.as_mut().unwrap().inputs.nations[0].tag = "WRONG".into(),
        |d| d.world.as_mut().unwrap().inputs.states[0].owner = 65000,
        |d| d.world.as_mut().unwrap().inputs.states[0].population = -1,
        |d| d.world.as_mut().unwrap().inputs.states[0].base = -1,
        |d| d.world.as_mut().unwrap().inputs.states[0].resources[0].0 = "unknown".into(),
        |d| d.world.as_mut().unwrap().inputs.states[0].buildings[0].1 = -1,
        |d| {
            let s = &mut d.world.as_mut().unwrap().inputs.states[0];
            s.resources.push(s.resources[0].clone());
        },
        |d| d.world.as_mut().unwrap().inputs.states[0].infrastructure += 1,
        |d| d.world.as_mut().unwrap().inputs.states[0].ledger.tick = 1,
        |d| d.world.as_mut().unwrap().inputs.states[0].ledger.value += 1,
        |d| {
            d.world.as_mut().unwrap().inputs.states[0]
                .ledger
                .target_stat = "other".into()
        },
        |d| {
            d.world.as_mut().unwrap().inputs.states[0].ledger.entries[0].source =
                Some("fake".into())
        },
        |d| d.world.as_mut().unwrap().inputs.states[0].ledger.entries[0].accumulated += 1,
        |d| d.world.as_mut().unwrap().inputs.states[0].ledger.entries[0].value += 1,
        |d| d.world.as_mut().unwrap().inputs.states[0].ledger.entries[0].op = LedgerOpV1::Add,
        |d| d.world.as_mut().unwrap().inputs.provinces[0].state = None,
        |d| d.world.as_mut().unwrap().inputs.provinces[0].owner = Some(2),
        |d| d.world.as_mut().unwrap().inputs.provinces[0].controller = Some(65000),
        |d| d.world.as_mut().unwrap().inputs.provinces[4].owner = Some(1),
        |d| d.world.as_mut().unwrap().inputs.states.reverse(),
        |d| {
            d.world.as_mut().unwrap().inputs.nations.pop();
        },
        |d| d.world.as_mut().unwrap().definitions_hash ^= 1,
    ];
    for (index, change) in changes.iter().enumerate() {
        let corrupt = mutate_body(&bytes, change);
        assert!(decode(&corrupt, &c, false).is_err(), "mutation {index}");
        assert!(
            decode(&corrupt, &c, true).is_err(),
            "forced mutation {index}"
        );
        assert_eq!(canonical_bytes(&sim).unwrap(), before);
    }
    sim.step().unwrap();
    assert_eq!(sim.pending_commands().len(), 1);
}
#[test]
fn req_sav_01_header_and_container_corruption_limits_and_trailing() {
    let c = context();
    let sim = c.simulation(7).unwrap();
    let bytes = encode(&sim, &c, 0, vec![]).unwrap();
    for len in 0..bytes.len() {
        assert!(
            decode(&bytes[..len], &c, false).is_err(),
            "truncation {len}"
        );
    }
    type Mutation = fn(&mut HeaderV1);
    let mutations: &[Mutation] = &[
        |h| h.format_version = 2,
        |h| h.engine_version = "0.2.0".into(),
        |h| h.scenario_id = "other".into(),
        |h| h.tick += 1,
        |h| h.seed += 1,
        |h| h.game_date.day += 1,
        |h| h.state_hash ^= 1,
        |h| h.saved_at_utc = -1,
        |h| h.saved_at_utc = i64::MAX,
        |h| h.player_nations = vec![1, 1],
        |h| h.player_nations = vec![65000],
        |h| h.packs.clear(),
        |h| h.packs.push(h.packs[0].clone()),
        |h| h.definitions_hash = None,
        |h| h.effective_defines_hash ^= 1,
    ];
    for change in mutations {
        assert!(decode(&mutate_header(&bytes, change), &c, true).is_err());
    }
    for version in [0u16, 2] {
        let mut b = bytes.clone();
        b[4..6].copy_from_slice(&version.to_le_bytes());
        assert!(
            decode(&b, &c, true)
                .err()
                .unwrap()
                .contains("UnsupportedFormat")
        );
    }
    for length in [0u32, u32::MAX] {
        let mut b = bytes.clone();
        b[6..10].copy_from_slice(&length.to_le_bytes());
        assert!(decode(&b, &c, true).is_err());
    }
    let mut bad = bytes.clone();
    bad[0] = 0;
    assert!(decode(&bad, &c, false).is_err());
    let end = 10 + u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
    for tail in [
        vec![0],
        bytes[end..].to_vec(),
        zstd::stream::encode_all(&[][..], 3).unwrap(),
    ] {
        let mut b = bytes.clone();
        b.extend(tail);
        assert!(decode(&b, &c, false).err().unwrap().contains("Trailing"));
    }
    let mut h = bytes[10..end].to_vec();
    h.push(0);
    let mut b = bytes[..6].to_vec();
    b.extend((h.len() as u32).to_le_bytes());
    b.extend(h);
    b.extend(&bytes[end..]);
    assert!(decode(&b, &c, false).is_err());
    let mut body = zstd::stream::decode_all(&bytes[end..]).unwrap();
    body.push(0);
    let mut b = bytes[..end].to_vec();
    b.extend(zstd::stream::encode_all(body.as_slice(), 3).unwrap());
    assert!(decode(&b, &c, false).is_err());
    let mut limits = Limits {
        file_max_bytes: bytes.len() as u64 - 1,
        ..Limits::default()
    };
    assert!(decode_with_limits(&bytes, &c, false, &limits).is_err());
    limits.file_max_bytes += 1;
    assert!(decode_with_limits(&bytes, &c, false, &limits).is_ok());
    let body = zstd::stream::decode_all(&bytes[end..]).unwrap();
    limits.body_max_bytes = body.len() as u64 - 1;
    assert!(
        decode_with_limits(&bytes, &c, false, &limits)
            .err()
            .unwrap()
            .contains("Limit")
    );
    limits.body_max_bytes += 1;
    assert!(decode_with_limits(&bytes, &c, false, &limits).is_ok());
    limits.header_max_bytes = (end - 10) as u64 - 1;
    assert!(decode_with_limits(&bytes, &c, false, &limits).is_err());
    limits.header_max_bytes += 1;
    assert!(decode_with_limits(&bytes, &c, false, &limits).is_ok());
    let mut compressed = zstd::stream::write::Encoder::new(Vec::new(), 3).unwrap();
    compressed.window_log(24).unwrap();
    compressed.write_all(&vec![0; 9_000_000]).unwrap();
    let mut bomb = bytes[..end].to_vec();
    bomb.extend(compressed.finish().unwrap());
    assert!(decode_with_limits(&bomb, &c, false, &limits).is_err());
    let mut huge = vec![255; 9];
    huge.push(1);
    let mut bad = bytes[..6].to_vec();
    bad.extend((huge.len() as u32).to_le_bytes());
    bad.extend(huge);
    bad.extend(&bytes[end..]);
    assert!(decode(&bad, &c, false).err().unwrap().contains("Limit"));
    limits.string_max_bytes = 1;
    assert!(encode_with_limits(&sim, &c, 0, vec![], &limits).is_err());
}
#[test]
fn req_sav_01_force_only_accepts_pack_metadata_changes() {
    let dir = tempfile::tempdir().unwrap();
    support::copy_pack(&dir.path().join("pack"));
    let root = dir.path().join("pack");
    let c = SaveContext::national(&root, "m1").unwrap();
    let bytes = encode(&c.simulation(7).unwrap(), &c, 0, vec![]).unwrap();
    std::fs::write(root.join("SOURCES.md"), "original changed attribution\n").unwrap();
    let current = SaveContext::national(&root, "m1").unwrap();
    assert!(decode(&bytes, &current, false).is_err());
    let forced = decode(&bytes, &current, true).unwrap();
    assert_eq!(forced.warnings.len(), 1);
    assert_eq!(
        forced.header.state_hash,
        forced.simulation.state_hash().unwrap()
    );
    let path = root.join("scenarios/m1/defines.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, old.replace("500, 200", "501, 200")).unwrap();
    let changed = SaveContext::national(&root, "m1").unwrap();
    assert!(decode(&bytes, &changed, true).is_err());
}
#[test]
fn req_sav_01_allocation_budget_rejects_shape_before_activation() {
    let c = context();
    let bytes = encode(&c.simulation(7).unwrap(), &c, 0, vec![]).unwrap();
    let limits = Limits {
        allocation_budget_bytes: 1,
        ..Limits::default()
    };
    assert!(
        decode_with_limits(&bytes, &c, false, &limits)
            .err()
            .unwrap()
            .contains("allocation")
    );
}
#[test]
fn req_sav_01_signed_raw_extremes_and_display_timestamp_identity() {
    let c = context();
    let original = c.simulation(7).unwrap();
    for bits in [i64::MIN, i64::MAX, -1, 0, 1] {
        let mut dto = original.export_save().unwrap();
        let s = &mut dto.world.as_mut().unwrap().inputs.states[1];
        s.modifiers = vec![ModifierV1 {
            source: "test.raw".into(),
            target_stat: "infrastructure".into(),
            op: ModifierOpV1::Add,
            value: bits,
            expires: Some(u64::MAX),
        }];
        s.infrastructure = bits;
        s.ledger.value = bits;
        s.ledger.entries.push(EntryV1 {
            source: Some("test.raw".into()),
            op: LedgerOpV1::Add,
            value: bits,
            accumulated: bits,
        });
        let sim = Simulation::from_save(dto, c.restore_context()).unwrap();
        let bytes = encode(&sim, &c, 0, vec![]).unwrap();
        let other_time = encode(&sim, &c, 1, vec![]).unwrap();
        assert_ne!(bytes, other_time);
        let restored = decode(&bytes, &c, false).unwrap();
        let later = decode(&other_time, &c, false).unwrap();
        assert_eq!(restored.header.state_hash, later.header.state_hash);
        assert_eq!(
            restored.simulation.world().unwrap().inputs().states()[1].modifiers()[0]
                .value
                .to_bits(),
            bits
        );
    }
}
#[test]
fn req_sav_01_invalid_pending_export_preserves_m0_and_m1() {
    let m1 = context();
    let m0 = SaveContext::m0(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/examples/m0"),
        "testland",
    )
    .unwrap();
    for c in [m0, m1] {
        let mut sim = c.simulation(7).unwrap();
        sim.enqueue(0, NationId(65535), 0, Command::SetSpeed(0))
            .unwrap();
        let before = canonical_bytes(&sim).unwrap();
        assert!(
            encode(&sim, &c, 0, vec![])
                .err()
                .unwrap()
                .contains("InvalidQueue")
        );
        assert_eq!(before, canonical_bytes(&sim).unwrap());
        let step = sim.step().unwrap();
        assert!(step.commands[0].result.is_err());
        assert!(sim.pending_commands().is_empty());
        let bytes = encode(&sim, &c, 0, vec![]).unwrap();
        assert_eq!(
            decode(&bytes, &c, false)
                .unwrap()
                .simulation
                .state_hash()
                .unwrap(),
            sim.state_hash().unwrap()
        );
    }
}
