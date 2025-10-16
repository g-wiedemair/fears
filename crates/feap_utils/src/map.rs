use core::any::TypeId;
use feap_core::{collections::HashMap, hash::NoOpHash};

/// A specialized hashmap type with Key of [`TypeId`]
/// Iteration order only depends on the order of insertions and deletions
pub type TypeIdMap<V> = HashMap<TypeId, V, NoOpHash>;
