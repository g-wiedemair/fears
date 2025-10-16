use super::{StorageType, clone::ComponentCloneBehavior};
use crate::{
    component::QueuedComponents, query::DebugCheckedUnwrap, resource::Resource,
    storage::sparse_set::SparseSetIndex,
};
use alloc::vec::Vec;
use core::{alloc::Layout, any::TypeId, fmt::Debug, mem::needs_drop};
use feap_core::{ptr::OwningPtr, sync::PoisonError, sync::RwLock};
use feap_utils::{debug_info::DebugName, map::TypeIdMap};

/// Stores metadata for a type of component or resource stored in a specific [`World`]
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub(super) id: ComponentId,
    pub(super) descriptor: ComponentDescriptor,
}

impl ComponentInfo {
    /// Creates a new [`ComponentInfo`]
    pub(crate) fn new(id: ComponentId, descriptor: ComponentDescriptor) -> Self {
        ComponentInfo { id, descriptor }
    }

    /// Returns the name of the current component.
    #[inline]
    pub fn name(&self) -> DebugName {
        self.descriptor.name.clone()
    }

    /// Returns the layout used to store values of this component in memory.
    #[inline]
    pub fn layout(&self) -> Layout {
        self.descriptor.layout
    }

    /// Get the function which should be called to clean up values of
    /// the underlying component type. This maps to the
    /// [`Drop`] implementation for 'normal' Rust components
    ///
    /// Returns `None` if values of the underlying component type don't
    /// need to be dropped, e.g. as reported by [`needs_drop`].
    #[inline]
    pub fn drop(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
        self.descriptor.drop
    }

    /// Returns `true` if the underlying component type can be freely shared between threads
    #[inline]
    pub fn is_send_and_sync(&self) -> bool {
        self.descriptor.is_send_and_sync
    }
}

/// A value which uniquely identifies the type of [`Component`] or [`Resource`] within a [`World`]
///
/// Each time a new `Component` type is registered within a `World` a corresponding `ComponentId` is created to track it.
///
#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ComponentId(pub(super) usize);

impl ComponentId {
    /// Returns the index of the current Component
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

impl SparseSetIndex for ComponentId {
    #[inline]
    fn sparse_set_index(&self) -> usize {
        self.index()
    }

    #[inline]
    fn get_sparse_set_index(value: usize) -> Self {
        Self(value)
    }
}

/// A value describing a component or resource, which may or may not correspond to a Rust type
#[derive(Clone)]
pub struct ComponentDescriptor {
    name: DebugName,
    storage_type: StorageType,
    is_send_and_sync: bool,
    type_id: Option<TypeId>,
    layout: Layout,
    drop: Option<for<'a> unsafe fn(OwningPtr<'a>)>,
    mutable: bool,
    clone_behavior: ComponentCloneBehavior,
}

impl Debug for ComponentDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ComponentDescriptor")
            .field("name", &self.name)
            .field("storage_type", &self.storage_type)
            .field("is_send_and_sync", &self.is_send_and_sync)
            .field("type_id", &self.type_id)
            .field("layout", &self.layout)
            .field("mutable", &self.mutable)
            .field("clone_behavior", &self.clone_behavior)
            .finish()
    }
}

impl ComponentDescriptor {
    unsafe fn drop_ptr<T>(x: OwningPtr<'_>) {
        unsafe {
            x.drop_as::<T>();
        }
    }

    /// Create a new `ComponentDescriptor` for a resource
    /// The [`StorageType`] for a resource is always [`StorageType::Table`]
    pub fn new_resource<T: Resource>() -> Self {
        Self {
            name: DebugName::type_name::<T>(),
            storage_type: StorageType::Table,
            is_send_and_sync: true,
            type_id: Some(TypeId::of::<T>()),
            layout: Layout::new::<T>(),
            drop: needs_drop::<T>().then_some(Self::drop_ptr::<T> as _),
            mutable: true,
            clone_behavior: ComponentCloneBehavior::Default,
        }
    }
}

/// Stores metadata associated with each kind of [`Component`] in a given [`World`]
#[derive(Debug, Default)]
pub struct Components {
    pub(super) components: Vec<Option<ComponentInfo>>,
    pub(super) resource_indices: TypeIdMap<ComponentId>,
    // This is kept internal and local to verify that no deadlocks can occur
    pub(super) queued: RwLock<QueuedComponents>,
}

impl Components {
    /// This registers any descriptor, component or resource
    #[inline]
    pub(super) unsafe fn register_component_inner(
        &mut self,
        id: ComponentId,
        descriptor: ComponentDescriptor,
    ) {
        let info = ComponentInfo::new(id, descriptor);
        let least_len = id.0 + 1;
        if self.components.len() < least_len {
            self.components.resize_with(least_len, || None);
        }
        let slot = unsafe { self.components.get_mut(id.0).debug_checked_unwrap() };
        debug_assert!(slot.is_none());
        *slot = Some(info);
    }

    #[inline]
    pub(super) unsafe fn register_resource_unchecked(
        &mut self,
        type_id: TypeId,
        component_id: ComponentId,
        descriptor: ComponentDescriptor,
    ) {
        unsafe {
            self.register_component_inner(component_id, descriptor);
        }
        let prev = self.resource_indices.insert(type_id, component_id);
        debug_assert!(prev.is_none());
    }

    /// Gets the metadata associated with the given component, if it is registered
    #[inline]
    pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        self.components.get(id.0).and_then(|info| info.as_ref())
    }

    /// Type-erased equivalent of [`Components::valid_resource_id()`]
    #[inline]
    pub fn get_valid_resource_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.resource_indices.get(&type_id).copied()
    }

    /// A faster version of [`Self::any_queued`]
    #[inline]
    pub fn any_queued_mut(&mut self) -> bool {
        self.num_queued_mut() > 0
    }

    /// A faster version of [`Self::num_queued`]
    #[inline]
    pub fn num_queued_mut(&mut self) -> usize {
        let queued = self
            .queued
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner);
        queued.components.len() + queued.dynamic_registrations.len() + queued.resources.len()
    }
}
