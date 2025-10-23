use crate::Plugin;
use feap_ecs::{
    change_detection::Mut,
    resource::Resource,
    schedule::{ExecutorKind, InternedScheduleLabel, Schedule, ScheduleLabel, SystemSet},
    system::Local,
    world::World,
};

/// The schedule that contains the app logic that is evaluated each tick of [`App::update()`]
///
/// By default, it will run the following schedules in the given order:
/// On the first run of the schedule (and only on the first run), it will run:
/// * [`PreStartup`]
/// * [`Startup`]
/// * [`PostStartup`]
///
/// Then it will run:
/// * [`First`]
/// * [`PreUpdate`]
/// * [`StateTransition`]
/// * [`RunFixedMainLoop`]
///   * This will run [`FixedMain`] zero to many times, based on how much time has elapsed
/// * [`Update`]
/// * [`PostUpdate`]
/// * [`Last`]
///
/// Note: Rendering is not executed in the main schedule by default.
/// Instead, rendering is performed in a separate [`SubApp`]
/// which exchanges data with the main app in between the main schedule runs.
///
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Main;

impl Main {
    /// A system that runs the "main schedule"
    pub fn run_main(world: &mut World, mut run_at_least_once: Local<bool>) {
        if !*run_at_least_once {
            world.resource_scope(|world, order: Mut<MainScheduleOrder>| {
                for &label in &order.startup_labels {
                    let _ = world.try_run_schedule(label);
                }
            });
            *run_at_least_once = true;
        }

        world.resource_scope(|world, order: Mut<MainScheduleOrder>| {
            for &label in &order.labels {
                let _ = world.try_run_schedule(label);
            }
        });
    }
}

/// The schedule that runs before [`Startup`]
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PreStartup;

/// The schedule that runs once when the app starts
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Startup;

/// The schedule that runs after [`Startup`]
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PostStartup;

/// Runs first in the schedule
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct First;

/// The schedule that contains logic that must run before [`Update`].
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PreUpdate;

/// The schedule that contains any logic that must run once per render frame
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Update;

/// The schedule that contains scene spawning
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct SpawnScene;

/// The schedule that contains logic that must run after [`Update`].
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PostUpdate;

/// The schedule that runs last
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Last;

/// Runs the [`FixedMain`] schedule in a loop according until all relevant elapsed time has been "consumed"
///
/// Note that in contrast to most other Feap schedules, systems added directly to
/// [`RunFixedMainLoop`] will *NOT* be parallelized between each other
///
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct RunFixedMainLoop;

/// The schedule that contains systems which only run after a fixed period of time has elapsed
///
/// This is run by the [`RunFixedMainLoop`] schedule.
///
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FixedMain;

impl FixedMain {
    /// A system that runs the fixed timestep's "main schedule"
    pub fn run_fixed_main(_world: &mut World) {
        todo!()
    }
}

/// Set enum for the systems that want to run inside [`RunFixedMainLoop`]
/// but before or after the fixed update logic. Systems in this set
/// will run exactly once per frame, regardless of the number of fixed updates.
/// They will also run under a variable timestep.
///
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, SystemSet)]
pub enum RunFixedMainLoopSystems {
    /// Runs before the fixed update logic
    BeforeFixedMainLoop,
    /// Contains the fixed update logic
    FixedMainLoop,
    /// Runs after the fixed update logic
    AfterFixedMainLoop,
}

/// Runs first in the [`FixedMain`] schedule
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FixedFirst;

/// The schedule that contains logic that must run before [`FixedUpdate`].
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FixedPreUpdate;

/// The schedule that contains most logic, which runs at a fixed rate rather than every render frame
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FixedUpdate;

/// The schedule that runs after the  [`FixedUpdate`] schedule, for reacting to changes made in the main update logic.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FixedPostUpdate;

/// The schedule that runs last in [`FixedMain`].
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FixedLast;

/// Defines the schedules to be run for the [`Main`] schedule, including their order
#[derive(Resource, Debug)]
pub struct MainScheduleOrder {
    /// The labels to run for the main phase of the [`Main`] schedule (in the order they will be run)
    pub labels: Vec<InternedScheduleLabel>,
    /// The labels to run for the startup phase of the [`Main`] schedule (in the order they will be run)
    pub startup_labels: Vec<InternedScheduleLabel>,
}

impl Default for MainScheduleOrder {
    fn default() -> Self {
        Self {
            labels: vec![
                First.intern(),
                PreUpdate.intern(),
                RunFixedMainLoop.intern(),
                Update.intern(),
                SpawnScene.intern(),
                PostUpdate.intern(),
                Last.intern(),
            ],
            startup_labels: vec![PreStartup.intern(), Startup.intern(), PostStartup.intern()],
        }
    }
}

/// Defines the schedules to be run for the [`FixedMain`] schedule, including their order
#[derive(Resource, Debug)]
pub struct FixedMainScheduleOrder {
    /// The labels to run for the [`FixedMain`] schedule (in the order they will be run
    pub labels: Vec<InternedScheduleLabel>,
}

impl Default for FixedMainScheduleOrder {
    fn default() -> Self {
        Self {
            labels: vec![
                FixedFirst.intern(),
                FixedPreUpdate.intern(),
                FixedUpdate.intern(),
                FixedPostUpdate.intern(),
                FixedLast.intern(),
            ],
        }
    }
}

/// Initializes the [`Main`] schedule, sub schedules, and resources for a given [`App`]
pub struct MainSchedulePlugin;

impl Plugin for MainSchedulePlugin {
    fn build(&self, app: &mut crate::App) {
        // Simple "facilitator" schedules benefit from simpler single threaded scheduling
        let mut main_schedule = Schedule::new(Main);
        main_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        let mut fixed_main_schedule = Schedule::new(FixedMain);
        fixed_main_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        let mut fixed_main_loop_schedule = Schedule::new(RunFixedMainLoop);
        fixed_main_loop_schedule.set_executor_kind(ExecutorKind::SingleThreaded);

        app.add_schedule(main_schedule)
            .add_schedule(fixed_main_schedule)
            .add_schedule(fixed_main_loop_schedule)
            .init_resource::<MainScheduleOrder>()
            .init_resource::<FixedMainScheduleOrder>()
            .add_systems(Main, Main::run_main);
        // .add_systems(FixedMain, FixedMain::run_fixed_main)
        // .configure_sets(
        //     RunFixedMainLoop,
        //     (
        //         RunFixedMainLoopSystems::BeforeFixedMainLoop,
        //         RunFixedMainLoopSystems::FixedMainLoop,
        //         RunFixedMainLoopSystems::AfterFixedMainLoop,
        //     )
        //         .chain(),
        // );
    }
}
