use crate::{
    schedule::{InternedSystemSet, SystemSet, SystemTypeSet},
    system::{
        IntoSystem, System, SystemInput,
        exclusive_system_param::{ExclusiveSystemParam, ExclusiveSystemParamItem},
        fucntion_system::IntoResult,
    },
    world::World,
};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
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
            // param_state: None,
            // system_meta: SystemMeta::new::<F>(),
            marker: PhantomData,
        }
    }
}

impl<Marker, Out, F> System for ExclusiveFunctionSystem<Marker, Out, F>
where
    Marker: 'static,
    Out: 'static,
    F::Out: IntoResult<Out>,
    F: ExclusiveSystemParamFunction<Marker>,
{
    type In = F::In;
    type Out = Out;

    fn default_system_sets(&self) -> Vec<InternedSystemSet> {
        let set = SystemTypeSet::<Self>::new();
        vec![set.intern()]
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
        }

    };
}

all_tuples!(impl_exclusive_system_function, 0, 16, F);
