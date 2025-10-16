//! Provides replacements for `std::hash` items using [`foldhash`]

use core::hash::{BuildHasher, Hasher};
pub use foldhash::fast::{FixedState, FoldHasher as DefaultHasher};

const FIXED_HASHER: FixedState =
    FixedState::with_seed(0b1001010111101110000001001100010000000011001001101011001001111000);

/// Deterministic hasher based upon a random but fixed state
#[derive(Copy, Clone, Default, Debug)]
pub struct FixedHasher;

impl BuildHasher for FixedHasher {
    type Hasher = DefaultHasher<'static>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        FIXED_HASHER.build_hasher()
    }
}

/// [`BuildHasher`] for types that already contain a high-quality hash
#[derive(Clone, Default)]
pub struct NoOpHash;

impl BuildHasher for NoOpHash {
    type Hasher = NoOpHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher(0)
    }
}

#[doc(hidden)]
pub struct NoOpHasher(u64);

// This is for types that already contain a high-quality hash and want to skip
// re-hashing that hash
impl Hasher for NoOpHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        // This should never be called by consumers.
        self.0 = bytes.iter().fold(self.0, |hash, b| {
            hash.rotate_left(8).wrapping_add(*b as u64)
        });
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}
