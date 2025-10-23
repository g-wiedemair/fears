//! Types that detect when their internal data mutate

use crate::{
    component::{Tick, TickCells},
    resource::Resource,
};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    panic::Location,
};
use feap_core::ptr::{PtrMut, UnsafeCellDeref};

/// Types that can read change detection information
/// This change detection is controlled by [`DetectChangesMut`] types such as [`RestMut`]
pub trait DetectChanges {
    /// The location that last caused this to change.
    fn changed_by(&self) -> MaybeLocation;
}

/// Types that implement reliable change detection
pub trait DetectChangesMut: DetectChanges {
    /// The type contained within this smart pointer
    type Inner: ?Sized;

    /// Flags this value as having been changed
    fn set_changed(&mut self);
}

macro_rules! change_detection_impl {
    ($name:ident < $( $generics:tt ),+ >, $target:ty, $($traits:ident)?)  => {
        impl<$($generics),* : ?Sized $(+ $traits)?> DetectChanges for $name<$($generics),*> {
            #[inline]
            fn changed_by(&self) -> MaybeLocation {
                self.changed_by.copied()
            }
        }

        impl<$($generics),*: ?Sized $(+ $traits)?> Deref for $name<$($generics),*> {
            type Target = $target;

            #[inline]
            fn deref(&self) -> &Self::Target {
                self.value
            }
        }


        impl<$($generics),* $(: $traits)?> AsRef<$target> for $name<$($generics),*> {
            #[inline]
            fn as_ref(&self) -> &$target {
                self.deref()
            }
        }
    };
}

macro_rules! change_detection_mut_impl {
    ($name:ident < $( $generics:tt ),+ >, $target:ty, $($traits:ident)?) => {
        impl<$($generics),* : ?Sized $(+ $traits)?> DetectChangesMut for $name<$($generics),*> {
            type Inner = $target;

            #[inline]
            #[track_caller]
            fn set_changed(&mut self) {
                *self.ticks.changed = self.ticks.this_run;
                self.changed_by.assign(MaybeLocation::caller());
            }
        }

        impl<$($generics),* : ?Sized $(+ $traits)?> DerefMut for $name<$($generics),*> {
            #[inline]
            #[track_caller]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.set_changed();
                self.changed_by.assign(MaybeLocation::caller());
                self.value
            }
        }

        impl<$($generics),* $(: $traits)?> AsMut<$target> for $name<$($generics),*> {
            #[inline]
            fn as_mut(&mut self) -> &mut $target {
                self.deref_mut()
            }
        }
    }
}

/// Shared borrow of a [`Resource`]
///
pub struct Res<'w, T: ?Sized + Resource> {
    pub(crate) value: &'w T,
}

/// Unique mutable borrow of a [`Resource`]
///
pub struct ResMut<'w, T: ?Sized + Resource> {
    pub(crate) value: &'w mut T,
}

/// A value that contains a `T` if the `track_location` feature is enabled
/// and is a ZST if it is not
///
/// The overall API is similar to [`Option`], but whether the value is `Some` or `None` is set at compile
/// time and is the same for all values
///
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaybeLocation<T: ?Sized = &'static Location<'static>> {
    marker: PhantomData<T>,
    #[cfg(feature = "track_location")]
    value: T,
}

impl MaybeLocation {
    #[inline]
    #[track_caller]
    pub const fn caller() -> Self {
        MaybeLocation {
            #[cfg(feature = "track_location")]
            value: Location::caller(),
            marker: PhantomData,
        }
    }
}

impl<T> MaybeLocation<T> {
    /// Maps an `MaybeLocation<T> `to `MaybeLocation<U>` by applying a function to a contained value.
    #[inline]
    pub fn map<U>(self, _f: impl FnOnce(T) -> U) -> MaybeLocation<U> {
        MaybeLocation {
            #[cfg(feature = "track_location")]
            value: _f(self.value),
            marker: PhantomData,
        }
    }
}

impl<T> MaybeLocation<&T> {
    /// Maps an `MaybeLocation<&T>` to an `MaybeLocation<T>` by copying the contents.
    #[inline]
    pub const fn copied(&self) -> MaybeLocation<T>
    where
        T: Copy,
    {
        MaybeLocation {
            #[cfg(feature = "track_location")]
            value: *self.value,
            marker: PhantomData,
        }
    }
}

impl<T> MaybeLocation<&mut T> {
    /// Maps an `MaybeLocation<&mut T>` to an `MaybeLocation<T>` by copying the contents.
    #[inline]
    pub const fn copied(&self) -> MaybeLocation<T>
    where
        T: Copy,
    {
        MaybeLocation {
            #[cfg(feature = "track_location")]
            value: *self.value,
            marker: PhantomData,
        }
    }

    /// Assigns the contents of an `MaybeLocation<T>` to an `MaybeLocation<&mut T>`.
    #[inline]
    pub fn assign(&mut self, _value: MaybeLocation<T>) {
        #[cfg(feature = "track_location")]
        {
            *self.value = _value.value;
        }
    }
}

impl<T: ?Sized> MaybeLocation<T> {
    /// Converts from `&MaybeLocation<T>` to `MaybeLocation<&T>`.
    #[inline]
    pub const fn as_ref(&self) -> MaybeLocation<&T> {
        MaybeLocation {
            #[cfg(feature = "track_location")]
            value: &self.value,
            marker: PhantomData,
        }
    }

    /// Converts from `&mut MaybeLocation<T>` to `MaybeLocation<&mut T>`
    #[inline]
    pub const fn as_mut(&mut self) -> MaybeLocation<&mut T> {
        MaybeLocation {
            #[cfg(feature = "track_location")]
            value: &mut self.value,
            marker: PhantomData,
        }
    }
}

pub(crate) struct TicksMut<'w> {
    pub(crate) added: &'w mut Tick,
    pub(crate) changed: &'w mut Tick,
    pub(crate) last_run: Tick,
    pub(crate) this_run: Tick,
}

impl<'w> TicksMut<'w> {
    #[inline]
    pub(crate) unsafe fn from_tick_cells(
        cells: TickCells<'w>,
        last_run: Tick,
        this_run: Tick,
    ) -> Self {
        Self {
            added: unsafe { cells.added.deref_mut() },
            changed: unsafe { cells.changed.deref_mut() },
            last_run,
            this_run,
        }
    }
}

/// Unique mutable borrow of an entity's component or a resource
///
/// This can be used in queries to access change detection from immutable query methods
///
pub struct Mut<'w, T: ?Sized> {
    pub(crate) value: &'w mut T,
    pub(crate) ticks: TicksMut<'w>,
    pub(crate) changed_by: MaybeLocation<&'w mut &'static Location<'static>>,
}

change_detection_impl!(Mut<'w, T>, T,);
change_detection_mut_impl!(Mut<'w, T>, T,);

/// Unique mutable borrow of resources or an entity's component
/// Similar to [`Mut`], but no generic over the component type,
/// instead exposing the raw pointer as a *mut
pub struct MutUntyped<'w> {
    pub(crate) value: PtrMut<'w>,
    pub(crate) ticks: TicksMut<'w>,
    pub(crate) changed_by: MaybeLocation<&'w mut &'static Location<'static>>,
}

impl<'w> MutUntyped<'w> {
    /// Transforms this [`MutUntyped`] into a [`Mut<T>`] with the same lifetime
    pub unsafe fn with_type<T>(self) -> Mut<'w, T> {
        Mut {
            value: unsafe { self.value.deref_mut() },
            ticks: self.ticks,
            changed_by: self.changed_by,
        }
    }
}
