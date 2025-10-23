use alloc::{vec::Vec, boxed::Box};
use core::{mem::MaybeUninit, ptr::NonNull};

/// Wraps pointers to a [`CommandQueue`], used internally to avoid stacked borrow rules when
/// partially applying the world's command queue recursively
#[derive(Clone)]
pub(crate) struct RawCommandQueue {
    pub(crate) bytes: NonNull<Vec<MaybeUninit<u8>>>,
    pub(crate) cursor: NonNull<usize>,
}

impl RawCommandQueue {
    /// Returns a new `RawCommandQueue` instance, this must be manually dropped
    pub(crate) fn new() -> Self {
        unsafe {
            Self {
                bytes: NonNull::new_unchecked(Box::into_raw(Box::default())),
                cursor: NonNull::new_unchecked(Box::into_raw(Box::new(0usize))),
            }
        }
    }

    /// Returns true if the queue is empty
    pub unsafe fn is_empty(&self) -> bool {
        (unsafe { *self.cursor.as_ref() }) >= (unsafe { self.bytes.as_ref() }).len()
    }
}
