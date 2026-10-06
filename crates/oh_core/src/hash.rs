//! The fixed 64-bit FNV-1a byte algorithm specified by DR-07.

// Format constants, not tunable simulation coefficients.
const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const PRIME: u64 = 0x100000001b3;

/// Incremental hash for byte streams; chunk boundaries have no effect.
/// This non-cryptographic hash must not be used for authentication.
#[derive(Clone, Copy, Debug)]
pub struct Fnv1a64(u64);

impl Fnv1a64 {
    pub const fn new() -> Self {
        Self(OFFSET_BASIS)
    }

    pub fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(PRIME);
        }
    }

    pub const fn finish(self) -> u64 {
        self.0
    }
}

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = Fnv1a64::new();
    hash.update(bytes);
    hash.finish()
}
