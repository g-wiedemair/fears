#![expect(
    clippy::module_inception,
    reason = "This instance of module inception is being discussed"
)]
mod executor;
mod schedule;
mod set;

use crate::{define_label, intern::Interned};
use executor::{MultiThreadedExecutor, SingleThreadedExecutor, SystemExecutor};

pub use executor::ExecutorKind;
pub use feap_ecs_macros::ScheduleLabel;
pub use schedule::*;
pub use set::*;

pub type InternedScheduleLabel = Interned<dyn ScheduleLabel>;

define_label!(
    /// A strongly-typed class of labels used to identify a [`Schedule`]
    ///
    /// Each schedule in a [`World`] has a unique schedule label value,
    /// and schedules can be automatically created from labels via [`Schedules::add_systems()`]
    ///
    #[diagnostic::on_unimplemented(
        note = "consider annotating `{Self}` with `#[derive(ScheduleLabel)]`"
    )]
    ScheduleLabel,
    SCHEDULE_LABEL_INTERNER
);
