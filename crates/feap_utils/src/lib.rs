//! General utilities for first-party engine crates
//!

pub mod debug_info;
pub mod map;

cfg::std! {
    extern crate std;
}

cfg::alloc! {
    extern crate alloc;
}

/// Configuration information for this crate
pub mod cfg {
    pub(crate) use feap_core::cfg::*;

    pub use feap_core::cfg::{alloc, std};
}

use core::mem::ManuallyDrop;

/// A type which calls a function when dropped.
/// This can be used to ensure that cleanup code is run even in case of a panic
pub struct OnDrop<F: FnOnce()> {
    callback: ManuallyDrop<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    /// Returns an object that will invoke the specified callback when dropped
    pub fn new(callback: F) -> Self {
        Self {
            callback: ManuallyDrop::new(callback),
        }
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        #![expect(
            unsafe_code,
            reason = "Taking from a ManuallyDrop requires unsafe code."
        )]
        let callback = unsafe { ManuallyDrop::take(&mut self.callback) };
        callback();
    }
}
