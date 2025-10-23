#![expect(unsafe_code, reason = "Raw pointers are inherently unsafe.")]

use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::ManuallyDrop,
    num::NonZeroUsize,
    ptr::{self, NonNull},
};

/// Used as a type argument to specify that the pointer is guaranteed to be [aligned]
///
#[derive(Debug, Copy, Clone)]
pub struct Aligned;

/// Used as a type argument to specify that the pointer may not be [aligned]
///
#[derive(Debug, Copy, Clone)]
pub struct Unaligned;

/// Trait that is only implemented for [`Aligned`] and [`Unaligned`] to work around the lack of
/// ability to have const generics of an enum
pub trait IsAligned: sealed::Sealed {}

impl IsAligned for Aligned {}

impl IsAligned for Unaligned {}

/// Type-erased borrow of some unknown type chosen when constructing this type
///
/// This type tries to act "borrow-like" which means that:
/// - It should be considered immutable
/// - It must always point to a valid value of whatever the pointer type is
/// - The lifetime `'a` accurately represents how long the pointer is valid for
///
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Ptr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a u8, A)>);

/// Type-erased mutable borrow of some unknown type chosen when constructing this type
///
/// This type tries to act "borrow-like" which means that:
/// - Pointer is considered exclusive and mutable. It cannot be cloned as this would lead to
///   aliased mutability.
/// - It must always point to a valid value of whatever the pointee type is.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
///
#[repr(transparent)]
pub struct PtrMut<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

/// Type-erased [`Box`]-like pointer to some unknown type chosen when constructing this type
///
/// Conceptually represents ownership of whatever data is being pointed to and so is
/// responsible for calling its `Drop` impl. This pointer is _not_ responsible for freeing
/// the memory pointed to by this pointer as it may be pointing to an element in a `Vec` or
/// to a local in a function etc.
///
#[repr(transparent)]
pub struct OwningPtr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

macro_rules! impl_ptr {
    ($ptr:ident) => {
        impl<'a> $ptr<'a, Aligned> {}

        impl<'a, A: IsAligned> From<$ptr<'a, A>> for NonNull<u8> {
            fn from(ptr: $ptr<'a, A>) -> Self {
                ptr.0
            }
        }

        impl<A: IsAligned> $ptr<'_, A> {
            /// Calculates the offset from a pointer
            #[inline]
            pub unsafe fn byte_add(self, count: usize) -> Self {
                Self(
                    unsafe { NonNull::new_unchecked(self.as_ptr().add(count)) },
                    PhantomData,
                )
            }
        }
    };
}

impl_ptr!(Ptr);
impl_ptr!(PtrMut);
impl_ptr!(OwningPtr);

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Aligned {}
    impl Sealed for super::Unaligned {}
}

//-----------------------------------------------------------------------------------
// Ptr

impl<'a, A: IsAligned> Ptr<'a, A> {
    /// Creates a new instance from a raw pointer
    #[inline]
    pub unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Transforms this [`Ptr`] into a [`PtrMut`]
    #[inline]
    pub unsafe fn assert_unique(self) -> PtrMut<'a, A> {
        PtrMut(self.0, PhantomData)
    }

    /// Transforms this [`Ptr<T>`] into a `&T` with the same lifetime
    #[inline]
    pub unsafe fn deref<T>(self) -> &'a T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { &*ptr }
    }

    /// Gets the underlying pointer, erasing the associated lifetime
    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }
}

//-----------------------------------------------------------------------------------
// PtrMut

impl<'a, T: ?Sized> From<&'a mut T> for PtrMut<'a> {
    #[inline]
    fn from(val: &'a mut T) -> Self {
        // SAFETY: The returned pointer has the same lifetime as the passed reference.
        // The reference is mutable, and thus will not alias.
        unsafe { Self::new(NonNull::from(val).cast()) }
    }
}

