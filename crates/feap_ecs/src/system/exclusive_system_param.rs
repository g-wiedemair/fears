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
        }
    };
}

all_tuples!(impl_exclusive_system_param_tuple, 0, 16, P);
