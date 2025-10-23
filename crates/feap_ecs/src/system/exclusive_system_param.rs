use crate::system::Local;
use crate::world::FromWorld;
use crate::{system::fucntion_system::SystemMeta, world::World};
use feap_core::cell::SyncCell;
use variadics_please::all_tuples;

/// A parameter that can be used in an exclusive system (a system with an `&mut World` parameter
/// Any parameters implementing this trait must come after the `& mut World` parameter
#[diagnostic::on_unimplemented(
    message = "`{Self}` can not be used as a parameter for an exclusive system",
    label = "invalid system parameter"
)]
pub trait ExclusiveSystemParam: Sized {
    /// Used to store data which persists across invocations of a system
    type State: Send + Sync + 'static;
    /// The item type returned when constructing this system param
    type Item<'s>: ExclusiveSystemParam<State = Self::State>;

    /// Creates a new instance of this param's [`State`]
    fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self::State;

    /// Creates a parameter to be passed into an [`ExclusiveSystemParamFunction`]
    fn get_param<'s>(state: &'s mut Self::State, system_meta: &SystemMeta) -> Self::Item<'s>;
}

impl<'_s, T: FromWorld + Send + 'static> ExclusiveSystemParam for Local<'_s, T> {
    type State = SyncCell<T>;
    type Item<'s> = Local<'s, T>;

    fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        SyncCell::new(T::from_world(world))
    }

    fn get_param<'s>(state: &'s mut Self::State, _system_meta: &SystemMeta) -> Self::Item<'s> {
        Local(state.get())
    }
}

/// Shorthand way of accessing the associated type [`ExclusiveSystemParam::Item`]
/// for a given [`ExclusiveSystemParam`]
pub type ExclusiveSystemParamItem<'s, P> = <P as ExclusiveSystemParam>::Item<'s>;

macro_rules! impl_exclusive_system_param_tuple {
    ($(#[$meta:meta])* $($param: ident),*) => {
        #[expect(
            clippy::allow_attributes,
            reason = "This is within a macro, and as such, the below lints may not always apply."
        )]
        #[allow(
            non_snake_case,
            reason = "Certain variable names are provided by the caller, not by us."
        )]
        #[allow(
            unused_variables,
            reason = "Zero-length tuples won't use any of the parameters."
        )]
        $(#[$meta])*
        impl<$($param: ExclusiveSystemParam),*> ExclusiveSystemParam for ($($param,)*) {
            type State = ($($param::State,)*);
            type Item<'s> = ($($param::Item<'s>,)*);

            #[inline]
            fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
                (($($param::init(world, system_meta),)*))
            }

            #[inline]
            fn get_param<'s>(
                state: &'s mut Self::State,
                system_meta: &SystemMeta,
            ) -> Self::Item<'s> {
                let ($($param,)*) = state;
                #[allow(
                    clippy::unused_unit,
                    reason = "Zero-length tuples won't have any params to get."
                )]
                ($($param::get_param($param, system_meta),)*)
            }
        }
    };
}

all_tuples!(impl_exclusive_system_param_tuple, 0, 16, P);
