use oh_proto::ClientMessage;

#[test]
fn movement_command_wire_decodes_for_joined_nation_dispatch() {
    let raw = r#"{"type":"MovementCommand","sequence":"1","command":{"type":"Move","division":"0","destination":2}}"#;
    let result = serde_json::from_str::<ClientMessage>(raw);
    assert!(result.is_ok(), "S1 movement wire must decode: {result:?}");
}

use oh_core::{DivisionId, Fx, NationId, ProvinceId};
use oh_proto::{FixedValue, MovementCommand, MovementView, ServerMessage};
use oh_sim::{Command, Simulation, military::Action};

fn simulation() -> Simulation {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/packs/testland_m2_military");
    let loaded = oh_data::national::load_scenario(&root, "m2_military").unwrap();
    Simulation::with_world(
        "m2_military".into(),
        oh_sim::Date::new(2000, 1, 1).unwrap(),
        1,
        oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
        oh_sim::world::World::from_loaded(&loaded).unwrap(),
    )
    .unwrap()
}
fn apply(sim: &mut Simulation, command: Command) {
    sim.enqueue(sim.snapshot().tick(), NationId(1), 1, command)
        .unwrap();
    assert!(
        sim.step()
            .unwrap()
            .commands
            .iter()
            .all(|c| c.result.is_ok())
    );
}
fn fixed(value: Fx) -> FixedValue {
    FixedValue {
        value: value.to_string(),
        bits: value.to_bits().to_string(),
        fractional_bits: 32,
    }
}
fn compare(sim: &Simulation) -> MovementView {
    let hash = sim.state_hash().unwrap();
    let view = MovementView::from_sim(sim).unwrap();
    assert_eq!(view.state_hash, format!("{hash:016x}"));
    assert_eq!(view.tick, sim.snapshot().tick().to_string());
    for (unit, projected) in sim.movement().unwrap().units().iter().zip(&view.units) {
        assert_eq!(projected.division, unit.id().0.to_string());
        assert_eq!(projected.nation, unit.nation().0);
        assert_eq!(projected.province, unit.province().0);
        assert_eq!(
            projected.route,
            unit.remaining_route()
                .iter()
                .map(|id| id.0)
                .collect::<Vec<_>>()
        );
        assert_eq!(projected.elapsed, fixed(unit.elapsed()));
        assert_eq!(projected.current_leg_hours, fixed(unit.current_leg_hours()));
        assert_eq!(
            projected.remaining_hours,
            fixed(unit.remaining_hours().unwrap())
        );
        assert!(unit.elapsed() <= unit.current_leg_hours());
    }
    assert_eq!(view.units.len(), sim.movement().unwrap().units().len());
    assert_eq!(sim.state_hash().unwrap(), hash, "query is read-only");
    let packet = ServerMessage::MovementResult {
        request: u64::MAX.to_string(),
        supported: true,
        reason_key: None,
        movement: Some(view.clone()),
    };
    assert_eq!(
        oh_proto::decode_server(&oh_proto::encode(&packet).unwrap()).unwrap(),
        packet
    );
    view
}

#[test]
fn movement_strict_command_ids_fields_and_roundtrip() {
    for command in [
        MovementCommand::Move {
            division: u32::MAX.to_string(),
            destination: u16::MAX,
        },
        MovementCommand::Stop {
            division: "0".into(),
        },
    ] {
        assert!(command.clone().into_action().is_ok());
        let packet = ClientMessage::MovementCommand {
            sequence: u64::MAX.to_string(),
            command,
        };
        assert_eq!(
            oh_proto::decode_client(&oh_proto::encode(&packet).unwrap()).unwrap(),
            packet
        );
    }
    for id in ["", "-1", "+1", "01", " 1", "1 ", "4294967296", "1.0"] {
        assert!(
            MovementCommand::Stop {
                division: id.into()
            }
            .into_action()
            .is_err()
        );
        assert!(
            MovementCommand::Move {
                division: id.into(),
                destination: 1
            }
            .into_action()
            .is_err()
        );
    }
    for raw in [
        r#"{"type":"Move","division":"0","destination":65536}"#,
        r#"{"type":"Move","division":"0","destination":-1}"#,
        r#"{"type":"Move","division":"0","destination":2,"nation":2}"#,
        r#"{"type":"Move","division":"0","destination":2,"speed":"10"}"#,
        r#"{"type":"Stop","division":"0","allowed":[1,2]}"#,
        r#"{"type":"Stop","division":0}"#,
    ] {
        assert!(
            serde_json::from_str::<MovementCommand>(raw).is_err(),
            "{raw}"
        );
    }
}

