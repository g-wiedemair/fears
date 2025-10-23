//! Types for declaring and storing [`Component`]s

mod clone;
mod info;
mod register;
mod required;
mod tick;

pub use feap_ecs_macros::Component;
pub use info::*;
pub use register::*;
pub use tick::*;
pub use required::*;

use crate::{
    component::{clone::ComponentCloneBehavior, required::RequiredComponentsRegistrator},
    entity::EntityMapper,
    lifecycle::ComponentHook,
};

/// A data type that can be used to store data for an [`Entity`]
///
/// [`Component`] is a [derivable trait]: this means that a data type can implement it by applying a `#[derive(Component)]` attribute to it
/// However, components must always satisfy the `Send + Sync + 'static` trait bounds
///
/// # Component and data access
/// Components can be marked as immutable by adding th `#[component(immutable)]` attribute
///
/// # Choosing a storage type
/// Components can be stored in the world using different strategies with their own performance implications.
/// By default, components are added to the [`Table`] storage, which is optimized for query iteration.
/// Alternatively, components can be added to the [`SparseSet`] storage, which is optimized for component insertion and removal
/// This is achieved by adding an additional `#[component(storage = "SparseSet")]` attribute to the derive one.
///
/// # Setting the clone behavior
/// You can specify how the [`Component`] is cloned when deriving it
/// Your options are the functions and variants of [`ComponentCloneBehavior`]
///
/// # Implementing the trait for foreign types
///
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a `Component`",
    label = "invalid `Component`",
    note = "consider annotating `{Self}` with `#[derive(Component)]`"
)]
pub trait Component: Send + Sync + 'static {
    /// A constant indicating the storage type used for this component
    const STORAGE_TYPE: StorageType;

    /// A marker type to assist Feap with determining if this component is
    /// mutable, or immutable. Mutable components will have [`Component<Mutablility = Immutable>`]
    type Mutability: ComponentMutability;

    /// Gets the `on_add` [`ComponentHook`] for this [`Component`] if one is defined
    fn on_add() -> Option<ComponentHook> {
        None
    }

    /// Gets the `on_insert` [`ComponentHook`] for this [`Component`] if one is defined
    fn on_insert() -> Option<ComponentHook> {
        None
    }

    /// Gets the `on_replace` [`ComponentHook`] for this [`Component`] if one is defined.
    fn on_replace() -> Option<ComponentHook> {
        None
    }

    /// Gets the `on_remove` [`ComponentHook`] for this [`Component`] if one is defined.
    fn on_remove() -> Option<ComponentHook> {
        None
    }

    /// Gets the `on_despawn` [`ComponentHook`] for this [`Component`] if one is defined.
    fn on_despawn() -> Option<ComponentHook> {
        None
    }

    /// Reisters required components
    fn register_required_components(
        _component_id: ComponentId,
        _required_components: &mut RequiredComponentsRegistrator,
    ) {
    }

    /// Called when registering this component, allowing to override clone function (or disable cloning altogether) for this component
    #[inline]
    fn clone_behavior() -> ComponentCloneBehavior {
        ComponentCloneBehavior::Default
    }

    /// Maps the entities on this component using the given [`EntityMapper`]. This is used to remap entities in contexts like scenes and entity cloning.
    #[inline]
    fn map_entities<E: EntityMapper>(_this: &mut Self, _mapper: &mut E) {}
}

mod private {
    pub trait Seal {}
}

/// The mutability option for a [`Component::Mutability`] or `#[component(immutable)]`
/// when using the derive macro.
///
pub trait ComponentMutability: private::Seal + 'static {
    /// Boolean to indicate if this mutability setting implies a mutable or immutable component
    const MUTABLE: bool;
}

/// The storage used for a specific component type
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum StorageType {
    /// Provides fast and cache-friendly iteration, but slower addition and removal of components
    #[default]
    Table,
    /// Provides fast addition and removal of components, but slower iteration
    SparseSet,
}
