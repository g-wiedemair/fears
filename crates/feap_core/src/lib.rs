#![no_std]

pub mod cfg;

cfg::std! {
    extern crate std;
}

cfg::alloc! {
    extern crate alloc;

    pub mod collections;
}

pub mod hash;
pub mod ptr;
pub mod sync;
