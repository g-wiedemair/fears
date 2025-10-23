//! A reimplementation of the currently unstable [`std::sync::Exclusive`]

/// See [`Exclusive`] for stdlib's upcoming implementation, which should replace this
#[repr(transparent)]
pub struct SyncCell<T: ?Sized> {
    inner: T,
}

impl<T: Sized> SyncCell<T> {
    /// Construct a new instance of a `SyncCell` from the given value
    pub fn new(inner: T) -> Self {
        Self {inner}
    }
}

impl<T: ?Sized> SyncCell<T> {
    /// Get a reference to this `SyncCell`s inner value
    pub fn get(&mut self) -> &mut T {
        &mut self.inner
    }
}

unsafe impl<T: ?Sized> Sync for SyncCell<T> {}
