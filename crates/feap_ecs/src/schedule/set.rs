use super::ScheduleLabel;
use crate::{define_label, intern::Interned};
use alloc::boxed::Box;
use core::{any::TypeId, fmt::Debug, hash::Hash, marker::PhantomData};
pub use feap_ecs_macros::SystemSet;
use std::hash::Hasher;

define_label!(
    /// System sets are tag-like labels that can be used to group systems together
    ///
    /// This allows you to share configuration (like run conditions) across multiple systems,
    /// and order systems or system sets relative to conceptual groups of systems.
    ///
    #[diagnostic::on_unimplemented(
        note = "consider annotating `{Self}` with `#[derive(SystemSet)]`"
    )]
    SystemSet,
    SYSTEM_SET_INTERNER,
    extra_methods: {
        /// Returns `Some` if this system set is a [`SystemTypeSet`]
        fn system_type(&self) -> Option<TypeId> {
            None
        }
    },
    extra_methods_impl: {
        fn system_type(&self) -> Option<TypeId> {
            (**self).system_type()
        }
    }
);

/// A shorthand for `Interned<dyn SystemSet>`
pub type InternedSystemSet = Interned<dyn SystemSet>;
/// A shorthand for `Interned<dyn ScheduleLabel>`
pub type InternedScheduleLabel = Interned<dyn ScheduleLabel>;

/// A [`SystemSet`] grouping instances of the same function
///
/// This kind of set is automatically populated and thus has some special rules
/// - You cannot manually add members
/// - You cannot configure them
/// - You cannot order something relative to one if it has more than one member
pub struct SystemTypeSet<T: 'static>(PhantomData<fn() -> T>);

impl<T: 'static> SystemTypeSet<T> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Debug for SystemTypeSet<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl<T> Hash for SystemTypeSet<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        // all systems of a given type are the same
    }
}

impl<T> Clone for SystemTypeSet<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SystemTypeSet<T> {}

impl<T> PartialEq for SystemTypeSet<T> {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        // all systems of a given type are the same
        true
    }
}

impl<T> Eq for SystemTypeSet<T> {}

impl<T> SystemSet for SystemTypeSet<T> {
    fn dyn_clone(&self) -> Box<dyn SystemSet> {
        Box::new(*self)
    }
}
