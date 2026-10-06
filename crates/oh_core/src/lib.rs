//! Deterministic value types and byte encodings shared by the simulation layers.
//!
//! This layer has no clocks, I/O, OS entropy, or game coefficients. Callers keep
//! state collections in ID order (`Vec`) or key order (`BTreeMap`) before hashing.
#![no_std]

extern crate alloc;

pub mod canonical;
pub mod hash;
pub mod ids;
pub mod rng;

/// Ratios and coefficients, as required by DR-01.
pub type Fx = fixed::types::I32F32;
/// Accumulated quantities, as required by DR-01.
pub type Qty = fixed::types::I48F16;

pub use canonical::{SerializationError, canonical_bytes, from_canonical_bytes, state_hash};
pub use hash::{Fnv1a64, fnv1a64};
pub use ids::{DivisionId, GameDay, NationId, ProvinceId, StateId, SystemId};
pub use rand_chacha::rand_core::Rng;
pub use rng::{EntityId, RngKey, simulation_rng};
