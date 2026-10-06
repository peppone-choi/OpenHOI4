//! Postcard serialization and hashing with an explicit state-ordering contract.
//!
//! Postcard preserves the order supplied by `Serialize`; it does not sort maps
//! or entities. State types must use key-ordered `BTreeMap`/`BTreeSet` or ID-ordered
//! vectors, fixed-width integers, and `Fx`/`Qty` (DR-01/02/07). Do not serialize
//! `HashMap`/`HashSet`, floats, pointer-sized state integers, or custom serializers
//! that depend on I/O/time/entropy. These obligations apply recursively to fields.
//! This generic API cannot inspect Rust field types and does not enforce that
//! contract; state definitions and their tests must do so.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::fnv1a64;

pub use postcard::Error as SerializationError;

/// Encode an already ordered state, returning errors instead of hashing partial bytes.
pub fn canonical_bytes<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, SerializationError> {
    postcard::to_allocvec(value)
}

/// Decode exactly one postcard value, rejecting trailing or truncated bytes.
///
/// The name describes the input format; this is not a validator of minimal
/// varints or the state-ordering contract. Borrowed fields are supported.
pub fn from_canonical_bytes<'de, T: Deserialize<'de>>(
    bytes: &'de [u8],
) -> Result<T, SerializationError> {
    let (value, remaining) = postcard::take_from_bytes(bytes)?;
    if remaining.is_empty() {
        Ok(value)
    } else {
        Err(SerializationError::DeserializeBadEncoding)
    }
}

/// Hash the complete canonical postcard bytes with FNV-1a 64 (DR-07).
pub fn state_hash<T: Serialize + ?Sized>(state: &T) -> Result<u64, SerializationError> {
    Ok(fnv1a64(&canonical_bytes(state)?))
}
