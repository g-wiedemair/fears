use super::{
    ExecutorKind, InternedScheduleLabel, InternedSystemSet, IntoScheduleConfigs,
    MultiThreadedExecutor, ScheduleGraph, ScheduleLabel, SingleThreadedExecutor, SystemExecutor,
    error::{ScheduleBuildError, ScheduleBuildWarning}, executor::SystemSchedule,
};
use crate::{component::ComponentId, resource::Resource, system::ScheduleSystem, world::World};
use alloc::{boxed::Box, vec::Vec, collections::BTreeSet};
use core::any::Any;
use feap_core::collections::HashMap;
use feap_utils::map::TypeIdMap;

/// A collection of systems, and the metadata and executor needed to run them
/// in a certain order under certain conditions
///
/// Each schedule has a [`ScheduleLabel`] value. This value is used to uniquely identify the
/// schedule when added to a [`World`]s [`Schedules`], and may be used to specify which schedule
/// a system should be added to.
///
pub struct Schedule {
    label: InternedScheduleLabel,
    graph: ScheduleGraph,
    executable: SystemSchedule,
    executor: Box<dyn SystemExecutor>,
    executor_initialized: bool,
    warnings: Vec<ScheduleBuildWarning>
}

impl Schedule {
    /// Constructs an empty [`Schedule`]
    pub fn new(label: impl ScheduleLabel) -> Self {
        Self {
            label: label.intern(),
            graph: ScheduleGraph::new(),
            executable: SystemSchedule::new(),
            executor: make_executor(ExecutorKind::default()),
            executor_initialized: false,
            warnings: Vec::new(),
        }
    }

    /// Returns the [`ScheduleGraph`]
    pub fn graph(&self) -> &ScheduleGraph {
        &self.graph
    }

    /// Sets the schedule's execution strategy
    pub fn set_executor_kind(&mut self, executor: ExecutorKind) -> &mut Self {
        if executor != self.executor.kind() {
            self.executor = make_executor(executor);
            self.executor_initialized = false;
        }
        self
    }

    /// Add a collection of systems to the schedule
    pub fn add_systems<M>(
        &mut self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.graph.process_configs(systems.into_configs(), false);
        self
    }

    /// Configures a collection of system sets in this schedule, adding them if they does not exist
    #[track_caller]
    pub fn configure_sets<M>(
        &mut self,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) -> &mut Self {
        self.graph.configure_sets(sets);
        self
    }

    /// Runs all systems in this schedule on the `world`, using its current execution strategy
    pub fn run(&mut self, world: &mut World) {
        #[cfg(feature = "trace")]
        let _span = info_span!("schedule", name = ?self.label).entered();

        world.check_change_ticks();
        self.initialize(world).unwrap_or_else(|e| {
            panic!(
                "Error when initializing schedule {:?}: {}",
                self.label,
                e.to_string(self.graph(), world)
            )
        });

        todo!()
    }

    /// Initializes any newly-added systems and conditions, rebuilds the executable schedule,
    /// and re-initializes the executor
    /// Moves all systems and run conditions out of the [`ScheduleGraph`]
    pub fn initialize(&mut self, world: &mut World) -> Result<(), ScheduleBuildError> {
        if self.graph.changed {
            self.graph.initialize(world);
            let ignored_ambiguities = world
                .get_resource_or_init::<Schedules>()
                .ignored_scheduling_ambiguities
                .clone();
            self.warnings = self.graph.update_schedule(
                world,
                &mut self.executable,
                &ignored_ambiguities,
                self.label,
            )?;
            todo!()
        }

        todo!()
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

    /// Removes the schedule corresponding to the `label` from the map, returning it if it existed
    pub fn remove(&mut self, label: impl ScheduleLabel) -> Option<Schedule> {
        self.inner.remove(&label.intern())
    }

    /// a mutable reference to the schedules associated with `label`, creating one if it doesn't exist
    pub fn entry(&mut self, label: impl ScheduleLabel) -> &mut Schedule {
        self.inner
            .entry(label.intern())
            .or_insert_with(|| Schedule::new(label))
    }

    /// Adds one or more systems to the [`Schedule`] matching the provided [`ScheduleLabel`]
    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.entry(schedule).add_systems(systems);
        self
    }

    /// Configures a collection of system sets in the provided schedule, adding any sets that do not exist
    #[track_caller]
    pub fn configure_sets<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) -> &mut Self {
        self.entry(schedule).configure_sets(sets);
        self
    }
}

/// Chain systems into dependencies
#[derive(Default)]
pub enum Chain {
    /// Systems are independent. Nodes are allowed to run in any order
    #[default]
    Unchained,
    /// Systems are chained. `before -> after` ordering constraints
    /// will be added between the successive elements
    Chained(TypeIdMap<Box<dyn Any>>),
}

impl Chain {
    /// Specify that the systems must be chained
    pub fn set_chained(&mut self) {
        if matches!(self, Chain::Unchained) {
            *self = Self::Chained(Default::default());
        };
    }
}
