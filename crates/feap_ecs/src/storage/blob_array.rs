use alloc::alloc::handle_alloc_error;
use core::{alloc::Layout, num::NonZeroUsize, ptr::NonNull};
use feap_core::ptr::{self, OwningPtr, Ptr, PtrMut};

/// A flat, typed-erased data storage type
///
/// Used to densely store homogeneous ECS data. A blob is usually an arbitrary block of contiguous memory without any identity, and
/// could be used to represent any arbitrary data (i.e. string, arrays, etc). This type only stores meta-data about the blob that it stores,
/// and a pointer to the location of the start of the array, similar to a C-style `void*` array.
///
/// This type is reliant on its owning type to store the capacity and length information.
#[derive(Debug)]
pub(super) struct BlobArray {
    item_layout: Layout,
    data: NonNull<u8>,
    pub drop: Option<unsafe fn(OwningPtr<'_>)>,
    #[cfg(debug_assertions)]
    capacity: usize,
}

impl BlobArray {
    /// Create a new [`BlobArray`] with a specified `capacity`.
    /// If `capacity` is 0, no allocations will be made.
    ///
    /// `drop` is an optional function pointer that is meant to be invoked when any element in the [`BlobArray`]
    /// should be dropped. For all Rust-based types, this should match 1:1 with the implementation of [`Drop`]
    /// if present, and should be `None` if `T: !Drop`. For non-Rust based types, this should match any cleanup
    /// processes typically associated with the stored element.
    pub unsafe fn with_capacity(
        item_layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
        capacity: usize,
    ) -> Self {
        if capacity == 0 {
            let align = NonZeroUsize::new(item_layout.align()).expect("alignment must be > 0");
            let data = ptr::dangling_with_align(align);
            Self {
                item_layout,
                drop: drop_fn,
                data,
                #[cfg(debug_assertions)]
                capacity,
            }
        } else {
            unsafe {
                let mut arr = Self::with_capacity(item_layout, drop_fn, 0);
                arr.alloc(NonZeroUsize::new_unchecked(capacity));
                arr
            }
        }
    }

    /// Return `true` if this [`BlobArray`] stores `ZSTs`.
    pub fn is_zst(&self) -> bool {
        self.item_layout.size() == 0
    }

    /// Allocate a block of memory for the array. This should be used to initialize the array, do not use this
    /// method if there are already elements stored in the array - use [`Self::realloc`] instead.
    pub(super) fn alloc(&mut self, capacity: NonZeroUsize) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.capacity, 0);
        if !self.is_zst() {
            let new_layout = array_layout(&self.item_layout, capacity.get())
                .expect("array layout should be valid");
            let new_data = unsafe { alloc::alloc::alloc(new_layout) };
            self.data = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
        }
        #[cfg(debug_assertions)]
        {
            self.capacity = capacity.into();
        }
    }

    /// Initializes the value at `index` to `value`. This function does not do any bounds checking.
    #[inline]
    pub unsafe fn initialize_unchecked(&mut self, index: usize, value: OwningPtr<'_>) {
        #[cfg(debug_assertions)]
        debug_assert!(self.capacity > index);
        let size = self.item_layout.size();
        let dst = self.get_unchecked_mut(index);
        core::ptr::copy::<u8>(value.as_ptr(), dst.as_ptr(), size);
    }

    /// Returns a reference to the element at `index`, without doing bounds checking
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> Ptr<'_> {
        #[cfg(debug_assertions)]
        debug_assert!(index < self.capacity);
        let size = self.item_layout.size();
        // SAFETY:
        // - The caller ensures that `index` fits in this array,
        //   so this operation will not overflow the original allocation.
        // - `size` is a multiple of the erased type's alignment,
        //   so adding a multiple of `size` will preserve alignment.
        unsafe { self.get_ptr().byte_add(index * size) }
    }

    /// Returns a mutable reference to the element at `index`, without doing bounds checking
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> PtrMut<'_> {
        #[cfg(debug_assertions)]
        debug_assert!(index < self.capacity);
        let size = self.item_layout.size();
        unsafe { self.get_ptr_mut().byte_add(index * size) }
    }

    /// Gets a [`Ptr`] to the start of the array
    #[inline]
    pub fn get_ptr(&self) -> Ptr<'_> {
        unsafe { Ptr::new(self.data) }
    }

    /// Gets a [`PtrMut`] to the start of the array
    #[inline]
    pub fn get_ptr_mut(&mut self) -> PtrMut<'_> {
        unsafe { PtrMut::new(self.data) }
    }
}

pub(super) fn array_layout(layout: &Layout, n: usize) -> Option<Layout> {
    let (array_layout, offset) = repeat_layout(layout, n)?;
    debug_assert_eq!(layout.size(), offset);
    Some(array_layout)
}

fn repeat_layout(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
    // This cannot overflow. Quoting from the invariant of Layout:
    // > `size`, when rounded up to the nearest multiple of `align`,
    // > must not overflow (i.e., the rounded value must be less than`usize::MAX`)
    let padded_size = layout.size() + padding_needed_for(layout, layout.align());
    let alloc_size = padded_size.checked_mul(n)?;

    unsafe {
        Some((
            Layout::from_size_align_unchecked(alloc_size, layout.align()),
            padded_size,
        ))
    }
}

const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let len = layout.size();

    // Rounded up value is:
    //   len_rounded_up = (len + align - 1) & !(align - 1);
    // and then we return the padding difference: `len_rounded_up - len`.
    //
    // We use modular arithmetic throughout:
    //
    // 1. align is guaranteed to be > 0, so align - 1 is always
    //    valid.
    //
    // 2. `len + align - 1` can overflow by at most `align - 1`,
    //    so the &-mask with `!(align - 1)` will ensure that in the
    //    case of overflow, `len_rounded_up` will itself be 0.
    //    Thus the returned padding, when added to `len`, yields 0,
    //    which trivially satisfies the alignment `align`.
    //
    // (Of course, attempts to allocate blocks of memory whose
    // size and padding overflow in the above manner should cause
    // the allocator to yield an error anyway.)

    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}
