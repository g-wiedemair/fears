mod multi_threaded;
mod single_threaded;

pub(super) use multi_threaded::*;
pub(super) use single_threaded::*;

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
