use oh_core::{DivisionId, NationId, ProvinceId};
use oh_sim::{Command, Simulation, military::Action};
fn apply(s: &mut Simulation, sequence: u64, command: Command) {
    s.enqueue(s.snapshot().tick(), NationId(1), sequence, command)
        .unwrap();
    assert!(s.step().unwrap().commands.iter().all(|c| c.result.is_ok()));
}
#[test]
fn deployed_m2_same_id_move_roster_rejections_stop_and_codec_resume() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/packs/testland_m2_military");
    let c = oh_save::SaveContext::national(&root, "m2_military").unwrap();
    let mut s = c.simulation(1).unwrap();
    let empty = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    let restored = oh_save::decode(&empty, &c, false).unwrap().simulation;
    assert_eq!(oh_save::encode(&restored, &c, 0, vec![1]).unwrap(), empty);
    apply(
        &mut s,
        1,
        Command::Military(Action::Train {
            template: "m2_small".into(),
        }),
    );
    while s.snapshot().tick() < 72 {
        s.step().unwrap();
    }
    apply(
        &mut s,
        2,
        Command::Military(Action::Deploy {
            job: 0,
            army: 0,
            province: ProvinceId(1),
            allow_understrength: true,
        }),
    );
    let id = s.military().unwrap().jobs()[&0].division().unwrap();
    assert_eq!(id, DivisionId(0));
    assert_eq!(
        s.movement().unwrap().unit(id).unwrap().province(),
        ProvinceId(1)
    );
    let world = s.world().unwrap();
    let destination = world
        .defs()
        .map()
        .edges
        .iter()
        .find_map(|e| {
            let other = if e.a == 1 {
                e.b
            } else if e.b == 1 {
                e.a
            } else {
                return None;
            };
            (e.kind != oh_data::map::EdgeKind::Strait
                && world.province(ProvinceId(other)).unwrap().controller() == Some(NationId(1)))
            .then_some(ProvinceId(other))
        })
        .expect("adjacent own land");
    for (nation, unit, destination) in [
        (NationId(2), id, destination),
        (NationId(1), DivisionId(999), destination),
        (NationId(1), id, ProvinceId(40)),
    ] {
        let before = s.state_hash().unwrap();
        assert!(
            s.enqueue(
                s.snapshot().tick(),
                nation,
                10,
                Command::Move { unit, destination }
            )
            .is_err()
        );
        assert_eq!(s.state_hash().unwrap(), before);
    }
    apply(
        &mut s,
        3,
        Command::Move {
            unit: id,
            destination,
        },
    );
    let bytes = oh_save::encode(&s, &c, 0, vec![1]).unwrap();
    let mut resumed = oh_save::decode(&bytes, &c, false).unwrap().simulation;
    assert_eq!(s.state_hash().unwrap(), resumed.state_hash().unwrap());
    for _ in 0..2048 {
        s.step().unwrap();
        resumed.step().unwrap();
        assert_eq!(s.state_hash().unwrap(), resumed.state_hash().unwrap());
        if s.movement().unwrap().unit(id).unwrap().province() == destination {
            break;
        }
    }
    assert_eq!(
        s.military().unwrap().divisions()[&id.0].province(),
        destination
    );
    apply(
        &mut s,
        4,
        Command::Move {
            unit: id,
            destination: ProvinceId(1),
        },
    );
    // Stop at an unstarted edge retains no route, per existing movement semantics.
    s.enqueue(s.snapshot().tick(), NationId(1), 5, Command::Pause(true))
        .unwrap();
    s.step().unwrap();
    // Existing Stop preserves an in-progress first edge; it discards subsequent legs.
    apply(&mut s, 6, Command::Stop { unit: id });
    assert!(
        s.movement()
            .unwrap()
            .unit(id)
            .unwrap()
            .remaining_route()
            .len()
            <= 1
    );
}
