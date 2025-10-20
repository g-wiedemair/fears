use crate::{
    schedule::{ScheduleGraph, node::NodeId},
    world::World,
};
use alloc::{string::String, vec::Vec};

/// Category of errors encountered during [`Schedule::initialize`]
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum ScheduleBuildError {
    #[error("`{0:?}` and `{1:?}` have both `in_set` and `before`-`after` relationships (these might be transitive). This combination is unsolvable as a system cannot run before or after a set it belongs to.")]
    CrossDependency(NodeId, NodeId),
    #[error("Tried to run a schedule before all of its systems have been initialized.")]
    Uninitialized,
    #[error(transparent)]
    Elevated(#[from] ScheduleBuildWarning)
}

impl ScheduleBuildError {
    /// Renders the error as a human-readable string with node identifiers
    /// replaced with their names.
    ///
    /// The given `graph` and `world` are used to resolve the names of the nodes
    /// and components involved in the error. The same `graph` and `world`
    /// should be used as those used to [`initialize`] the [`Schedule`].
    /// Failure to do so will result in incorrect or incomplete error messages
    pub fn to_string(&self, graph: &ScheduleGraph, world: &World) -> String {
        todo!()
    }
}

/// Category of warnings encountered during [`Schedule::initialize`]
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum ScheduleBuildWarning {
    /// The hierarchy of system sets contains redundant edges
    /// This warning is **enabled** by default, but can be disabled
    #[error("The hierarchy of system sets contains redundant edges: {0:?}")]
    HierarchyRedundancy(Vec<(NodeId, NodeId)>),
}

impl ScheduleBuildWarning {
    /// Renders the warning as a human readable string with node identifiers
    /// replaced with their names
    pub fn to_string(&self, graph: &ScheduleGraph, world: &World) -> String {
        // match self {
        //     Self::HierarchRedundancy(transitive_edges) => {
        //         ScheduleBuildError::hierarchy_redundancy_to_string(transitive_edges, graph)
        //     }
        // }
        todo!()
    }
}
