#![no_std]
#![expect(
    unsafe_op_in_unsafe_fn,
    reason = "To be removed once all applicable unsafe code has an unsafe block with a safety comment."
)]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

extern crate self as feap_ecs;

pub(crate) mod change_detection;
pub mod component;
pub mod intern;
pub mod label;
pub mod query;
pub mod resource;
pub mod schedule;
pub mod storage;
pub mod system;
pub mod world;
