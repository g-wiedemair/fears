//! Provides various atomic alternatives to language primitives
//!
//! Certain platforms lack complete atomic support, requiring the use of a fallback
//!

pub use atomic_ptr::AtomicUsize;

#[cfg(target_has_atomic = "ptr")]
use core::sync::atomic as atomic_ptr;
