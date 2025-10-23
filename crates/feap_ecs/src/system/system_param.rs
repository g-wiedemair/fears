use crate::{
    change_detection::{Res, ResMut},
    component::ComponentId,
    query::FilteredAccessSet,
    resource::Resource,
    system::fucntion_system::SystemMeta,
    world::{DeferredWorld, FromWorld, World},
};
use alloc::borrow::Cow;
use core::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use feap_core::cell::SyncCell;
use feap_utils::debug_info::DebugName;
use thiserror::Error;
use variadics_please::all_tuples;

/// A parameter that can be used in a [`System`]
///
/// This trait can be derived with the [`derive@super::SystemParam`] macro
/// This macro only works if each field on the derived struct implements [`SystemParam`]
///
pub unsafe trait SystemParam: Sized {
    /// Used to store data which persists across invocations of a system
    type State: Send + Sync + 'static;
    /// The item type returned when constructing this system param
    type Item<'world, 'state>: SystemParam<State = Self::State>;

    /// Creates a new instance of this param's [`State`]
    fn init_state(world: &mut World) -> Self::State;

    /// Registers any [`World`] access used by this [`SystemParam`]
    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        component_access_set: &mut FilteredAccessSet,
        world: &mut World,
    );
}

/// A [`SystemParam`] that only reads a given [`World`]
pub unsafe trait ReadOnlySystemParam: SystemParam {}

/// Shorthand way of accessing the associated type [`SystemParam::Item`]
pub type SystemParamItem<'w, 's, P> = <P as SystemParam>::Item<'w, 's>;

unsafe impl<'a, T: Resource> ReadOnlySystemParam for Res<'a, T> {}
unsafe impl<'a, T: Resource> SystemParam for Res<'a, T> {
    type State = ComponentId;
    type Item<'w, 's> = Res<'w, T>;

    fn init_state(world: &mut World) -> Self::State {
        todo!()
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        component_access_set: &mut FilteredAccessSet,
        world: &mut World,
    ) {
        todo!()
    }
}

unsafe impl<'a, T: Resource> SystemParam for ResMut<'a, T> {
    type State = ComponentId;
    type Item<'w, 's> = ResMut<'w, T>;

    fn init_state(world: &mut World) -> Self::State {
        todo!()
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        component_access_set: &mut FilteredAccessSet,
        world: &mut World,
    ) {
        todo!()
    }
}

unsafe impl ReadOnlySystemParam for &'_ World {}
unsafe impl SystemParam for &'_ World {
    type State = ();
    type Item<'w, 's> = &'w World;

    fn init_state(world: &mut World) -> Self::State {
        todo!()
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        component_access_set: &mut FilteredAccessSet,
        world: &mut World,
    ) {
        todo!()
    }
}

unsafe impl<'w> SystemParam for DeferredWorld<'w> {
    type State = ();
    type Item<'world, 'state> = DeferredWorld<'world>;

    fn init_state(world: &mut World) -> Self::State {
        todo!()
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        component_access_set: &mut FilteredAccessSet,
        world: &mut World,
    ) {
        todo!()
    }
}

/// A system local [`SystemParam`]
///
/// A local may only be accessed by the system itself and is therefore not visible to other systems
/// If two or more systems specify the same local type each will have their own unique local.
///
#[derive(Debug)]
pub struct Local<'s, T: FromWorld + Send + 'static>(pub(crate) &'s mut T);

unsafe impl<'s, T: FromWorld + Send + 'static> ReadOnlySystemParam for Local<'s, T> {}

impl<'s, T: FromWorld + Send + 'static> Deref for Local<'s, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'s, T: FromWorld + Send + 'static> DerefMut for Local<'s, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

unsafe impl<'a, T: FromWorld + Send + 'static> SystemParam for Local<'a, T> {
    type State = SyncCell<T>;
    type Item<'w, 's> = Local<'s, T>;

    fn init_state(world: &mut World) -> Self::State {
        todo!()
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        component_access_set: &mut FilteredAccessSet,
        world: &mut World,
    ) {
        todo!()
    }
}

macro_rules! impl_system_param_tuple {
    ($(#[$meta:meta])* $($param:ident),*) => {
        $(#[$meta])*
        unsafe impl<$($param: SystemParam),*> SystemParam for ($($param,)*) {
            type State = ($($param::State,)*);
            type Item<'w, 's> = ($($param::Item::<'w, 's>,)*);

            #[inline]
            fn init_state(world: &mut World) -> Self::State {
                (($($param::init_state(world),)*))
            }

            fn init_access(state: &Self::State, _system_meta: &mut SystemMeta, _component_access_set: &mut FilteredAccessSet, _world: &mut World) {
                let ($($param,)*) = state;
                $($param::init_access($param, _system_meta, _component_access_set, _world);)*
            }
        }
    };
}

all_tuples!(impl_system_param_tuple, 0, 16, P);

/// An error that occurs when a system parameter is not valid,
/// used by system executors to determine what to do with a system
#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub struct SystemParamValidationError {
    /// Whether the system should be skipped
    pub skipped: bool,
    /// A message describing the validation error
    pub message: Cow<'static, str>,
    /// A string identifying the invalid parameter
    pub param: DebugName,
    /// A string identifying the field within a parameter
    pub field: Cow<'static, str>,
}

impl Display for SystemParamValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // write!(
        //     f,
        //     "Parameter `{}{}` failed validation: {}",
        //     self.param.shortname(),
        //     self.field,
        //     self.message
        // )?;
        if !self.skipped {
            write!(
                f,
                "\nIf this is an expected state, wrap the parameter in `Option<T>` and handle `None`, or wrap the parameter in `If<T>` to skip the system when it happens."
            )?;
        }
        todo!()
    }
}
