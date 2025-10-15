//! Provides `RwLock`, `RwLockReadGuard`, `RwLockWriteGuard`

pub use implementation::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(feature = "std")]
use std::sync as implementation;

#[cfg(not(feature = "std"))]
mod implementation {}
