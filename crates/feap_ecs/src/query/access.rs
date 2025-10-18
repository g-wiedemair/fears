/// A collection of [`FilteredAccess`] instances
///
/// Used internally to statically check if system have conflicting access
/// It stores multiple sets of accesses
/// - A "combined" set, which is the access of all filters in this set combined
/// - The set of access of each individual filter in this set
#[derive(Debug, PartialEq, Eq, Default)]
pub struct FilteredAccessSet {}

impl FilteredAccessSet {
    /// Creates a new empty [`FilteredAccessSet`]
    pub const fn new() -> Self { 
        FilteredAccessSet {} 
    }
}
