use super::system::BoxedSystem;

/// Type alias for a [`BoxedSystem`] that a [`Schedule`] can store
pub type ScheduleSystem = BoxedSystem<(), ()>;
