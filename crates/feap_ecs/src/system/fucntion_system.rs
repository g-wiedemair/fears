use super::{IntoSystem, SystemStateFlags, System, SystemInput, SystemParam, SystemParamItem};
use crate::{
    component::Tick,
    query::FilteredAccessSet,
    schedule::{InternedSystemSet, SystemSet, SystemTypeSet},
    world::{World, WorldId},
};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use feap_utils::debug_info::DebugName;
use variadics_please::all_tuples;

/// The metadata of a [`System`]
#[derive(Clone)]
pub struct SystemMeta {
    pub(crate) name: DebugName,
    flags: SystemStateFlags,
    pub(crate) last_run: Tick,
    #[cfg(feature = "trace")]
    pub(crate) system_span: Span,
    #[cfg(feature = "trace")]
    pub(crate) commands_span: Span,
}

impl SystemMeta {
    pub(crate) fn new<T>() -> Self {
        let name = DebugName::type_name::<T>();
        Self {
            #[cfg(feature = "trace")]
            system_span: info_span!(parent: None, "system", name = name.clone().as_string()),
            #[cfg(feature = "trace")]
            commands_span: info_span!(parent: None, "system_commands", name = name.clone().as_string()),
            name,
            flags: SystemStateFlags::empty(),
            last_run: Tick::new(0),
        }
    }
}

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
    state: Option<FunctionSystemState<F::Param>>,
    system_meta: SystemMeta,
    marker: PhantomData<fn() -> (Marker, Out)>,
}

/// The state of a [`FunctionSystem`], which must be initialized with [`System::initialize`]
/// before the system can be run. A panic will occur if the system is run without being initialized
struct FunctionSystemState<P: SystemParam> {
    /// The cached state of the system's [`SystemParam`]s
    param: P::State,
    /// The id of the [`World`] this system was initialized with.
    /// If the world passed to [`System::run_unsafe`] does not match this id, a panic will occur
    world_id: WorldId,
}

impl<Marker, Out, F> System for FunctionSystem<Marker, Out, F>
where
    Marker: 'static,
    Out: 'static,
    F: SystemParamFunction<Marker, Out: IntoResult<Out>>,
{
    type In = F::In;
    type Out = Out;

    #[inline]
    fn initialize(&mut self, world: &mut World) -> FilteredAccessSet {
        if let Some(state) = &self.state {
            assert_eq!(
                state.world_id,
                world.id(),
                "System built with a different world than the one it was added to."
            );
        }
        let state = self.state.get_or_insert_with(|| FunctionSystemState {
            param: F::Param::init_state(world),
            world_id: world.id(),
        });
        self.system_meta.last_run = world.change_tick().relative_to(Tick::MAX);
        let mut component_access_set = FilteredAccessSet::new();
        F::Param::init_access(
            &state.param,
            &mut self.system_meta,
            &mut component_access_set,
            world,
        );
        component_access_set
    }

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
            state: None,
            system_meta: SystemMeta::new::<F>(),
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
