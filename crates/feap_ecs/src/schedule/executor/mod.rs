mod multi_threaded;
mod single_threaded;

pub(super) use multi_threaded::*;
pub(super) use single_threaded::*;

use crate::schedule::node::{ConditionWithAccess, SystemKey, SystemSetKey, SystemWithAccess};
use alloc::vec::Vec;

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
            set_ids: Vec::new(),
            set_conditions: Vec::new(),
        }
    }
}
