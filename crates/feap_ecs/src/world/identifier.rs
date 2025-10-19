use crate::world::{FromWorld, World};
use feap_core::sync::atomic::{AtomicUsize, Ordering};

/// A unique identifier for a [`World`]
///
/// The trait [`FromWorld`] is implemented for this type, which returns the
/// ID of the world passed to [`FromWorld`]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct WorldId(usize);

static MAX_WORLD_ID: AtomicUsize = AtomicUsize::new(0);

impl WorldId {
    /// Create a new, unique [`WorldId`]. 
    /// Returns [`None`] if the supply of unique [`WorldId`]s has been exhausted
    pub fn new() -> Option<Self> {
        MAX_WORLD_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                val.checked_add(1)
            })
            .map(WorldId)
            .ok()
    }
}

impl FromWorld for WorldId {
    #[inline]
    fn from_world(world: &mut World) -> Self {
        world.id
    }
}
