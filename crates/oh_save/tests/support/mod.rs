use oh_core::{Fx, NationId};
use oh_save::SaveContext;
use oh_sim::{Command, Simulation};
use std::path::Path;
mod copy;
pub use copy::copy_pack;
pub fn mutable_pack(target: &Path) {
    copy_pack(target);
    let path = target.join("scenarios/m1/scenario.toml");
    let mut source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("2000-01-01", "2000-02-28");
    source.push_str("\n[state_modifiers]\n1 = [\n{ source = 'a.raw', target_stat = 'infrastructure', operation = 'add', value = '0.00000000023283064365386962890625', expires = 49 },\n{ source = 'b.add', target_stat = 'infrastructure', operation = 'add', value = '0.5' },\n{ source = 'c.mul', target_stat = 'infrastructure', operation = 'mul', value = '2' }\n]\n");
    std::fs::write(path, source).unwrap();
}
pub fn scheduled(context: &SaveContext) -> Simulation {
    let mut sim = context.simulation(7).unwrap();
    // Deliberately reversed insertion; execution order is nation then sequence.
    for (nation, sequence, command) in [
        (42, 3, Command::SetSpeed(5)),
        (0, 9, Command::Pause(true)),
        (42, 1, Command::Pause(false)),
        (0, 1, Command::SetSpeed(2)),
    ] {
        sim.enqueue(48, NationId(nation), sequence, command)
            .unwrap();
    }
    sim.enqueue(49, NationId(0), 10, Command::SetSpeed(3))
        .unwrap();
    sim
}
pub fn advance(sim: &mut Simulation, ticks: u64) {
    for _ in 0..ticks {
        assert!(sim.step().unwrap().advanced);
    }
}
pub fn assert_expiry(sim: &Simulation, expected: i64) {
    let state = &sim.world().unwrap().inputs().states()[0];
    assert_eq!(state.infrastructure().to_bits(), expected);
    assert_eq!(state.ledger().value().to_bits(), expected);
    assert_eq!(state.modifiers().len(), 3);
    assert_eq!(state.modifiers()[0].value, Fx::from_bits(1));
}
