use alloc::vec::Vec;
use core::{hash::Hash, marker::PhantomData};
use nonmax::NonMaxUsize;

#[derive(Debug)]
pub(crate) struct SparseArray<I, V = I> {
    values: Vec<Option<V>>,
    marker: PhantomData<I>,
}

impl<I: SparseSetIndex, V> Default for SparseArray<I, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, V> SparseArray<I, V> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            values: Vec::new(),
            marker: PhantomData,
        }
    }
}

macro_rules! impl_sparse_array {
    ($ty:ident) => {
        impl<I: SparseSetIndex, V> $ty<I, V> {
            /// Returns a reference to the value at `index`
            #[inline]
            pub fn get(&self, index: I) -> Option<&V> {
                let index = index.sparse_set_index();
                self.values.get(index).and_then(Option::as_ref)
            }
        }
    };
}

impl_sparse_array!(SparseArray);

impl<I: SparseSetIndex, V> SparseArray<I, V> {
    /// Inserts `value` at `index` in the array
    #[inline]
    pub fn insert(&mut self, index: I, value: V) {
        let index = index.sparse_set_index();
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }
}

/// A data structure that blends dense and sparse storage
/// `I` is the type of the indices, while `V` is the type of data
#[derive(Debug)]
pub struct SparseSet<I, V: 'static> {
    dense: Vec<V>,
    indices: Vec<I>,
    sparse: SparseArray<I, NonMaxUsize>,
}

impl<I: SparseSetIndex, V> Default for SparseSet<I, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, V> SparseSet<I, V> {
    pub const fn new() -> Self {
        Self {
            dense: Vec::new(),
            indices: Vec::new(),
            sparse: SparseArray::new(),
        }
    }
}

macro_rules! impl_sparse_set {
    ($ty:ident) => {
        impl<I: SparseSetIndex, V> $ty<I, V> {
            /// Returns a reference to the value for `index`
            pub fn get(&self, index: I) -> Option<&V> {
                self.sparse
                    .get(index)
                    .map(|dense_index| unsafe { self.dense.get_unchecked(dense_index.get()) })
            }
        }
    };
}

impl_sparse_set!(SparseSet);

impl<I: SparseSetIndex, V> SparseSet<I, V> {
    /// Returns a reference to the value for `index`,
    /// inserting one computed from `func` if not already present
    pub fn get_or_insert_with(&mut self, index: I, func: impl FnOnce() -> V) -> &mut V {
        if let Some(dense_index) = self.sparse.get(index.clone()).cloned() {
            unsafe { self.dense.get_unchecked_mut(dense_index.get()) }
        } else {
            let value = func();
            let dense_index = self.dense.len();
            self.sparse
                .insert(index.clone(), NonMaxUsize::new(dense_index).unwrap());
            self.indices.push(index);
            self.dense.push(value);
            unsafe { self.dense.get_unchecked_mut(dense_index) }
        }
    }
}

/// Represents something that can be stored in a [`SparseSet`] as an integer
pub trait SparseSetIndex: Clone + PartialEq + Eq + Hash {
    fn sparse_set_index(&self) -> usize;
    fn get_sparse_set_index(value: usize) -> Self;
}

macro_rules! impl_sparse_set_index {
    ($($ty:ty),+) => {
        $(impl SparseSetIndex for $ty {
            #[inline]
            fn sparse_set_index(&self) -> usize {
                *self as usize
            }
            #[inline]
            fn get_sparse_set_index(value: usize) -> Self {
                value as $ty
            }
        })*
    };
}

impl_sparse_set_index!(u8, u16, u32, u64, usize);
