mod multi_threaded;
mod single_threaded;

pub(super) use multi_threaded::*;
pub(super) use single_threaded::*;

use crate::{
    error::{ErrorContext, FeapError},
    schedule::node::{ConditionWithAccess, SystemKey, SystemSetKey, SystemWithAccess},
    system::System,
    world::World,
};
use alloc::vec::Vec;
use core::any::TypeId;
use fixedbitset::FixedBitSet;

/// Specifies how a [`Schedule`] will be run
/// The default depends on the target platform
#[derive(PartialEq, Eq, Default, Debug, Copy, Clone)]
pub enum ExecutorKind {
    #[cfg_attr(
        any(
            target_arch = "wasm32",
            not(feature = "std"),
            not(feature = "multi_threaded"),
        ),
        default
    )]
    SingleThreaded,
    #[cfg(feature = "std")]
    #[cfg_attr(all(not(target_arch = "wasm32"), feature = "multi_threaded"), default)]
    MultiThreaded,
}

/// Types that can run a [`SystemSchedule`] on a [`World`]
pub(super) trait SystemExecutor: Send + Sync {
    fn kind(&self) -> ExecutorKind;
    fn init(&mut self, schedule: &SystemSchedule);
    fn run(
        &mut self,
        schedule: &mut SystemSchedule,
        world: &mut World,
        skip_systems: Option<&FixedBitSet>,
        error_handler: fn(FeapError, ErrorContext),
    );
}

/// Holds systems and conditions of a [`Schedule`] sorted in topological order
/// (along with dependency information for `multi-threaded` execution).
///
/// Since the arrays are sorted in the same order, elements are referenced by their index
/// [`FixedBitSet`] is used as a smaller, more efficient substitute of `HashSet<usize>`
#[derive(Default)]
pub struct SystemSchedule {
    /// List of system node ids
    pub(super) system_ids: Vec<SystemKey>,
    /// Indexed by system node id
    pub(super) systems: Vec<SystemWithAccess>,
    /// Indexed by system node id
    pub(super) system_conditions: Vec<Vec<ConditionWithAccess>>,
    /// Indexed by system node ids
    pub(super) sets_with_conditions_of_systems: Vec<FixedBitSet>,
    /// List of system set node ids
    pub(super) set_ids: Vec<SystemSetKey>,
    /// Indexed by system set node id
    pub(super) set_conditions: Vec<Vec<ConditionWithAccess>>,
}

impl SystemSchedule {
    /// Creates an empty [`SystemSchedule`]
    pub const fn new() -> Self {
        Self {
            system_ids: Vec::new(),
            systems: Vec::new(),
            system_conditions: Vec::new(),
            sets_with_conditions_of_systems: Vec::new(),
            set_ids: Vec::new(),
            set_conditions: Vec::new(),
        }
    }
}

/// A special [`System`] that instructs the executor to call [`System::apply_deferred`] on the systems
/// that have run but not applied their [`Deferred`] system parameters or other system buffers
pub struct ApplyDeferred;

/// Returns `true` if the [`System`] is an instance of [`ApplyDeferred`]
pub(super) fn is_apply_deferred(system: &dyn System<In = (), Out = ()>) -> bool {
    system.type_id() == TypeId::of::<ApplyDeferred>()
}

/// These functions hide the bottom of the callstack from `RUST_BACKTRACE=1`
/// The full callstack will still be visible with `RUST_BACKTRACE=full`
mod __rust_begin_short_backtrace {
    use crate::{
        system::{ReadOnlySystem, RunSystemError, ScheduleSystem},
        world::World,
    };
    use core::hint::black_box;

    #[inline(never)]
    pub(super) fn run_without_applying_deferred(
        system: &mut ScheduleSystem,
        world: &mut World,
    ) -> Result<(), RunSystemError> {
        let result = system.run_without_applying_deferred((), world);
        black_box(());
        result
    }

    #[inline(never)]
    pub(super) fn readonly_run<O: 'static>(
        system: &mut dyn ReadOnlySystem<In = (), Out = O>,
        world: &mut World,
    ) -> Result<O, RunSystemError> {
        todo!()
    }
}