#[test]
fn movement_query_empty_context_is_supported_and_read_only() {
    let sim = simulation();
    assert!(sim.movement().is_none());
    let hash = sim.state_hash().unwrap();
    let view = MovementView::from_sim(&sim).unwrap();
    assert!(view.units.is_empty());
    assert_eq!(view.tick, "0");
    assert_eq!(view.state_hash, format!("{hash:016x}"));
    assert_eq!(sim.state_hash().unwrap(), hash);
}

#[test]
fn movement_query_matches_unit_getters_leg_sum_stop_and_arrival_roster() {
    let mut sim = simulation();
    apply(
        &mut sim,
        Command::Military(Action::Train {
            template: "m2_small".into(),
        }),
    );
    while sim.snapshot().tick() < 72 {
        sim.step().unwrap();
    }
    apply(&mut sim, Command::Pause(true));
    apply(
        &mut sim,
        Command::Military(Action::Deploy {
            job: 0,
            army: 0,
            province: ProvinceId(1),
            allow_understrength: true,
        }),
    );
    let id = DivisionId(0);
    let empty = compare(&sim);
    assert!(empty.units[0].route.is_empty());
    assert_eq!(empty.units[0].current_leg_hours, fixed(Fx::ZERO));
    apply(
        &mut sim,
        Command::Move {
            unit: id,
            destination: ProvinceId(14),
        },
    );
    let planned = compare(&sim);
    assert!(planned.units[0].route.len() >= 2);
    let mut total = Fx::ZERO;
    let mut from = ProvinceId(1);
    let movement = sim.movement().unwrap();
    let world = sim.world().unwrap();
    for to in &planned.units[0].route {
        // PR21 D2 uses identity factors; verify the total independently from DTO assembly.
        let edge = world.defs().map().edge(from.0, *to).unwrap();
        let hours = edge
            .distance_km
            .checked_div(movement.unit(id).unwrap().speed())
            .unwrap();
        total = total.checked_add(hours).unwrap();
        from = ProvinceId(*to);
    }
    assert_eq!(planned.units[0].remaining_hours, fixed(total));
    apply(&mut sim, Command::Pause(false));
    apply(&mut sim, Command::Pause(true));
    let moving = compare(&sim);
    assert!(moving.units[0].elapsed.bits.parse::<i64>().unwrap() > 0);
    assert_eq!(
        moving.units[0].remaining_hours,
        fixed(total.checked_sub(Fx::ONE).unwrap())
    );
    let leg_end = moving.units[0].route[0];
    apply(&mut sim, Command::Stop { unit: id });
    let stopped = compare(&sim);
    assert_eq!(stopped.units[0].route, vec![leg_end]);
    let unit = sim.movement().unwrap().unit(id).unwrap();
    assert_eq!(
        stopped.units[0].remaining_hours,
        fixed(
            unit.current_leg_hours()
                .checked_sub(unit.elapsed())
                .unwrap()
        )
    );
    apply(&mut sim, Command::Pause(false));
    for _ in 0..2048 {
        if sim
            .movement()
            .unwrap()
            .unit(id)
            .unwrap()
            .remaining_route()
            .is_empty()
        {
            break;
        }
        sim.step().unwrap();
    }
    apply(&mut sim, Command::Pause(true));
    let arrived = compare(&sim);
    assert_eq!(arrived.units[0].province, leg_end);
    assert_eq!(
        sim.military().unwrap().divisions()[&id.0].province().0,
        leg_end
    );
    assert_eq!(arrived.units[0].elapsed, fixed(Fx::ZERO));
    assert_eq!(arrived.units[0].current_leg_hours, fixed(Fx::ZERO));
    assert_eq!(arrived.units[0].remaining_hours, fixed(Fx::ZERO));
    apply(&mut sim, Command::Stop { unit: id });
    assert_eq!(compare(&sim), arrived);
}
