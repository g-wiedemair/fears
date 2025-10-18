use super::input::SystemInput;
use alloc::{boxed::Box, vec::Vec};
use crate::schedule::InternedSystemSet;

/// An ECS system that can be added to a [`Schedule`]
///
/// Systems are functions with all arguments implementing [`SystemParam`]
/// Systems are added to an application using `App::add_systems(Update, my_system)`
/// or similar methods, and will generally run once per pass of the main loop
///
/// Systems are executed in parallel, in opportunistic order; data access is managed automatically
/// It's possible to specify explicit execution order between specific systems
#[diagnostic::on_unimplemented(message = "`{Self}` is not a system", label = "invalid_system")]
pub trait System: Send + Sync + 'static {
    /// The System's input
    type In: SystemInput;
    /// The System's output
    type Out;
    
    /// Returns the system's default [`SystemSet`]
    fn default_system_sets(&self) -> Vec<InternedSystemSet>;
}

/// A convenience type alias for a boxed [`System`] trait object
pub type BoxedSystem<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;

/// [`System`] types that do not modify the [`World`] when run
/// This is implemented for any systems whose parameters all implement [`ReadOnlySystemParam`]
#[diagnostic::on_unimplemented(message = "`{Self}` is not a read-only system", label = "invalid read-only system")]
pub unsafe trait ReadOnlySystem: System {}
