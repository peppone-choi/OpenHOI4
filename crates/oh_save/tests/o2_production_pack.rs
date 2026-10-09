#[path = "../../oh_data/tests/support/o2_production_pack.rs"]
mod fixture;
use oh_core::{NationId, Qty, canonical_bytes};
use oh_sim::{
    Command, Simulation,
    production::{Action, ProductionError},
};

fn create_all(context: &oh_save::SaveContext) -> Simulation {
    let mut sim = context.simulation(7).unwrap();
    let definition = sim.world().unwrap().defs().production().unwrap().clone();
    for (nation, input) in definition.nations {
        let requested_ic = sim
            .economy()
            .unwrap()
            .nation(NationId(nation))
            .unwrap()
            .ledger()
            .allocation[2];
        assert!(requested_ic > Qty::ZERO);
        sim.enqueue(
            0,
            NationId(nation),
            1,
            Command::Production(Action::Create {
                model: input.allowed_models.first().unwrap().clone(),
                requested_ic,
            }),
        )
        .unwrap();
    }
    sim
}

fn days(sim: &mut Simulation, count: usize) -> [i128; 6] {
    let mut debits = [0_i128; 6];
    for _ in 0..count {
        for _ in 0..24 {
            let step = sim.step().unwrap();
            assert!(step.commands.iter().all(|result| result.result.is_ok()));
        }
        let day = sim.production().unwrap().day().unwrap();
        assert_eq!(day.nations.len(), 6);
        for (nation, entry) in &day.nations {
            assert_eq!(entry.lines.len(), 1);
            let debit = entry
                .lines
                .values()
                .map(|line| i128::from(line.resources["steel"].debited.to_bits()))
                .sum::<i128>();
            assert!(debit > 0);
            assert!(debit <= i128::from(entry.flows["steel"]) * i128::from(Qty::ONE.to_bits()));
            debits[usize::from(*nation - 1)] += debit;
        }
    }
    debits
}

