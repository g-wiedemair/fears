use core::cell::UnsafeCell;

/// A value that tracks when a system ran relative to other systems
/// This is used to power change detection
///
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
pub struct Tick {
    tick: u32,
}

impl Tick {
    #[inline]
    pub const fn new(tick: u32) -> Self {
        Self { tick }
    }
}

/// Interior-mutable access to the [`Tick`] for a single component or resource
#[derive(Copy, Clone, Debug)]
pub struct TickCells<'a> {
    /// The tick indicating when the value was added to the world.
    pub added: &'a UnsafeCell<Tick>,
    /// The tick indicating the last time the value was modified.
    pub changed: &'a UnsafeCell<Tick>,
}
