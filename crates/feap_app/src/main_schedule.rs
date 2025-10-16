use crate::Plugin;
use feap_ecs::{
    schedule::{ExecutorKind, Schedule, ScheduleLabel, SystemSet},
    system::Local,
    world::World,
};

/// The schedule that contains the app logic that is evaluated each tick of [`App::update()`]
///
/// By default, it will run the following schedules in the given order:
/// On the first run of the schedule (and only on the first run), it will run:
///
/// Then it will run:
///
/// Note: Rendering is not executed in the main schedule by default.
/// Instead, rendering is performed in a separate [`SubApp`]
/// which exchanges data with the main app in between the main schedule runs.
///
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Main;

impl Main {
    /// A system that runs the "main schedule"
    pub fn run_main(_world: &mut World, _run_at_least_once: Local<bool>) {}
}

/// The schedule that contains systems which only run after a fixed period of time has elapsed
///
/// This is run by the [`RunFixedMainLoop`] schedule.
///
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FixedMain;

/// Runs the [`FixedMain`] schedule in a loop according until all relevant elapsed time has been "consumed"
///
/// Note that in contrast to most other Feap schedules, systems added directly to
/// [`RunFixedMainLoop`] will *NOT* be parallelized between each other
///
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct RunFixedMainLoop;

impl FixedMain {
    /// A system that runs the fixed timestep's "main schedule"
    pub fn run_fixed_main(_world: &mut World) {}
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
            // .add_systems(Main, Main::run_main)
            // .add_systems(FixedMain, FixedMain::run_fixed_main)
            // .configure_sets(
            //     RunFixedMainLoop,
            //     (
            //         RunFixedMainLoopSystems::BeforeFixedMainLoop,
            //         RunFixedMainLoopSystems::FixedMainLoop,
            //         RunFixedMainLoopSystems::AfterFixedMainLoop,
            //     )
            //         .chain(),
            // )
            ;
    }
}
