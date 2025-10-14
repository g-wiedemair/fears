use crate::App;
use core::any::Any;
use downcast_rs::Downcast;

/// A collection of feap app logic and configuration
///
/// Plugins configure an [`App`]. When an [`App`] registers a plugin,
/// the plugin's [`Plugin::build`] function is run. By default, a plugin
/// can only be added once to an [`App`].
///
pub trait Plugin: Downcast + Any + Send + Sync {
    /// Configures the [`App`] to which this plugin is added
    fn build(&self, app: &mut App);
}

/// Types that represent a set of [`Plugin`]s
///
/// This is implemented for all types which implement [`Plugin`]
///
pub trait Plugins<Marker>: sealed::Plugins<Marker> {}

impl<Marker, T> Plugins<Marker> for T where T: sealed::Plugins<Marker> {}

mod sealed {
    use crate::App;

    pub trait Plugins<Marker> {
        fn add_to_app(self, app: &mut App);
    }
}
