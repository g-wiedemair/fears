use crate::hash::FixedHasher;
use core::{
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
};
use hashbrown::hash_set as hb;

/// New-type for [`HashSet`](hb::HashSet) with [`FixedHasher`] as the default hashing provider.
/// Unlike [`hashbrown::HashSet`], [`HashSet`] defaults to [`FixedHasher`]
/// instead of [`RandomState`](crate::hash::RandomState).
/// This provides determinism by default with an acceptable compromise to denial
/// of service resistance in the context of a graphics engine.
#[repr(transparent)]
pub struct HashSet<T, S = FixedHasher>(hb::HashSet<T, S>);

impl<T, S> Clone for HashSet<T, S>
where
    hb::HashSet<T, S>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}

impl<T, S> Debug for HashSet<T, S>
where
    hb::HashSet<T, S>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <hb::HashSet<T, S> as Debug>::fmt(&self.0, f)
    }
}

impl<T, S> Default for HashSet<T, S>
where
    hb::HashSet<T, S>: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, S> PartialEq for HashSet<T, S>
where
    hb::HashSet<T, S>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T, S> Eq for HashSet<T, S> where hb::HashSet<T, S>: Eq {}

impl<T, S, X> FromIterator<X> for HashSet<T, S>
where
    hb::HashSet<T, S>: FromIterator<X>,
{
    #[inline]
    fn from_iter<U: IntoIterator<Item = X>>(iter: U) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl<T, S> IntoIterator for HashSet<T, S>
where
    hb::HashSet<T, S>: IntoIterator,
{
    type Item = <hb::HashSet<T, S> as IntoIterator>::Item;

    type IntoIter = <hb::HashSet<T, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, S> IntoIterator for &'a HashSet<T, S>
where
    &'a hb::HashSet<T, S>: IntoIterator,
{
    type Item = <&'a hb::HashSet<T, S> as IntoIterator>::Item;

    type IntoIter = <&'a hb::HashSet<T, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, T, S> IntoIterator for &'a mut HashSet<T, S>
where
    &'a mut hb::HashSet<T, S>: IntoIterator,
{
    type Item = <&'a mut hb::HashSet<T, S> as IntoIterator>::Item;

    type IntoIter = <&'a mut hb::HashSet<T, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<T, S, X> Extend<X> for HashSet<T, S>
where
    hb::HashSet<T, S>: Extend<X>,
{
    #[inline]
    fn extend<U: IntoIterator<Item = X>>(&mut self, iter: U) {
        self.0.extend(iter);
    }
}

impl<T, const N: usize> From<[T; N]> for HashSet<T, FixedHasher>
where
    T: Eq + Hash,
{
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

impl<T, S> From<crate::collections::HashMap<T, (), S>> for HashSet<T, S> {
    #[inline]
    fn from(value: crate::collections::HashMap<T, (), S>) -> Self {
        Self(hb::HashSet::from(hashbrown::HashMap::from(value)))
    }
}

impl<T, S> From<hb::HashSet<T, S>> for HashSet<T, S> {
    #[inline]
    fn from(value: hb::HashSet<T, S>) -> Self {
        Self(value)
    }
}

impl<T, S> From<HashSet<T, S>> for hb::HashSet<T, S> {
    #[inline]
    fn from(value: HashSet<T, S>) -> Self {
        value.0
    }
}

impl<T, S> Deref for HashSet<T, S> {
    type Target = hb::HashSet<T, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, S> DerefMut for HashSet<T, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "serialize")]
impl<T, S> serde::Serialize for HashSet<T, S>
where
    hb::HashSet<T, S>: serde::Serialize,
{
    #[inline]
    fn serialize<U>(&self, serializer: U) -> Result<U::Ok, U::Error>
    where
        U: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de, T, S> serde::Deserialize<'de> for HashSet<T, S>
where
    hb::HashSet<T, S>: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(serde::Deserialize::deserialize(deserializer)?))
    }
}

#[cfg(feature = "rayon")]
impl<T, S, U> FromParallelIterator<U> for HashSet<T, S>
where
    hb::HashSet<T, S>: FromParallelIterator<U>,
    U: Send,
{
    fn from_par_iter<P>(par_iter: P) -> Self
    where
        P: IntoParallelIterator<Item = U>,
    {
        Self(<hb::HashSet<T, S> as FromParallelIterator<U>>::from_par_iter(par_iter))
    }
}

#[cfg(feature = "rayon")]
impl<T, S> IntoParallelIterator for HashSet<T, S>
where
    hb::HashSet<T, S>: IntoParallelIterator,
{
    type Item = <hb::HashSet<T, S> as IntoParallelIterator>::Item;
    type Iter = <hb::HashSet<T, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, T: Sync, S> IntoParallelIterator for &'a HashSet<T, S>
where
    &'a hb::HashSet<T, S>: IntoParallelIterator,
{
    type Item = <&'a hb::HashSet<T, S> as IntoParallelIterator>::Item;
    type Iter = <&'a hb::HashSet<T, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        (&self.0).into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<T, S, U> ParallelExtend<U> for HashSet<T, S>
where
    hb::HashSet<T, S>: ParallelExtend<U>,
    U: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = U>,
    {
        <hb::HashSet<T, S> as ParallelExtend<U>>::par_extend(&mut self.0, par_iter);
    }
}

impl<T> HashSet<T, FixedHasher> {
    /// Creates an empty [`HashSet`]
    #[inline]
    pub const fn new() -> Self {
        Self::with_hasher(FixedHasher)
    }

    /// Creates an empty [`HashSet`] with the specified capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, FixedHasher)
    }
}

impl<T, S> HashSet<T, S> {
    /// Creates a new empty hash set which will use the given hasher to hash keys
    #[inline]
    pub const fn with_hasher(hasher: S) -> Self {
        Self(hb::HashSet::with_hasher(hasher))
    }

    /// Creates an empty [`HashSet`] with the specified capacity, using `hasher` to hash the keys
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self(hb::HashSet::with_capacity_and_hasher(capacity, hasher))
    }
}
