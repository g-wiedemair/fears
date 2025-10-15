//! Provides `LockResult`, `PoisonError`, `TryLockError`, `TryLockResult`

pub use implementation::PoisonError;

#[cfg(feature = "std")]
use std::sync as implementation;

#[cfg(not(feature = "std"))]
mod implementation {}
