use crate::{
    Plugin, Plugins, SubApp, SubApps,
    main_schedule::{Main, MainSchedulePlugin},
    plugin::{PlaceholderPlugin, PluginsState},
};
use core::panic::AssertUnwindSafe;
use feap_core::collections::HashMap;
use feap_ecs::schedule::ScheduleLabel;

#[cfg(feature = "trace")]
use tracing::info_span;

#[cfg(feature = "std")]
use std::panic::{catch_unwind, resume_unwind};

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error("duplicate plugin {plugin_name:?}")]
    DuplicatePlugin { plugin_name: String },
}

/// [`App`] is the primary API for writing user applications. It automates the setup of a
/// [standard lifecycle](Main) and provides interface glue for [plugins](`Plugin`).
///
/// A single [`App`] can contain multiple [`SubApp`] instances, but [`App`] methods only affect
/// the "main" one.
///
pub struct App {
    pub(crate) sub_apps: SubApps,
    /// The function that will manage the app's lifecycle.
    pub(crate) runner: RunnerFn,
}

impl Default for App {
    fn default() -> Self {
        let mut app = App::empty();
        app.sub_apps.main.update_schedule = Some(Main.intern());
        app.add_plugins(MainSchedulePlugin);
        app
    }
}

impl App {
    /// Creates a new [`App`] with some default structure to enable core engine features
    /// ```
    pub fn new() -> App {
        App::default()
    }

    /// Creates a new empty [`App`] with minimal configuration
    ///
    pub fn empty() -> App {
        App {
            sub_apps: SubApps {
                main: SubApp::new(),
                sub_apps: HashMap::default(),
            },
            runner: Box::new(run_once),
        }
    }

    /// Returns a reference to the main [`SubApp`].
    pub fn main(&self) -> &SubApp {
        &self.sub_apps.main
    }

    /// Returns a reference to the main [`SubApp`]
    pub fn main_mut(&mut self) -> &mut SubApp {
        &mut self.sub_apps.main
    }

    /// Runs the [`App`], by calling its [runner].
    ///
    pub fn run(&mut self) {
        #[cfg(feature = "trace")]
        let _feap_app_run_span = info_span!("feap_app").entered();
        if self.is_building_plugins() {
            panic!("App::run() was called while a plugin was building.");
        }

        let runner = core::mem::replace(&mut self.runner, Box::new(run_once));
        let app = std::mem::take(self);
        (runner)(app);
    }

    /// Returns `true` if any of the sub-apps are building plugins
    pub(crate) fn is_building_plugins(&self) -> bool {
        self.sub_apps.iter().any(SubApp::is_building_plugins)
    }

    /// Returns the state of all plugins. This is usually called by the event loop, but can be
    /// useful for situations where you want to use [`App::update`]
    #[inline]
    pub fn plugins_state(&mut self) -> PluginsState {
        let mut overall_plugins_state = match self.main_mut().plugins_state {
            PluginsState::Adding => {
                let mut state = PluginsState::Ready;
                let plugins = core::mem::take(&mut self.main_mut().plugin_registry);
                for plugin in &plugins {
                    if !plugin.ready(self) {
                        state = PluginsState::Adding;
                        break;
                    }
                }
                self.main_mut().plugin_registry = plugins;
                state
            }
            state => state,
        };

        // Overall state is the earliest state of any sup-app
        self.sub_apps.iter_mut().skip(1).for_each(|s| {
            overall_plugins_state = overall_plugins_state.min(s.plugins_state());
        });

        overall_plugins_state
    }

    /// Installs a [`Plugin`] collection
    ///
    /// Feap prioritizes modularity as a core principle.
    /// All features are implemented as plugins, even the complex ones like rendering.
    ///
    /// [`Plugin`]s can be grouped into a set by using a [`PluginGroup`].
    ///
    #[track_caller]
    pub fn add_plugins<M>(&mut self, plugins: impl Plugins<M>) -> &mut Self {
        if matches!(
            self.plugins_state(),
            PluginsState::Cleaned | PluginsState::Finished
        ) {
            panic!(
                "Plugins cannot be added after App::cleanup() or App::finish() has been called."
            );
        }
        plugins.add_to_app(self);
        self
    }

    pub(crate) fn add_boxed_plugin(
        &mut self,
        plugin: Box<dyn Plugin>,
    ) -> Result<&mut Self, AppError> {
        log::debug!("added plugin: {}", plugin.name());
        if plugin.is_unique() && self.main_mut().plugin_names.contains(plugin.name()) {
            Err(AppError::DuplicatePlugin {
                plugin_name: plugin.name().to_string(),
            })?;
        }

        // Reserve position in the plugin registry.
        let index = self.main().plugin_registry.len();
        self.main_mut()
            .plugin_registry
            .push(Box::new(PlaceholderPlugin));

        self.main_mut().plugin_build_depth += 1;

        #[cfg(feature = "trace")]
        let _plugin_build_span = info_span!("plugin build", plugin = plugin.name()).entered();

        let f = AssertUnwindSafe(|| plugin.build(self));

        #[cfg(feature = "std")]
        let result = catch_unwind(f);
        #[cfg(not(feature = "std"))]
        f();

        self.main_mut()
            .plugin_names
            .insert(plugin.name().to_string());
        self.main_mut().plugin_build_depth -= 1;

        #[cfg(feature = "std")]
        if let Err(payload) = result {
            resume_unwind(payload);
        }

        self.main_mut().plugin_registry[index] = plugin;
        Ok(self)
    }
}

type RunnerFn = Box<dyn FnOnce(App) -> AppExit>;

fn run_once(_app: App) -> AppExit {
    // while app.plugins_state() == PluginsState::Adding {
    //     #[cfg(not(target_arch = "wasm32"))]
    //     feap_tasks::tick_global_task_pools_on_main_thread();
    // }

    // app.finish();
    // app.cleanup();

    // app.update();

    // app.should_exit().unwrap_or(AppExit::Success)

    println!("App run_once ..., fix me.");
    AppExit::Success
}

/// A [`BufferedEvent`] that indicates the [`App`] should exit.
///
pub enum AppExit {
    /// [`App`] exited successfully.
    Success,
}
