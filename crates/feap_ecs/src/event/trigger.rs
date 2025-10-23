use crate::{event::Event, observer::CachedObservers, world::DeferredWorld};

/// [`Trigger`] determines _how_ an [`Event`] is triggered when [`World::trigger`] is called.
/// This decides which [`Observer`]s will run, what data gets passed to them, and the order they will be executed in.
pub unsafe trait Trigger<E: Event> {
    unsafe fn trigger(
        &mut self,
        world: DeferredWorld,
        observers: &CachedObservers,
        trigger_context: &TriggerContext,
        event: &mut E,
    );
}

/// A [`Trigger`] that runs _every_ "global" [`Observer`] that matches the given [`Event`].
///
/// The [`Event`] derive defaults to using this [`Trigger`], and it is usable for any [`Event`] type.
#[derive(Default, Debug)]
pub struct GlobalTrigger;

unsafe impl<E: for<'a> Event<Trigger<'a> = Self>> Trigger<E> for GlobalTrigger {
    unsafe fn trigger(
        &mut self,
        _world: DeferredWorld,
        _observers: &CachedObservers,
        _trigger_context: &TriggerContext,
        _event: &mut E,
    ) {
        todo!()
    }
}

/// Metadata about a specific [`Event`] that triggered an observer
pub struct TriggerContext {}
