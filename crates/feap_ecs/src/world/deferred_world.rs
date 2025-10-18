use crate::world::UnsafeWorldCell;

/// A [`World`] reference that disallows structural ECS changes
/// This includes initializing resources, registering components or spawning entities
pub struct DeferredWorld<'w> {
    world: UnsafeWorldCell<'w>,
}
