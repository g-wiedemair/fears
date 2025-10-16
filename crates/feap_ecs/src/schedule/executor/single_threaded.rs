use super::SystemExecutor;

/// Runs the schedule using a single thread
#[derive(Default)]
pub struct SingleThreadedExecutor {}

impl SystemExecutor for SingleThreadedExecutor {
    fn kind(&self) -> super::ExecutorKind {
        super::ExecutorKind::SingleThreaded
    }
}

impl SingleThreadedExecutor {
    /// Creates a new single-threaded executor for use in a [`Schedule`]
    pub const fn new() -> Self {
        Self {}
    }
}