#[test]
fn six_nation_365_days_twice_and_180_plus_185_real_v6_resume() {
    let pack = fixture::CopyPack::new();
    let context = oh_save::SaveContext::national(pack.root(), fixture::SCENARIO).unwrap();
    let mut first = create_all(&context);
    let debit_first = days(&mut first, 365);
    let mut second = create_all(&context);
    assert_eq!(days(&mut second, 365), debit_first);
    assert_eq!(
        canonical_bytes(&first).unwrap(),
        canonical_bytes(&second).unwrap()
    );
    assert_eq!(first.state_hash().unwrap(), second.state_hash().unwrap());
    let production = first.production().unwrap();
    assert_eq!(production.lines().len(), 6);
    assert_eq!(production.next_line_id(), 6);
    for line in production.lines().values() {
        assert!(production.stock(line.nation(), line.model()).unwrap() > 0);
    }

    let mut split = create_all(&context);
    let debit_180 = days(&mut split, 180);
    let bytes = oh_save::encode(&split, &context, 0, vec![1, 2, 3, 4, 5, 6]).unwrap();
    assert_eq!(
        oh_save::inspect_header(&bytes, &Default::default())
            .unwrap()
            .format_version,
        6
    );
    let mut resumed = oh_save::decode(&bytes, &context, false).unwrap().simulation;
    assert_eq!(
        canonical_bytes(&split).unwrap(),
        canonical_bytes(&resumed).unwrap()
    );
    let debit_185 = days(&mut resumed, 185);
    for n in 0..6 {
        assert_eq!(debit_180[n] + debit_185[n], debit_first[n]);
    }
    assert_eq!(
        canonical_bytes(&first).unwrap(),
        canonical_bytes(&resumed).unwrap()
    );
    assert_eq!(
        oh_save::encode(&first, &context, 0, vec![1, 2, 3, 4, 5, 6]).unwrap(),
        oh_save::encode(&resumed, &context, 0, vec![1, 2, 3, 4, 5, 6]).unwrap()
    );

    // Optional separate-process acceptance inputs; never part of the data pack.
    if let Some(directory) = std::env::var_os("OH_O2_OUTPUT_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        assert!(!directory.join("paused-180.ohsave").exists());
        split
            .enqueue(
                split.snapshot().tick(),
                NationId(1),
                2,
                Command::Pause(true),
            )
            .unwrap();
        split.step().unwrap();
        std::fs::write(
            directory.join("paused-180.ohsave"),
            oh_save::encode(&split, &context, 0, vec![1, 2, 3, 4, 5, 6]).unwrap(),
        )
        .unwrap();
        let summary = serde_json::json!({"tick":split.snapshot().tick().to_string(),"paused_hash":split.state_hash().unwrap().to_string(),"hash_365":first.state_hash().unwrap().to_string(),"steel_debit_bits_365":debit_first.map(|value| value.to_string()),"production":split.production().unwrap()});
        std::fs::write(
            directory.join("expected.json"),
            serde_json::to_vec_pretty(&summary).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn disallowed_model_rejection_is_atomic_and_allowed_control_creates() {
    let context = oh_save::SaveContext::national(&fixture::root(), fixture::SCENARIO).unwrap();
    let mut sim = context.simulation(7).unwrap();
    let requested_ic = sim
        .economy()
        .unwrap()
        .nation(NationId(2))
        .unwrap()
        .ledger()
        .allocation[2];
    let before = canonical_bytes(&sim).unwrap();
    let hash = sim.state_hash().unwrap();
    assert_eq!(
        sim.enqueue(
            0,
            NationId(2),
            1,
            Command::Production(Action::Create {
                model: "m2_equipment_2".into(),
                requested_ic
            })
        ),
        Err(oh_sim::Error::Production(ProductionError::InvalidReference))
    );
    assert_eq!(canonical_bytes(&sim).unwrap(), before);
    assert_eq!(sim.state_hash().unwrap(), hash);
    sim.enqueue(
        0,
        NationId(2),
        1,
        Command::Production(Action::Create {
            model: "m2_equipment_1".into(),
            requested_ic,
        }),
    )
    .unwrap();
    assert!(sim.step().unwrap().commands[0].result.is_ok());
    assert_eq!(sim.production().unwrap().lines().len(), 1);
}

#[test]
fn inventory_overflow_day_is_atomic_with_valid_control() {
    for overflow in [false, true] {
        let pack = fixture::CopyPack::new();
        if overflow {
            let path = pack.root().join("common/production/initial.toml");
            let text = std::fs::read_to_string(&path).unwrap();
            let from = "stock = { m2_equipment_1 = 0, m2_equipment_2 = 0 }";
            assert!(text.contains(from));
            std::fs::write(
                path,
                text.replacen(
                    from,
                    "stock = { m2_equipment_1 = 9223372036854775807, m2_equipment_2 = 0 }",
                    1,
                ),
            )
            .unwrap();
        }
        let context = oh_save::SaveContext::national(pack.root(), fixture::SCENARIO).unwrap();
        let mut sim = create_all(&context);
        for _ in 0..23 {
            sim.step().unwrap();
        }
        let before = canonical_bytes(&sim).unwrap();
        let hash = sim.state_hash().unwrap();
        if overflow {
            assert_eq!(
                sim.step(),
                Err(oh_sim::Error::Production(ProductionError::Overflow))
            );
            assert_eq!(canonical_bytes(&sim).unwrap(), before);
            assert_eq!(sim.state_hash().unwrap(), hash);
            assert_eq!(sim.snapshot().tick(), 23);
        } else {
            sim.step().unwrap();
            assert!(
                sim.production()
                    .unwrap()
                    .stock(NationId(1), "m2_equipment_1")
                    .unwrap()
                    > 0
            );
        }
    }
}
