use super::System;
use core::ops::{Deref, DerefMut};
use variadics_please::all_tuples;

/// Trait for types that can be used as input to [`System`]s
pub trait SystemInput: Sized {
    /// The wrapper input type that is defined as the first argument to [`FunctionSystem`]s
    type Param<'i>: SystemInput;
    /// The inner input type that is passed to functions that run systems
    type Inner<'i>;
}

/// Shorthand way to get the [`System::In`] for a [`System`] as a [`SystemInput::Inner`]
pub type SystemIn<'a, S> = <<S as System>::In as SystemInput>::Inner<'a>;

/// A [`SystemInput`] type which denotes that a [`System`] receives
/// an input value of type `T` from its caller
///
/// [`System`]s may take an optional input which they require to be passed to them when they
/// are being [`run`]. For [`FunctionSystem`]s the input may be marked with this `In` type,
/// but only the first param of a function may be tagged as an input. This also means a system
/// can only have one or zero input parameters
#[derive(Debug)]
pub struct In<T>(pub T);

impl<T: 'static> SystemInput for In<T> {
    type Param<'i> = In<T>;
    type Inner<'i> = T;
}

impl<T> Deref for In<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for In<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

macro_rules! impl_system_input_tuple {
    ($(#[$meta:meta])* $($name:ident),*) => {
        $(#[$meta])*
        impl<$($name: SystemInput),*> SystemInput for ($($name,)*) {
            type Param<'i> = ($($name::Param<'i>,)*);
            type Inner<'i> = ($($name::Inner<'i>,)*);

            // #[expect(
            //     clippy::allow_attributes,
            //     reason = "This is in a macro; as such, the below lints may not always apply."
            // )]
            // #[allow(
            //     non_snake_case,
            //     reason = "Certain variable names are provided by the caller, not by us."
            // )]
            // #[allow(
            //     clippy::unused_unit,
            //     reason = "Zero-length tuples won't have anything to wrap."
            // )]
        }
    }
}

all_tuples!(impl_system_input_tuple, 0, 8, I);
