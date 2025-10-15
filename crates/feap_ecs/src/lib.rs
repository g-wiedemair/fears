#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

extern crate self as feap_ecs;

pub mod intern;
pub mod label;
pub mod schedule;
