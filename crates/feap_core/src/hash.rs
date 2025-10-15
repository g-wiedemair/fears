//! Provides replacements for `std::hash` items using [`foldhash`]

use core::hash::BuildHasher;
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
