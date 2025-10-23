#![no_std]
#![expect(
    unsafe_op_in_unsafe_fn,
    reason = "To be removed once all applicable unsafe code has an unsafe block with a safety comment."
)]

extern crate alloc;
extern crate self as feap_ecs;
#[cfg(feature = "std")]
extern crate std;

pub mod change_detection;
pub mod component;
mod entity;
mod error;
mod event;
pub mod intern;
pub mod label;
mod lifecycle;
mod message;
pub mod observer;
pub mod query;
mod relationship;
pub mod resource;
pub mod schedule;
pub mod storage;
pub mod system;
pub mod world;
