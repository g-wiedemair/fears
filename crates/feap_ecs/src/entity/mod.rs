mod map_entities;

pub use map_entities::*;

use crate::{
    change_detection::MaybeLocation,
    component::{CheckChangeTicks, Tick},
};
use alloc::vec::Vec;
use core::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
    hash::Hasher,
};
use derive_more::derive::Display;
#[cfg(target_has_atomic = "64")]
use feap_core::sync::atomic::AtomicI64 as AtomicIdCursor;
use nonmax::NonMaxU32;

/// This represents the row or `index` of an [`Entity`] within the [`Entities`] table.
/// This is a lighter weight version of [`Entity`]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[repr(transparent)]
pub struct EntityRow(NonMaxU32);

impl EntityRow {
    const PLACEHOLDER: Self = Self(NonMaxU32::MAX);

    /// Gets some bits that represent this value
    #[inline(always)]
    const fn to_bits(self) -> u32 {
        unsafe { core::mem::transmute::<NonMaxU32, u32>(self.0) }
    }

    /// Gets the index of the entity
    #[inline(always)]
    pub const fn index(self) -> u32 {
        self.0.get()
    }
}

/// This tracks different versions or generations of an [`EntityRow`]
/// Importantly, this can wrap, meaning each generation is not necessarily unique
/// This should be treated as a opaque identifier, and its internal representation may be subject to change
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub struct EntityGeneration(u32);

impl EntityGeneration {
    /// Represents the first generation of an [`EntityRow`]
    pub const FIRST: Self = Self(0);

    /// Gets some bits that represent this value
    #[inline(always)]
    pub const fn to_bits(self) -> u32 {
        self.0
    }
}

/// Lightweight identifier of an [`Entity`]
///
/// The identifier is implemented using a [generational index}: a combination of an index ([`EntityRow`])
/// and a generation ([`EntityGeneration`])
/// This allows fast insertion after data removal in an array while minimizing loss of spatial locality
#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct Entity {
    #[cfg(target_endian = "little")]
    row: EntityRow,
    generation: EntityGeneration,
    #[cfg(target_endian = "big")]
    row: EntityRow,
}

impl PartialEq for Entity {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl Eq for Entity {}

impl PartialOrd for Entity {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entity {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_bits().cmp(&other.to_bits())
    }
}

impl Hash for Entity {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state)
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self == &Self::PLACEHOLDER {
            write!(f, "PLACEHOLDER")
        } else {
            write!(f, "{}v{}", self.index(), self.generation())
        }
    }
}

impl Entity {
    /// An entity ID with a placeholder value.
    pub const PLACEHOLDER: Self = Self::from_row(EntityRow::PLACEHOLDER);

    /// Creates a new instance with the given index and generation
    #[inline(always)]
    pub const fn from_row_and_generation(row: EntityRow, generation: EntityGeneration) -> Entity {
        Self { row, generation }
    }

    /// Creates a new entity ID with the specified `row` and a generation of 1
    #[inline(always)]
    pub const fn from_row(row: EntityRow) -> Entity {
        Self::from_row_and_generation(row, EntityGeneration::FIRST)
    }

    /// Convert to a form convenient for passing outside of rust
    /// Only useful for identifying entities within the same instance of an application
    #[inline(always)]
    pub const fn to_bits(self) -> u64 {
        self.row.to_bits() as u64 | ((self.generation.to_bits() as u64) << 32)
    }

    /// Return a transiently unique identifier
    #[inline]
    pub const fn row(self) -> EntityRow {
        self.row
    }

    /// Equivalent to `self.row().index()`
    #[inline]
    pub const fn index(self) -> u32 {
        self.row.index()
    }

    /// Returns the generation of this Entity`s row.
    #[inline]
    pub const fn generation(self) -> EntityGeneration {
        self.generation
    }
}

/// A [`World`]s internal metadata store on all of its entities
///
/// Contains metadata on:
///  - The generation of every entity
///  - The alive/dead status of a particular entity
///  - The location of the entity's components im memory
///
#[derive(Debug)]
pub struct Entities {
    meta: Vec<EntityMeta>,

    /// The `pending` and `free_cursor` fields describe three sets of Entity IDs
    /// that have been freed or are in the process of being allocated:
    /// - The `freelist` IDs, previously freed by `free()`. Allocation will always prefer these over brand new IDs
    /// - The `reserved` list of IDs that were once in the freelist, but got reserved. The are waiting for `flush` to make them fully allocated
    /// - The count of new IDs that do not yet exist in `self.meta`, but which we have handed out and reserved.
    /// The contents of `pending` look like this:
    ///
    /// ```txt
    /// ----------------------------
    /// |  freelist  |  reserved   |
    /// ----------------------------
    ///              ^             ^
    ///          free_cursor   pending.len()
    /// ```
    ///
    pending: Vec<EntityRow>,
    free_cursor: AtomicIdCursor,
}

impl Entities {
    pub(crate) const fn new() -> Self {
        Entities {
            meta: Vec::new(),
            pending: Vec::new(),
            free_cursor: AtomicIdCursor::new(0),
        }
    }

    /// Allocates space for entities previously reserved with [`reserve_entity`],
    /// then initializes each one using the supplied function
    ///
    pub unsafe fn flush(
        &mut self,
        _init: impl FnMut(Entity, &mut EntityIdLocation),
        _by: MaybeLocation,
        _tick: Tick,
    ) {
        let free_cursor = self.free_cursor.get_mut();
        let current_free_cursor = *free_cursor;

        let new_free_cursor = if current_free_cursor >= 0 {
            current_free_cursor as usize
        } else {
            todo!()
        };

        for _row in self.pending.drain(new_free_cursor..) {
            todo!()
        }
    }

    #[inline]
    pub(crate) fn check_change_ticks(&mut self, _check: CheckChangeTicks) {
        for _meta in &mut self.meta {
            todo!()
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct EntityMeta {}

/// A location of an entity in an archetype
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EntityLocation {}

/// An [`Entity`] id may or may not correspond to a valid conceptual entity
/// If it does, the conceptual entity may or may not have a location
/// If it has no location, the [`EntityLocation`] will be `None`
pub type EntityIdLocation = Option<EntityLocation>;
