use alloc::boxed::Box;
use crate::system::ReadOnlySystem;

/// A type-erased run condition stored in a [`Box`]
pub type BoxedCondition<In = ()> = Box<dyn ReadOnlySystem<In = In, Out = bool>>;
