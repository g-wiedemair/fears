use crate::message::Messages;
use crate::{
    component::ComponentId, entity::Entity, message::Message, storage::sparse_set::SparseSet,
};
use core::fmt::Debug;
use derive_more::derive::Into;
use crate::world::DeferredWorld;

/// The type used for [`Component`] lifecycle hooks such as `on_add`, `on_insert` or `on_remove`
pub type ComponentHook = for<'w> fn(DeferredWorld<'w>, HookContext);

/// Context provided to a [`ComponentHook`]
#[derive(Clone, Copy, Debug)]
pub struct HookContext {
    
}

/// Wrapper around [`Entity`] for [`RemovedComponents`]
#[derive(Message, Debug, Clone, Into)]
pub struct RemovedComponentEntity(Entity);

/// Stores the [`RemovedComponents`] event buffers for all types of component in a given [`World`]
#[derive(Default, Debug)]
pub struct RemovedComponentMessages {
    event_sets: SparseSet<ComponentId, Messages<RemovedComponentEntity>>,
}

impl RemovedComponentMessages {
    /// For each type of component, swaps the event buffers and clears the oldest
    pub fn update(&mut self) {
        for (_component_id, messages) in self.event_sets.iter_mut() {
            todo!()
        }
    }
}
