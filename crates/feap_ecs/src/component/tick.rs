use crate::event::Event;
use core::cell::UnsafeCell;

/// The (arbitrarily chosen) minimum number of world tick increments between `check_tick` scans.
///
/// Change ticks can only be scanned when systems aren't running. Thus, if the threshold is `N`,
/// the maximum is `2 * N - 1` (i.e. the world ticks `N - 1` times, then `N` times).
///
/// If no change is older than `u32::MAX - (2 * N - 1)` following a scan, none of their ages can
/// overflow and cause false positives.
pub const CHECK_TICK_THRESHOLD: u32 = 518_400_000;

/// The maximum change tick difference that won't overflow before the next `check_tick` scan.
///
/// Changes stop being detected once they become this old.
pub const MAX_CHANGE_AGE: u32 = u32::MAX - (2 * CHECK_TICK_THRESHOLD - 1);

/// A value that tracks when a system ran relative to other systems
/// This is used to power change detection
///
#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
pub struct Tick {
    tick: u32,
}

impl Tick {
    /// The maximum relative age for a change tick
    /// The value of this is equal to [`MAX_CHANGE_AGE`]
    pub const MAX: Self = Self::new(MAX_CHANGE_AGE);

    #[inline]
    pub const fn new(tick: u32) -> Self {
        Self { tick }
    }

    /// Gets the value of this given tick
    #[inline]
    pub const fn get(self) -> u32 {
        self.tick
    }

    /// Returns a change tick representing the relationship between `self` and `other`
    #[inline]
    pub fn relative_to(self, other: Self) -> Self {
        let tick = self.tick.wrapping_sub(other.tick);
        Self { tick }
    }

    /// Wraps this change tick's value if it exceeds [`Tick::MAX`]
    #[inline]
    pub fn check_tick(&mut self, check: CheckChangeTicks) -> bool {
        let age = check.present_tick().relative_to(*self);
        if age.get() > Self::MAX.get() {
            todo!()
        } else {
            false
        }
    }
}

/// An [`Event`] that can be used to maintain [`Tick`]s in custom data structures, enabling to make
/// use of feap's periodic checks that clamps ticks to a certain range, preventing overflows and thus
/// keeping methods like [`Tick::is_newer_than`] reliably return `false` for ticks that got too old.
#[derive(Debug, Clone, Copy, Event)]
pub struct CheckChangeTicks(pub(crate) Tick);

impl CheckChangeTicks {
    /// Gets the present `Tick` that other ticks get compared to
    pub fn present_tick(self) -> Tick {
        self.0
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

/// Records when a component or resource was added and when it was last mutably dereferenced
#[derive(Copy, Clone, Debug)]
pub struct ComponentTicks {
    /// Tick recording the time this component or resource was added
    pub added: Tick,
    /// Tick recording the time this component or resource was most recently changed
    pub changed: Tick,
}
