use super::{
    ExecutorKind, InternedScheduleLabel, MultiThreadedExecutor, ScheduleLabel,
    SingleThreadedExecutor, SystemExecutor,
};
use crate::{component::ComponentId, resource::Resource};
use alloc::{boxed::Box, collections::BTreeSet};
use feap_core::collections::HashMap;

/// A collection of systems, and the metadata and executor needed to run them
/// in a certain order under certain conditions
///
/// Each schedule has a [`ScheduleLabel`] value. This value is used to uniquely identify the
/// schedule when added to a [`World`]s [`Schedules`], and may be used to specify which schedule
/// a system should be added to.
///
pub struct Schedule {
    label: InternedScheduleLabel,
    executor: Box<dyn SystemExecutor>,
    executor_initialized: bool,
}

impl Schedule {
    /// Constructs an empty [`Schedule`]
    pub fn new(label: impl ScheduleLabel) -> Self {
        Self {
            label: label.intern(),
            executor: make_executor(ExecutorKind::default()),
            executor_initialized: false,
        }
    }

    /// Sets the schedule's execution strategy
    pub fn set_executor_kind(&mut self, executor: ExecutorKind) -> &mut Self {
        if executor != self.executor.kind() {
            self.executor = make_executor(executor);
            self.executor_initialized = false;
        }
        self
    }
}

fn make_executor(kind: ExecutorKind) -> Box<dyn SystemExecutor> {
    match kind {
        ExecutorKind::SingleThreaded => Box::new(SingleThreadedExecutor::new()),
        #[cfg(feature = "std")]
        ExecutorKind::MultiThreaded => Box::new(MultiThreadedExecutor::new()),
    }
}

/// Resource that stores [`Schedule`]s mapped to [`ScheduleLabel`] excluding the current running [`Schedule`]
#[derive(Default, Resource)]
pub struct Schedules {
    inner: HashMap<InternedScheduleLabel, Schedule>,
    /// List of [`ComponentId`]s to ignore when reporting system order ambiguity conflicts
    pub ignored_scheduling_ambiguities: BTreeSet<ComponentId>,
}

impl Schedules {
    /// Inserts a labeled schedule into the map
    pub fn insert(&mut self, schedule: Schedule) -> Option<Schedule> {
        self.inner.insert(schedule.label, schedule)
    }
}
