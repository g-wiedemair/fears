use super::{IntoSystem, System, SystemInput, SystemParam, SystemParamItem};
use crate::schedule::{InternedSystemSet, SystemSet, SystemTypeSet};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use variadics_please::all_tuples;

/// The [`System`] counterpart of an ordinary function
///
/// You get this by calling [`IntoSystem::into_system`] on a function that only accepts
/// [`SystemParam`]s. The output of the system becomes the function return type, while the
/// input becomes the functions first parameter or `()` if no such parameter exists
pub struct FunctionSystem<Marker, Out, F>
where
    F: SystemParamFunction<Marker>,
{
    func: F,
    marker: PhantomData<fn() -> (Marker, Out)>,
}

impl<Marker, Out, F> System for FunctionSystem<Marker, Out, F>
where
    Marker: 'static,
    Out: 'static,
    F: SystemParamFunction<Marker, Out: IntoResult<Out>>,
{
    type In = F::In;
    type Out = Out;

    fn default_system_sets(&self) -> Vec<InternedSystemSet> {
        let set = SystemTypeSet::<Self>::new();
        vec![set.intern()]
    }
}

/// A marker type used to distinguish regular function systems from exclusive function systems
#[doc(hidden)]
pub struct IsFunctionSystem;

impl<Marker, Out, F> IntoSystem<F::In, Out, (IsFunctionSystem, Marker)> for F
where
    Out: 'static,
    Marker: 'static,
    F: SystemParamFunction<Marker, Out: IntoResult<Out>>,
{
    type System = FunctionSystem<Marker, Out, F>;

    fn into_system(func: Self) -> Self::System {
        FunctionSystem {
            func,
            marker: PhantomData,
        }
    }
}

/// A trait implemented for all functions that can be used as [`System`]s
///
/// This trait can be useful for making your own systems which accept other systems,
/// sometimes called higher order systems
///
/// This should be used in combination with [`ParamSet`] when calling other systems
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid system",
    label = "invalid system"
)]
pub trait SystemParamFunction<Marker>: Send + Sync + 'static {
    /// The input type of this system
    type In: SystemInput;
    /// The return type of this system
    type Out;
    /// The [`SystemParam`]s used by this system to access the [`World`]
    type Param: SystemParam;
}

/// A marker type used to distinguish function systems with and without input
#[doc(hidden)]
pub struct HasSystemInput;

macro_rules! impl_system_function {
    ($($param:ident),*) => {
      #[expect(
          clippy::allow_attributes,
          reason = "This is within a macro, and as such, the below lints may not always apply."
      )]
        #[allow(
          non_snake_case,
          reason = "Certain variable names are provided by the caller, not by us."
      )]
        impl<Out, Func, $($param: SystemParam),*> SystemParamFunction<fn($($param,)*) -> Out> for Func
        where
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func:
                FnMut($($param),*) -> Out +
                FnMut($(SystemParamItem<$param>),*) -> Out,
            Out: 'static
        {
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
        impl<In, Out, Func, $($param: SystemParam),*> SystemParamFunction<(HasSystemInput, fn(In, $($param,)*) -> Out)> for Func
        where
            Func: Send + Sync + 'static,
            for <'a> &'a mut Func:
                FnMut(In, $($param),*) -> Out +
                FnMut(In::Param<'_>, $(SystemParamItem<$param>),*) -> Out,
            In: SystemInput + 'static,
            Out: 'static
        {
            type In = In;
            type Out = Out;
            type Param = ($($param,)*);
        }
    };
}

all_tuples!(impl_system_function, 0, 16, F);

/// A type that may be converted to the output of a [`System`]
/// This is used to allow systems to return either a plain value or a [`Result`]
pub trait IntoResult<Out>: Sized {}

impl<T> IntoResult<T> for T {}
