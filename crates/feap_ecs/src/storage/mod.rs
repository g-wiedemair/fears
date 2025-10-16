//! Storage layouts for ECS data
//!
//! This module implements the low-level collections that store data in a [`World`].
//! These all offers minimal and often unsafe APIs, and have been made `pub` primarily for debugging

pub(crate) mod blob_array;
mod resource;
pub(crate) mod sparse_set;

pub(crate) use resource::{ResourceData, Resources};

/// The raw data stores of a [`World`]
#[derive(Default)]
pub struct Storages {
    /// Backing storage for resources
    pub resources: Resources<true>,
}
