use super::input::{SystemIn, SystemInput};
use crate::{
    query::FilteredAccessSet,
    schedule::InternedSystemSet,
    system::{system_param::SystemParamValidationError, RunSystemError},
    world::World,
};
use alloc::{boxed::Box, vec::Vec};
use bitflags::bitflags;
use core::any::TypeId;
use feap_ecs::world::UnsafeWorldCell;
use feap_utils::debug_info::DebugName;

bitflags! {
    /// Bitflags representing system states and requirements
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SystemStateFlags: u8 {
        /// Set if system cannot be sent across threads
        const NON_SEND = 1 << 0;
        /// Set if system requires exclusive World access
        const EXCLUSIVE = 1 << 1;
        /// Set if system has deferred buffers
        const DEFERRED = 1 << 2;
    }
}

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

    /// Returns the system's name
    fn name(&self) -> DebugName;

    /// Returns the [`TypeId`] of the underlying system type
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Initialize the system
    /// Returns a [`FilteredAccessSet`] with the access required to run the system
    fn initialize(&mut self, _world: &mut World) -> FilteredAccessSet;

    /// Returns the system's default [`SystemSet`]
    fn default_system_sets(&self) -> Vec<InternedSystemSet>;

    /// Runs the system with the given input in the world.
    /// Unlike [`System::run`], this will not apply deferred parameters
    unsafe fn run_unsafe(
        &mut self,
        input: SystemIn<'_, Self>,
        world: UnsafeWorldCell,
    ) -> Result<Self::Out, RunSystemError>;

    /// Runs the system with the given input in the world
    fn run_without_applying_deferred(
        &mut self,
        input: SystemIn<'_, Self>,
        world: &mut World,
    ) -> Result<Self::Out, RunSystemError> {
        let world_cell = world.as_unsafe_world_cell();
        unsafe { self.validate_param_unsafe(world_cell) }?;
        unsafe { self.run_unsafe(input, world_cell) }
    }

    /// Applies any [`Deferred`] system parameters
    /// This is where [`Commands`] are applied
    fn apply_deferred(&mut self, world: &mut World);

    /// Validates that all parameters can be acquired and that system can run without panic
    /// Built-in executors use this to prevent invalid systems from running
    unsafe fn validate_param_unsafe(
        &mut self,
        world: UnsafeWorldCell,
    ) -> Result<(), SystemParamValidationError>;
}

/// A convenience type alias for a boxed [`System`] trait object
pub type BoxedSystem<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;

/// [`System`] types that do not modify the [`World`] when run
/// This is implemented for any systems whose parameters all implement [`ReadOnlySystemParam`]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a read-only system",
    label = "invalid read-only system"
)]
pub unsafe trait ReadOnlySystem: System {}
