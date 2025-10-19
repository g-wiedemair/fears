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

    /// Has the plugin finished its setup? This can be useful for plugins that need something
    /// asynchronous to happen before they can finish their setup
    fn ready(&self, _app: &App) -> bool {
        true
    }
    
    /// Finish adding this plugin to the [`App`], once all plugins registered are ready
    fn finish(&self, _app: &mut App) {}
    
    /// Runs after all plugins are built and finished, but before the app schedule is executed
    fn cleanup(&self, _app: &mut App) {}

    /// Configures a name for the [`Plugin`] which is primarily used for checking plugin
    /// uniqueness and debugging
    fn name(&self) -> &str {
        core::any::type_name::<Self>()
    }

    /// If the plugin can be meaningfully instantiated several times in an [`App`],
    /// override this method to return `false`
    fn is_unique(&self) -> bool {
        true
    }
}

/// Plugins state in the application
#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum PluginsState {
    /// Plugins are being added
    Adding,
    /// All plugins already added are ready
    Ready,
    /// Finish has been executed for all plugins added
    Finished,
    /// Cleanup has been executed for all plugins added
    Cleaned,
}

/// A dummy plugin that's to temporarily occupy an entry in an app's plugin registry
pub(crate) struct PlaceholderPlugin;

impl Plugin for PlaceholderPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Types that represent a set of [`Plugin`]s
///
/// This is implemented for all types which implement [`Plugin`]
///
pub trait Plugins<Marker>: sealed::Plugins<Marker> {}

impl<Marker, T> Plugins<Marker> for T where T: sealed::Plugins<Marker> {}

mod sealed {
    use crate::{App, Plugin, app::AppError};

    pub trait Plugins<Marker> {
        fn add_to_app(self, app: &mut App);
    }

    pub struct PluginMarker;

    impl<P: Plugin> Plugins<PluginMarker> for P {
        #[track_caller]
        fn add_to_app(self, app: &mut App) {
            if let Err(AppError::DuplicatePlugin { plugin_name }) =
                app.add_boxed_plugin(Box::new(self))
            {
                panic!(
                    "Error adding plugin {plugin_name}: : plugin was already added in application"
                )
            }
        }
    }
}
