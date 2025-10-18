use crate::{App, Plugin, plugin::PluginsState};
use feap_core::collections::{HashMap, HashSet};
use feap_ecs::resource::Resource;
use feap_ecs::schedule::InternedSystemSet;
use feap_ecs::world::FromWorld;
use feap_ecs::{
    intern::Interned,
    schedule::{InternedScheduleLabel, IntoScheduleConfigs, Schedule, ScheduleLabel, Schedules},
    system::ScheduleSystem,
    world::World,
};

feap_ecs::define_label!(
    /// A strongly-typed class of labels used to identify an [`App`]
    #[diagnostic::on_unimplemented(
        note = "consider annotating `{Self}` with `#[derive(AppLabel)]`"
    )]
    AppLabel,
    APP_LABEL_INTERNER
);

/// A shorthand for `Interned<dyn AppLabel>`
pub type InternedAppLabel = Interned<dyn AppLabel>;

/// A secondary application with its own [`World`]. These can run independently of each other
///
/// These are useful for situations where certain processes (e.g. a render thread) need to be kept
/// separate from the main application
///
pub struct SubApp {
    /// The data of this application
    world: World,
    /// List of plugins that have been added
    pub(crate) plugin_registry: Vec<Box<dyn Plugin>>,
    /// The names of plugins that have been added to this app.
    pub(crate) plugin_names: HashSet<String>,
    /// Panics if an update is attempted while plugins are building
    pub(crate) plugin_build_depth: usize,
    pub(crate) plugins_state: PluginsState,
    /// The schedule that will be run by [`update`]
    pub update_schedule: Option<InternedScheduleLabel>,
}

impl Default for SubApp {
    fn default() -> Self {
        let mut world = World::new();
        world.init_resource::<Schedules>();
        Self {
            world,
            plugin_registry: Vec::default(),
            plugin_names: HashSet::default(),
            plugin_build_depth: 0,
            plugins_state: PluginsState::Adding,
            update_schedule: None,
        }
    }
}

impl SubApp {
    /// Returns a default, empty [`SubApp`]
    pub fn new() -> Self {
        Self::default()
    }

    /// This method is a workaround.
    /// Each [`SubApp`] can have its own plugins, but [`Plugin`] works on an [`App`] as a whole
    fn run_as_app<F>(&mut self, f: F)
    where
        F: FnOnce(&mut App),
    {
        let mut app = App::empty();
        core::mem::swap(self, &mut app.sub_apps.main);
        f(&mut app);
        core::mem::swap(self, &mut app.sub_apps.main);
    }

    /// Returns `true` if there is no plugin in the middle of being built.
    pub(crate) fn is_building_plugins(&self) -> bool {
        self.plugin_build_depth > 0
    }

    /// Returns the state of plugins
    #[inline]
    pub fn plugins_state(&mut self) -> PluginsState {
        match self.plugins_state {
            PluginsState::Adding => {
                let mut state = PluginsState::Ready;
                let plugins = core::mem::take(&mut self.plugin_registry);
                self.run_as_app(|app| {
                    for plugin in &plugins {
                        if !plugin.ready(app) {
                            state = PluginsState::Adding;
                            return;
                        }
                    }
                });
                state
            }
            state => state,
        }
    }

    pub fn add_schedule(&mut self, schedule: Schedule) -> &mut Self {
        let mut schedules = self.world.resource_mut::<Schedules>();
        schedules.insert(schedule);
        self
    }

    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        let mut schedules = self.world.resource_mut::<Schedules>();
        schedules.add_systems(schedule, systems);
        self
    }

    #[track_caller]
    pub fn configure_sets<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) -> &mut Self {
        let mut schedules = self.world.resource_mut::<Schedules>();
        schedules.configure_sets(schedule, sets);
        self
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }
}

/// The collection of sub-apps that belong to an [`App`]
#[derive(Default)]
pub struct SubApps {
    /// The primary sub-app that contains the "main" world
    pub main: SubApp,
    /// Other, labeled sub-apps
    pub sub_apps: HashMap<InternedAppLabel, SubApp>,
}

impl SubApps {
    /// Returns an iterator over the sub-apps (starting with the main one)
    pub fn iter(&self) -> impl Iterator<Item = &SubApp> + '_ {
        core::iter::once(&self.main).chain(self.sub_apps.values())
    }

    /// Returns a mutable iterator over the sup-apps (starting with the main one)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SubApp> + '_ {
        core::iter::once(&mut self.main).chain(self.sub_apps.values_mut())
    }
}
