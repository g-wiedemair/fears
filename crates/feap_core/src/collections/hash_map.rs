//! Provides [`HashMap`] based on [hashbrown]s implementation.
//! Unlike [`Hashbrown::HashMap`], [`HashMap`] defaults fo [`FixedHasher`]
//! instead of [`RandomState`].
//! This provides determinism by default with an acceptable compromise to denial
//! of service resistance in the context of a graphic engine

use crate::hash::FixedHasher;
use core::{
    fmt::Debug,
    hash::{BuildHasher, Hash},
    ops::{Deref, DerefMut, Index},
};
use hashbrown::{Equivalent, hash_map as hb};

#[repr(transparent)]
pub struct HashMap<K, V, S = FixedHasher>(hb::HashMap<K, V, S>);

impl<K, V, S> Clone for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0)
    }
}

impl<K, V, S> Debug for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <hb::HashMap<K, V, S> as Debug>::fmt(&self.0, f)
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, V, S> PartialEq for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V, S> Eq for HashMap<K, V, S> where hb::HashMap<K, V, S>: Eq {}

impl<K, V, S, T> FromIterator<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: FromIterator<T>,
{
    #[inline]
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl<K, V, S, T> Index<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Index<T>,
{
    type Output = <hb::HashMap<K, V, S> as Index<T>>::Output;

    #[inline]
    fn index(&self, index: T) -> &Self::Output {
        self.0.index(index)
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: IntoIterator,
{
    type Item = <hb::HashMap<K, V, S> as IntoIterator>::Item;

    type IntoIter = <hb::HashMap<K, V, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S>
where
    &'a hb::HashMap<K, V, S>: IntoIterator,
{
    type Item = <&'a hb::HashMap<K, V, S> as IntoIterator>::Item;

    type IntoIter = <&'a hb::HashMap<K, V, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S>
where
    &'a mut hb::HashMap<K, V, S>: IntoIterator,
{
    type Item = <&'a mut hb::HashMap<K, V, S> as IntoIterator>::Item;

    type IntoIter = <&'a mut hb::HashMap<K, V, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<K, V, S, T> Extend<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Extend<T>,
{
    #[inline]
    fn extend<U: IntoIterator<Item = T>>(&mut self, iter: U) {
        self.0.extend(iter);
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for HashMap<K, V, FixedHasher>
where
    K: Eq + Hash,
{
    fn from(arr: [(K, V); N]) -> Self {
        arr.into_iter().collect()
    }
}

impl<K, V, S> From<hb::HashMap<K, V, S>> for HashMap<K, V, S> {
    #[inline]
    fn from(value: hb::HashMap<K, V, S>) -> Self {
        Self(value)
    }
}

impl<K, V, S> From<HashMap<K, V, S>> for hb::HashMap<K, V, S> {
    #[inline]
    fn from(value: HashMap<K, V, S>) -> Self {
        value.0
    }
}

impl<K, V, S> Deref for HashMap<K, V, S> {
    type Target = hb::HashMap<K, V, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, S> DerefMut for HashMap<K, V, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "serialize")]
impl<K, V, S> serde::Serialize for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: serde::Serialize,
{
    #[inline]
    fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
    where
        T: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de, K, V, S> serde::Deserialize<'de> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: serde::Deserialize<'de>,
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
impl<K, V, S, T> FromParallelIterator<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: FromParallelIterator<T>,
    T: Send,
{
    fn from_par_iter<P>(par_iter: P) -> Self
    where
        P: IntoParallelIterator<Item = T>,
    {
        Self(<hb::HashMap<K, V, S> as FromParallelIterator<T>>::from_par_iter(par_iter))
    }
}

#[cfg(feature = "rayon")]
impl<K, V, S> IntoParallelIterator for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: IntoParallelIterator,
{
    type Item = <hb::HashMap<K, V, S> as IntoParallelIterator>::Item;
    type Iter = <hb::HashMap<K, V, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, K: Sync, V: Sync, S> IntoParallelIterator for &'a HashMap<K, V, S>
where
    &'a hb::HashMap<K, V, S>: IntoParallelIterator,
{
    type Item = <&'a hb::HashMap<K, V, S> as IntoParallelIterator>::Item;
    type Iter = <&'a hb::HashMap<K, V, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        (&self.0).into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, K: Sync, V: Sync, S> IntoParallelIterator for &'a mut HashMap<K, V, S>
where
    &'a mut hb::HashMap<K, V, S>: IntoParallelIterator,
{
    type Item = <&'a mut hb::HashMap<K, V, S> as IntoParallelIterator>::Item;
    type Iter = <&'a mut hb::HashMap<K, V, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        (&mut self.0).into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<K, V, S, T> ParallelExtend<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: ParallelExtend<T>,
    T: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        <hb::HashMap<K, V, S> as ParallelExtend<T>>::par_extend(&mut self.0, par_iter);
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    /// Returns a reference to the value corresponding to the key
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.get(k)
    }
}