impl<'a, A: IsAligned> PtrMut<'a, A> {
    #[inline]
    pub unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Transforms this [`PtrMut`] into an [`OwningPtr`]
    #[inline]
    pub unsafe fn promote(self) -> OwningPtr<'a, A> {
        OwningPtr(self.0, PhantomData)
    }

    /// Transforms this [`PtrMut`] into a `&mut T` with the same lifetime
    #[inline]
    pub unsafe fn deref_mut<T>(self) -> &'a mut T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { &mut *ptr }
    }

    /// Gets the underlying pointer, erasing the associated lifetime
    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }
}

//-----------------------------------------------------------------------------------
// OwningMut

impl<'a> OwningPtr<'a> {
    /// Creates a new instance from a raw pointer
    #[inline]
    pub unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Consumes a value and creates an [`OwningPtr`] to it while ensuring a double drop does not happen
    #[inline]
    pub fn make<T, F: FnOnce(OwningPtr<'_>) -> R, R>(val: T, f: F) -> R {
        let mut val = ManuallyDrop::new(val);
        f(unsafe { Self::make_internal(&mut val) })
    }

    unsafe fn make_internal<T>(temp: &mut ManuallyDrop<T>) -> OwningPtr<'_> {
        unsafe { PtrMut::from(&mut *temp).promote() }
    }
}

impl<'a, A: IsAligned> OwningPtr<'a, A> {
    /// Consumes the [`OwningPtr`] to obtain ownership of the underlying data of type `T`
    #[inline]
    pub unsafe fn read<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { ptr.read() }
    }

    /// Consumes the [`OwningPtr`] to drop the underlying data of type `T`
    #[inline]
    pub unsafe fn drop_as<T>(self) {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe {
            ptr.drop_in_place();
        }
    }

    /// Gets the underlying pointer, erasing the associated lifetime
    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }
}

//-----------------------------------------------------------------------------------

/// Creates a dangling pointer with specified alignment
pub const fn dangling_with_align(align: NonZeroUsize) -> NonNull<u8> {
    debug_assert!(align.is_power_of_two(), "Alignment must be power of two.");
    unsafe { NonNull::new_unchecked(ptr::null_mut::<u8>().wrapping_add(align.get())) }
}

trait DebugEnsureAligned {
    fn debug_ensure_aligned(self) -> Self;
}

// Disable this for miri runs as it already checks if pointer to reference
// casts are properly aligned
#[cfg(all(debug_assertions, not(miri)))]
impl<T: Sized> DebugEnsureAligned for *mut T {
    #[track_caller]
    fn debug_ensure_aligned(self) -> Self {
        assert!(
            self.is_aligned(),
            "pointer is not aligned. Address {:p} does not have alignment {} for type {}",
            self,
            align_of::<T>(),
            core::any::type_name::<T>()
        );
        self
    }
}

#[cfg(any(not(debug_assertions), miri))]
impl<T: Sized> DebugEnsureAligned for *mut T {
    #[inline(always)]
    fn debug_ensure_aligned(self) -> Self {
        self
    }
}

mod private {
    use core::cell::UnsafeCell;

    pub trait SealedUnsafeCell {}
    impl<'a, T> SealedUnsafeCell for &'a UnsafeCell<T> {}
}

/// Extension trait for helper methods on [`UnsafeCell`]
pub trait UnsafeCellDeref<'a, T>: private::SealedUnsafeCell {
    unsafe fn deref(self) -> &'a T;
    unsafe fn deref_mut(self) -> &'a mut T;
    unsafe fn read(self) -> T
    where
        T: Copy;
}

impl<'a, T> UnsafeCellDeref<'a, T> for &'a UnsafeCell<T> {
    #[inline]
    unsafe fn deref(self) -> &'a T {
        unsafe { &*self.get() }
    }

    #[inline]
    unsafe fn deref_mut(self) -> &'a mut T {
        // SAFETY: The caller upholds the alias rules.
        unsafe { &mut *self.get() }
    }

    #[inline]
    unsafe fn read(self) -> T
    where
        T: Copy,
    {
        unsafe { self.get().read() }
    }
}
