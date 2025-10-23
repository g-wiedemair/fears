use crate::system::input::SystemIn;
use crate::system::system_param::SystemParamValidationError;
use crate::system::RunSystemError;
use crate::world::UnsafeWorldCell;
use crate::{
    component::Tick,
    query::FilteredAccessSet,
    schedule::{InternedSystemSet, SystemSet, SystemTypeSet},
    system::{
        exclusive_system_param::{ExclusiveSystemParam, ExclusiveSystemParamItem}, fucntion_system::{IntoResult, SystemMeta}, IntoSystem,
        System,
        SystemInput,
    },
    world::World,
};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use feap_utils::debug_info::DebugName;
use variadics_please::all_tuples;

/// A function system that runs with exclusive [`World`] access
///
/// You get this by calling [`IntoSystem::into_system`] on a function that only accepts
/// [`ExclusiveSystemParam`]s
pub struct ExclusiveFunctionSystem<Marker, Out, F>
where
    F: ExclusiveSystemParamFunction<Marker>,
{
    func: F,
    param_state: Option<<F::Param as ExclusiveSystemParam>::State>,
    system_meta: SystemMeta,
    marker: PhantomData<fn() -> (Marker, Out)>,
}

/// A marker type used to distinguish exclusive function systems from regular function systems
#[doc(hidden)]
pub struct IxExclusiveFunctionSystem;

impl<Out, Marker, F> IntoSystem<F::In, Out, (IxExclusiveFunctionSystem, Marker, Out)> for F
where
    Out: 'static,
    Marker: 'static,
    F::Out: IntoResult<Out>,
    F: ExclusiveSystemParamFunction<Marker>,
{
    type System = ExclusiveFunctionSystem<Marker, Out, F>;

    fn into_system(func: Self) -> Self::System {
        ExclusiveFunctionSystem {
            func,
            param_state: None,
            system_meta: SystemMeta::new::<F>(),
            marker: PhantomData,
        }
    }
}

const PARAM_MESSAGE: &str = "System's param_state was not found. Did you forget to initialize this system before running it?";

impl<Marker, Out, F> System for ExclusiveFunctionSystem<Marker, Out, F>
where
    Marker: 'static,
    Out: 'static,
    F::Out: IntoResult<Out>,
    F: ExclusiveSystemParamFunction<Marker>,
{
    type In = F::In;
    type Out = Out;

    #[inline]
    fn name(&self) -> DebugName {
        self.system_meta.name.clone()
    }

    #[inline]
    fn initialize(&mut self, world: &mut World) -> FilteredAccessSet {
        self.system_meta.last_run = world.change_tick().relative_to(Tick::MAX);
        self.param_state = Some(F::Param::init(world, &mut self.system_meta));
        FilteredAccessSet::new()
    }

    fn default_system_sets(&self) -> Vec<InternedSystemSet> {
        let set = SystemTypeSet::<Self>::new();
        vec![set.intern()]
    }

    unsafe fn run_unsafe(
        &mut self,
        input: SystemIn<'_, Self>,
        world: UnsafeWorldCell,
    ) -> Result<Self::Out, RunSystemError> {
        let world = unsafe { world.world_mut() };
        world.last_change_tick_scope(self.system_meta.last_run, |world| {
            #[cfg(feature = "trace")]
            let _span_guard = self.system_meta.system_span.enter();

            let params = F::Param::get_param(
                self.param_state.as_mut().expect(PARAM_MESSAGE),
                &self.system_meta,
            );

            let out = self.func.run(world, input, params);

            world.flush();
            self.system_meta.last_run = world.increment_change_tick();

            IntoResult::into_result(out)
        })
    }

    fn apply_deferred(&mut self, _world: &mut World) {
        // exclusive systems do not have any buffers to apply
    }

    unsafe fn validate_param_unsafe(
        &mut self,
        _world: UnsafeWorldCell,
    ) -> Result<(), SystemParamValidationError> {
        // All exclusive system params are always available
        Ok(())
    }
}

/// A trait implemented for all exclusive system functions that can be used as [`System`]s
///
/// This trait can be useful for making your own systems which accept other systems,
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an exclusive system",
    label = "invalid system"
)]
pub trait ExclusiveSystemParamFunction<Marker>: Send + Sync + 'static {
    /// The input type to this system
    type In: SystemInput;
    /// The return type of this system
    type Out;
    /// The [ÈxclusiveSystemParam`]s defined by this system's `fn` parameters
    type Param: ExclusiveSystemParam;

    /// Executes this system once
    fn run(
        &mut self,
        world: &mut World,
        input: <Self::In as SystemInput>::Inner<'_>,
        param_value: ExclusiveSystemParamItem<Self::Param>,
    ) -> Self::Out;
}

/// A marker type used to distinguish exclusive function systems with and without input.
#[doc(hidden)]
pub struct HasExclusiveSystemInput;

macro_rules! impl_exclusive_system_function {
    ($($param: ident),*) => {
        #[expect(
            clippy::allow_attributes,
            reason = "This is within a macro, and as such, the below lints may not always apply."
        )]
        #[allow(
            non_snake_case,
            reason = "Certain variable names are provided by the caller, not by us."
        )]
        impl<Out, Func, $($param: ExclusiveSystemParam),*> ExclusiveSystemParamFunction<fn($($param,)*) -> Out> for Func
        where
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func:
                FnMut(&mut World, $($param),*) -> Out +
                FnMut(&mut World, $(ExclusiveSystemParamItem<$param>),*) -> Out,
            Out: 'static, {
            type In = ();
            type Out = Out;
            type Param = ($($param,)*);

            #[inline]
            fn run(&mut self, world: &mut World, _in: (), param_value: ExclusiveSystemParamItem< ($($param,)*)>) -> Out {
                fn call_inner<Out, $($param,)*>(
                    mut f: impl FnMut(&mut World, $($param,)*) -> Out,
                    world: &mut World,
                    $($param: $param,)*
                ) -> Out {
                    f(world, $($param,)*)
                }
                let ($($param,)*) = param_value;
                call_inner(self, world, $($param),*)
            }
        }

        #[expect(
            clippy::allow_attributes,
            reason = "This is within a macro, and as such, the below lints may not always apply."
        )]
        #[allow(
            non_snake_case,
            reason = "Certain variable names are provided by the caller, not by us."
        )]
        impl<In, Out, Func, $($param: ExclusiveSystemParam),*> ExclusiveSystemParamFunction<(HasExclusiveSystemInput, fn(In, $($param,)*) -> Out)> for Func
        where
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func:
                FnMut(In, &mut World, $($param),*) -> Out +
                FnMut(In::Param<'_>, &mut World, $(ExclusiveSystemParamItem<$param>),*) -> Out,
            In: SystemInput + 'static,
            Out: 'static,
        {
            type In = In;
            type Out = Out;
            type Param = ($($param,)*);

            #[inline]
            fn run(&mut self, world: &mut World, input: In::Inner<'_>, param_value: ExclusiveSystemParamItem< ($($param,)*)>) -> Out {
                todo!()
            }
        }
    };
}

all_tuples!(impl_exclusive_system_function, 0, 16, F);
