use super::SystemExecutor;

/// Runs the schedule using a single thread
#[derive(Default)]
pub struct MultiThreadedExecutor {}

impl SystemExecutor for MultiThreadedExecutor {
    fn kind(&self) -> super::ExecutorKind {
        super::ExecutorKind::MultiThreaded
    }
}

impl MultiThreadedExecutor {
    /// Creates a new single-threaded executor for use in a [`Schedule`]
    pub const fn new() -> Self {
        Self {}
    }
}
