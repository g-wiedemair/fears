use super::info::ComponentInfo;
use feap_core::ptr::Ptr;

/// Provides read access to the source component (the component being cloned) in a [`ComponentCloneFn`]
pub struct SourceComponent<'a> {
    ptr: Ptr<'a>,
    info: &'a ComponentInfo,
}

/// Context for component clone handlers
/// Provides fast access to useful resources and allows component clone handler to get information
pub struct ComponentCloneCtx {}

/// Function type that can be used to clone a component of an entity.
pub type ComponentCloneFn = fn(&SourceComponent, &mut ComponentCloneCtx);

/// The clone behavior to use when cloning or moving a [`Component`]
#[derive(Clone, Debug, Default)]
pub enum ComponentCloneBehavior {
    /// Uses the default behavior (which is passed to [`ComponentCloneBehavior::resolve`])
    #[default]
    Default,
    /// Do not clone/move this component
    Ignore,
    /// Uses a custom [`ComponentCloneFn`]
    Custom(ComponentCloneFn),
}
