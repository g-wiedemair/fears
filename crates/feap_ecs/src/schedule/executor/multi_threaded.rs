use fixedbitset::FixedBitSet;
use crate::{
    error::{ErrorContext, FeapError},
    world::World
};
use super::{ExecutorKind, SystemExecutor, SystemSchedule};

/// Runs the schedule using a single thread
#[derive(Default)]
pub struct MultiThreadedExecutor {}

impl SystemExecutor for MultiThreadedExecutor {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::MultiThreaded
    }

    fn init(&mut self, schedule: &SystemSchedule) {
        todo!()
    }

    fn run(&mut self, schedule: &mut SystemSchedule, world: &mut World, skip_systems: Option<&FixedBitSet>, error_handler: fn(FeapError, ErrorContext)) {
        todo!()
    }
}

impl MultiThreadedExecutor {
    /// Creates a new single-threaded executor for use in a [`Schedule`]
    pub const fn new() -> Self {
        Self {}
    }
}
