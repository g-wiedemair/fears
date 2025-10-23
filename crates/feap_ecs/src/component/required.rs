use crate::component::ComponentsRegistrator;

/// The collection of metadata for components that are required for a given component
#[derive(Default, Clone)]
pub struct RequiredComponents {}

/// This is a safe handle around `ComponentsRegistrator` and `RequiredComponents` to register required components
pub struct RequiredComponentsRegistrator<'a, 'w> {
    components: &'a mut ComponentsRegistrator<'w>,
    required_components: &'a mut RequiredComponents,
}
