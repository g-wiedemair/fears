use super::{ExecutorKind, SystemExecutor, SystemSchedule};
use crate::{
    error::{ErrorContext, ErrorHandler, FeapError},
    schedule::node::ConditionWithAccess,
    system::{RunSystemError, ScheduleSystem},
    world::World,
};
use core::panic::AssertUnwindSafe;
use fixedbitset::FixedBitSet;

/// Runs the schedule using a single thread
#[derive(Default)]
pub struct SingleThreadedExecutor {
    /// System sets whose conditions have been evaluated
    evaluated_sets: FixedBitSet,
    /// Systems that have run or been skipped
    completed_systems: FixedBitSet,
    /// Systems that have run but have not had their buffers applied
    unapplied_systems: FixedBitSet,
    /// Setting when true applies deferred system buffers after all systems have run
    apply_final_deferred: bool,
}

impl SystemExecutor for SingleThreadedExecutor {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::SingleThreaded
    }

    fn init(&mut self, schedule: &SystemSchedule) {
        // Pre-allocate space
        let sys_count = schedule.system_ids.len();
        let set_count = schedule.set_ids.len();
        self.evaluated_sets = FixedBitSet::with_capacity(set_count);
        self.completed_systems = FixedBitSet::with_capacity(sys_count);
        self.unapplied_systems = FixedBitSet::with_capacity(sys_count);
    }

    fn run(
        &mut self,
        schedule: &mut SystemSchedule,
        world: &mut World,
        _skip_systems: Option<&FixedBitSet>,
        error_handler: fn(FeapError, ErrorContext),
    ) {
        // If stepping is enabled, make sure we skip those systems that should not be run
        #[cfg(feature = "feap_debug_stepping")]
        if let Some(skipped_systems) = _skip_systems {
            todo!()
        }

        for system_index in 0..schedule.systems.len() {
            let system = &mut schedule.systems[system_index].system;

            #[cfg(feature = "trace")]
            let name = system.name();
            #[cfg(feature = "trace")]
            let should_run_span = info_span!("check_conditions", name = name.as_string()).entered();

            let mut should_run = !self.completed_systems.contains(system_index);
            for set_idx in schedule.sets_with_conditions_of_systems[system_index].ones() {
                todo!()
            }

            // Evaluate system's conditions
            let system_conditions_met = evaluate_and_fold_conditions(
                &mut schedule.system_conditions[system_index],
                world,
                error_handler,
                system,
                false,
            );

            should_run &= system_conditions_met;

            #[cfg(feature = "trace")]
            should_run_span.exit();

            // System has either been skipped or will run
            self.completed_systems.insert(system_index);

            if !should_run {
                continue;
            }

            if super::is_apply_deferred(&**system) {
                todo!()
            }

            let f = AssertUnwindSafe(|| {
                if let Err(RunSystemError::Failed(err)) =
                    super::__rust_begin_short_backtrace::run_without_applying_deferred(
                        system, world,
                    )
                {
                    todo!()
                }
            });

            #[cfg(feature = "std")]
            #[expect(clippy::print_stderr, reason = "Allowed behind `std` feature gate.")]
            {
                if let Err(payload) = std::panic::catch_unwind(f) {
                    std::eprintln!("Encountered a panic in system `{}`!", system.name());
                    std::panic::resume_unwind(payload);
                }
            }
            #[cfg(not(feature = "std"))]
            {
                (f)();
            }

            self.unapplied_systems.insert(system_index);
        }

        if self.apply_final_deferred {
            self.apply_deferred(schedule, world);
        }
        self.evaluated_sets.clear();
        self.completed_systems.clear();
    }
}

impl SingleThreadedExecutor {
    /// Creates a new single-threaded executor for use in a [`Schedule`]
    pub const fn new() -> Self {
        Self {
            evaluated_sets: FixedBitSet::new(),
            completed_systems: FixedBitSet::new(),
            unapplied_systems: FixedBitSet::new(),
            apply_final_deferred: true,
        }
    }

    fn apply_deferred(&mut self, schedule: &mut SystemSchedule, world: &mut World) {
        for system_index in self.unapplied_systems.ones() {
            let system = &mut schedule.systems[system_index].system;
            system.apply_deferred(world);
        }

        self.unapplied_systems.clear();
    }
}

fn evaluate_and_fold_conditions(
    conditions: &mut [ConditionWithAccess],
    world: &mut World,
    error_handler: ErrorHandler,
    for_system: &ScheduleSystem,
    on_set: bool,
) -> bool {
    #[expect(
        clippy::unnecessary_fold,
        reason = "Short-circuiting here would prevent conditions from mutating their own state as needed."
    )]
    conditions
        .iter_mut()
        .map(|ConditionWithAccess { condition, .. }| {
            super::__rust_begin_short_backtrace::readonly_run(&mut **condition, world)
                .unwrap_or_else(|err| todo!())
        })
        .fold(true, |acc, res| acc && res)
}
