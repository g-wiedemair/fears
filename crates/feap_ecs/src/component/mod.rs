mod clone;
mod info;
mod register;
mod tick;

pub use info::*;
pub use register::*;
pub use tick::*;

/// The storage used for a specific component type
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum StorageType {
    /// Provides fast and cache-friendly iteration, but slower addition and removal of components
    #[default]
    Table,
    /// Provides fast addition and removal of components, but slower iteration
    SparseSet,
}
