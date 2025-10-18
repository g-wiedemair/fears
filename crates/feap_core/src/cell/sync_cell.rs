//! A reimplementation of the currently unstable [`std::sync::Exclusive`]

/// See [`Exclusive`] for stdlib's upcoming implementation, which should replace this
#[repr(transparent)]
pub struct SyncCell<T: ?Sized> {
    inner: T,
}

unsafe impl<T: ?Sized> Sync for SyncCell<T> {}
