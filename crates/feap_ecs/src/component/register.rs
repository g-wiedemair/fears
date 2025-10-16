use super::{ComponentDescriptor, ComponentId, Components};
use crate::resource::Resource;
use alloc::vec::Vec;
use core::{any::TypeId, fmt::Debug, ops::Deref};
use feap_core::sync::PoisonError;
use feap_utils::map::TypeIdMap;

/// Generates [`ComponentId`]s
#[derive(Debug, Default)]
pub struct ComponentIds {
    next: feap_core::sync::atomic::AtomicUsize,
}

impl ComponentIds {
    /// Generates and returns the next [`ComponentId`]
    pub fn next_mut(&mut self) -> ComponentId {
        let id = self.next.get_mut();
        let result = ComponentId(*id);
        *id += 1;
        result
    }
}

/// A [`Components`] wrapper that enables additional features, like registration
pub struct ComponentsRegistrator<'w> {
    pub(super) components: &'w mut Components,
    pub(super) ids: &'w mut ComponentIds,
    pub(super) recursion_check_stack: Vec<ComponentId>,
}

impl Deref for ComponentsRegistrator<'_> {
    type Target = Components;

    fn deref(&self) -> &Self::Target {
        self.components
    }
}

impl<'w> ComponentsRegistrator<'w> {
    /// Construct a new [`ComponentsRegistrator`]
    pub unsafe fn new(components: &'w mut Components, ids: &'w mut ComponentIds) -> Self {
        Self {
            components,
            ids,
            recursion_check_stack: Vec::new(),
        }
    }

    /// Registers a [`Resource`] of type `T` with this instance.
    /// If a resource of this type has already been registered, this will return
    /// the ID of the pre-existing resource
    #[inline]
    pub fn register_resource<T: Resource>(&mut self) -> ComponentId {
        unsafe {
            self.register_resource_with(TypeId::of::<T>(), || {
                ComponentDescriptor::new_resource::<T>()
            })
        }
    }

    /// Same as [`Components::register_resource_unchecked`] but handles safety
    #[inline]
    unsafe fn register_resource_with(
        &mut self,
        type_id: TypeId,
        descriptor: impl FnOnce() -> ComponentDescriptor,
    ) -> ComponentId {
        if let Some(id) = self.resource_indices.get(&type_id) {
            return *id;
        }

        if let Some(_registrator) = self
            .components
            .queued
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .resources
            .remove(&type_id)
        {
            todo!()
        }

        let id = self.ids.next_mut();
        unsafe {
            self.components
                .register_resource_unchecked(type_id, id, descriptor());
        }
        id
    }

    /// Applies every queued registration
    pub fn apply_queued_registrations(&mut self) {
        if !self.any_queued_mut() {
            return;
        }

        todo!()
    }

    /// Equivalent of `Components::any_queued_mut`
    pub fn any_queued_mut(&mut self) -> bool {
        self.components.any_queued_mut()
    }
}

/// A queued component registration
pub(super) struct QueuedRegistration {
    pub(super) id: ComponentId,
}

/// Allows queuing components to be registered
#[derive(Default)]
pub struct QueuedComponents {
    pub(super) components: TypeIdMap<QueuedRegistration>,
    pub(super) resources: TypeIdMap<QueuedRegistration>,
    pub(super) dynamic_registrations: Vec<QueuedRegistration>,
}

impl Debug for QueuedComponents {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let components = self
            .components
            .iter()
            .map(|(type_id, queued)| (type_id, queued.id))
            .collect::<Vec<_>>();
        let resources = self
            .resources
            .iter()
            .map(|(type_id, queued)| (type_id, queued.id))
            .collect::<Vec<_>>();
        let dynamic_registrations = self
            .dynamic_registrations
            .iter()
            .map(|queued| queued.id)
            .collect::<Vec<_>>();
        write!(
            f,
            "components: {components:?}, resources: {resources:?}, dynamic_registrations: {dynamic_registrations:?}"
        )
    }
}
