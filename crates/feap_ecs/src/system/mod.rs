mod exclusive_function_system;
mod exclusive_system_param;
mod fucntion_system;
mod input;
mod schedule_system;
mod system;
mod system_param;

pub use input::SystemInput;
pub use schedule_system::ScheduleSystem;
pub use system::{BoxedSystem, ReadOnlySystem, System};
pub use system_param::{Local, SystemParam, SystemParamItem};

/// Conversion trait to turn something into a [`System`]
/// Use this to get a system from a function. Also note that every system implements this as well
pub trait IntoSystem<In: SystemInput, Out, Marker>: Sized {
    /// The type of [`System`] that this instance converts into
    type System: System<In = In, Out = Out>;

    /// Turns this value into its corresponding [`System`]
    fn into_system(this: Self) -> Self::System;
}

// All systems implicitly implements IntoSystem
impl<T: System> IntoSystem<T::In, T::Out, ()> for T {
    type System = T;

    fn into_system(this: Self) -> Self {
        this
    }
}
