//! Provides types used to statically intern immutable values
//!
//! Interning is a pattern used to save memory by deduplicating values,
//! speed up code by shrinking the stack size of large types,
//! and make comparisons for any type as fast as integers

use alloc::{borrow::ToOwned, boxed::Box};
use core::{fmt::Debug, hash::Hash, ops::Deref};
use feap_core::{
    collections::HashSet,
    hash::FixedHasher,
    sync::{PoisonError, RwLock},
};

/// An interned value. Will stay valid until the end of the program and will not drop
///
/// Interned values use reference equality, meaning they implement [`Eq`]
/// and [`Hash`] regardless of whether `T` implements these traits.
/// Two interned values are only guaranteed to compare equal if they were interned using
/// the same [`Interner`] instance.
///
pub struct Interned<T: ?Sized + 'static>(pub &'static T);

impl<T: ?Sized> Deref for Interned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: ?Sized> Clone for Interned<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Interned<T> {}

// Two Interned<T> should only be equal if they are clones from the same instance
impl<T: ?Sized + Internable> PartialEq for Interned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ref_eq(other.0)
    }
}

impl<T: ?Sized + Internable> Eq for Interned<T> {}

impl<T: ?Sized + Internable> Hash for Interned<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.ref_hash(state);
    }
}

impl<T: ?Sized + Debug> Debug for Interned<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<&Interned<T>> for Interned<T> {
    fn from(value: &Interned<T>) -> Self {
        *value
    }
}

/// A trait for internable values
///
/// This is used by [`Interner<T>`] to create static references for values that are interned
///
pub trait Internable: Hash + Eq {
    /// Creates a static reference to `self`, possibly leaking memory
    fn leak(&self) -> &'static Self;

    /// Returns `true` if the two references point to the same value
    fn ref_eq(&self, other: &Self) -> bool;

    /// Feeds the reference to the hasher
    fn ref_hash<H: core::hash::Hasher>(&self, state: &mut H);
}

impl Internable for str {
    fn leak(&self) -> &'static Self {
        let str = self.to_owned().into_boxed_str();
        Box::leak(str)
    }

    fn ref_eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr() && self.len() == other.len()
    }

    fn ref_hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        self.as_ptr().hash(state);
    }
}

/// A thread-safe interner which can be used to create [`Interned<T>`]
pub struct Interner<T: ?Sized + 'static>(RwLock<HashSet<&'static T>>);

impl<T: ?Sized> Default for Interner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Interner<T> {
    /// Creates a new empty interner
    pub const fn new() -> Self {
        Self(RwLock::new(HashSet::with_hasher(FixedHasher)))
    }
}

impl<T: Internable + ?Sized> Interner<T> {
    /// Return the [`Interned<T>`] corresponding to `value`
    /// If it is called the first time for `value`, it will possibly leak the value and return an
    /// [`Interned<T>`] using the obtained static reference.
    pub fn intern(&self, value: &T) -> Interned<T> {
        {
            let set = self.0.read().unwrap_or_else(PoisonError::into_inner);
            if let Some(value) = set.get(value) {
                return Interned(*value);
            }
        }

        {
            let mut set = self.0.write().unwrap_or_else(PoisonError::into_inner);
            if let Some(value) = set.get(value) {
                Interned(*value)
            } else {
                let leaked = value.leak();
                set.insert(leaked);
                Interned(leaked)
            }
        }
    }
}
