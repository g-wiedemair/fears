use crate::Plugin;
use feap_ecs::schedule::ScheduleLabel;

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

/// Initializes the [`Main`] schedule, sub schedules, and resources for a given [`App`]
pub struct MainSchedulePlugin;

impl Plugin for MainSchedulePlugin {
    fn build(&self, _app: &mut crate::App) {}
}
