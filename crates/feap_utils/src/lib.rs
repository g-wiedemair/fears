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
