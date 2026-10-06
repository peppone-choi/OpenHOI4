//! Numeric references with separate types and fixed-width serialized storage.
//!
//! IDs do not allocate or recycle slots. Their owners maintain monotonic IDs and
//! alive flags as described in 02 §4.4. Zero is valid; no sentinel is reserved.
//!
//! Different entity kinds cannot be accidentally passed to each other:
//! ```compile_fail
//! use oh_core::{NationId, ProvinceId};
//! fn lookup_nation(_: NationId) {}
//! lookup_nation(ProvinceId(1));
//! ```

use core::fmt;
use serde::{Deserialize, Serialize};

macro_rules! id {
    ($name:ident, $storage:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
        )]
        #[repr(transparent)]
        #[serde(transparent)]
        pub struct $name(pub $storage);

        impl $name {
            /// Return the fixed-width numeric representation.
            pub const fn get(self) -> $storage {
                self.0
            }
        }

        impl From<$storage> for $name {
            fn from(value: $storage) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $storage {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

id!(ProvinceId, u16, "A province reference (02 §4.4).");
id!(StateId, u16, "A state reference (02 §4.4).");
id!(NationId, u16, "A nation reference (02 §4.4).");
id!(DivisionId, u32, "A division reference (02 §4.4).");
id!(
    SystemId,
    u32,
    "A stable simulation-system namespace for RNG derivation."
);
id!(
    GameDay,
    u64,
    "Elapsed simulation days since the scenario start, independent of wall time."
);
