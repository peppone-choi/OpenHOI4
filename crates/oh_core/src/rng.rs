//! Per-system, per-day, per-entity ChaCha8 streams (DR-03).

use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};
use serde::{Deserialize, Serialize};

use crate::{DivisionId, GameDay, NationId, ProvinceId, StateId, SystemId};

/// Disjoint entity namespaces: equal raw IDs of different kinds get different streams.
///
/// Variant order and seed tags are a versioned encoding (ADR-0201). Append future
/// variants rather than renumbering these. `Global` is for a system without an entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum EntityId {
    Global,
    Province(ProvinceId),
    State(StateId),
    Nation(NationId),
    Division(DivisionId),
}

macro_rules! entity_conversion {
    ($id:ident, $variant:ident) => {
        impl From<$id> for EntityId {
            fn from(id: $id) -> Self {
                Self::$variant(id)
            }
        }
    };
}

entity_conversion!(ProvinceId, Province);
entity_conversion!(StateId, State);
entity_conversion!(NationId, Nation);
entity_conversion!(DivisionId, Division);

/// Everything needed to reconstruct the start of a stream; never a global RNG cursor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RngKey {
    pub game_seed: u64,
    pub system: SystemId,
    pub day: GameDay,
    pub entity: EntityId,
}

impl RngKey {
    pub fn new(
        game_seed: u64,
        system: SystemId,
        day: GameDay,
        entity: impl Into<EntityId>,
    ) -> Self {
        Self {
            game_seed,
            system,
            day,
            entity: entity.into(),
        }
    }

    /// Injective 32-byte tuple encoding, entirely little-endian:
    /// seed(u64), system(u32), day(u64), entity tag(u32), entity ID(u64).
    ///
    /// No native-endian conversion, hash compression, or hidden entropy is used.
    pub fn seed_bytes(self) -> [u8; 32] {
        let (tag, raw): (u32, u64) = match self.entity {
            EntityId::Global => (0, 0),
            EntityId::Province(id) => (1, u64::from(id.0)),
            EntityId::State(id) => (2, u64::from(id.0)),
            EntityId::Nation(id) => (3, u64::from(id.0)),
            EntityId::Division(id) => (4, u64::from(id.0)),
        };
        let mut seed = [0; 32];
        seed[..8].copy_from_slice(&self.game_seed.to_le_bytes());
        seed[8..12].copy_from_slice(&self.system.0.to_le_bytes());
        seed[12..20].copy_from_slice(&self.day.0.to_le_bytes());
        seed[20..24].copy_from_slice(&tag.to_le_bytes());
        seed[24..].copy_from_slice(&raw.to_le_bytes());
        seed
    }
}

/// Reconstruct a stream from the complete DR-03 key.
///
/// Create once for an entity/system/day evaluation and keep drawing from that
/// instance. Creating it again resets its position. Other streams and evaluation
/// order cannot affect its output. This is simulation randomness, not a secret key.
pub fn simulation_rng(key: RngKey) -> ChaCha8Rng {
    ChaCha8Rng::from_seed(key.seed_bytes())
}
