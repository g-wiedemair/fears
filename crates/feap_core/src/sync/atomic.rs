//! Provides various atomic alternatives to language primitives
//!
//! Certain platforms lack complete atomic support, requiring the use of a fallback
//!

pub use atomic_64::{AtomicI64, AtomicU64};
pub use atomic_ptr::AtomicUsize;
pub use core::sync::atomic::Ordering;

#[cfg(target_has_atomic = "64")]
use core::sync::atomic as atomic_64;
#[cfg(target_has_atomic = "ptr")]
use core::sync::atomic as atomic_ptr;
