//! Synthetic core-format diagnostic, not a scenario or simulation loop.
use std::collections::BTreeMap;

use oh_core::{
    DivisionId, Fx, GameDay, NationId, ProvinceId, Qty, Rng, RngKey, StateId, SystemId,
    simulation_rng,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub province: ProvinceId,
    pub state: StateId,
    pub nation: NationId,
    pub ratio: Fx,
    pub amount: Qty,
    pub draws: Vec<u64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticSnapshot {
    pub day: GameDay,
    pub entities: BTreeMap<DivisionId, Entry>,
}

pub fn diagnostic_snapshot() -> DiagnosticSnapshot {
    let day = GameDay(0xfedcba9876543210);
    let ids = [0, 1, 127, u32::MAX];
    let bits = [i64::MIN, -1, 1, i64::MAX];
    let entities = ids
        .into_iter()
        .enumerate()
        .map(|(index, id)| {
            let mut rng = simulation_rng(RngKey::new(
                0x0123456789abcdef,
                SystemId(0x87654321),
                day,
                DivisionId(id),
            ));
            (
                DivisionId(id),
                Entry {
                    province: ProvinceId(u16::MAX),
                    state: StateId(128),
                    nation: NationId(0),
                    ratio: Fx::from_bits(bits[index]),
                    amount: Qty::from_bits(bits[3 - index]),
                    draws: (0..4).map(|_| rng.next_u64()).collect(),
                },
            )
        })
        .collect();
    DiagnosticSnapshot { day, entities }
}
