mod trigger;

pub use self::trigger::*;
pub use feap_ecs_macros::Event;

use crate::{component::Component, world::World};
use core::marker::PhantomData;
use crate::component::ComponentId;

/// An  [`Event`] is something that "happens" at a given moment
/// To make an [`Event`] happen, you "trigger" it on a [`World`]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an `Event`",
    label = "invalid `Event`",
    note = "consider annotating `{Self}` with `#[derive(Event)]`"
)]
pub trait Event: Send + Sync + Sized + 'static {
    /// Defines which observers will run, what data will be passed to them, and the order they will be run in.
    type Trigger<'a>: Trigger<Self>;
}

impl World {
    /// Generates the [`EventKey`] for this event type
    /// If this type has already been registered, this will return the existing [`EventKey`]
    pub fn register_event_key<E: Event>(&mut self) -> EventKey {
        EventKey(self.register_component::<EventWrapperComponent<E>>())
    }
}

/// An internal type that implements [`Component`] for a given [` Event`] type
#[derive(Component)]
struct EventWrapperComponent<E: Event>(PhantomData<E>);

/// A unique identifier for an [`Event`], used b< [observers]
#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct EventKey(pub(crate) ComponentId);
