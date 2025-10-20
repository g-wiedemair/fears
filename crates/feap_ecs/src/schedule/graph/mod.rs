mod graph_map;
mod schedule_graph;
mod tarjan_scc;

pub use graph_map::{DiGraph, Direction, GraphNodeId, UnGraph};
pub use schedule_graph::ScheduleGraph;

use super::{
    config::{Schedulable, ScheduleConfig},
    node::NodeId,
    InternedSystemSet,
};
use crate::system::ScheduleSystem;
use alloc::{boxed::Box, vec::Vec};
use core::any::Any;
use feap_core::collections::{HashMap, HashSet};
use feap_utils::map::TypeIdMap;
use fixedbitset::FixedBitSet;

/// Metadata about how the node fits in the schedule graph
#[derive(Default)]
pub struct GraphInfo {
    /// The sets that the node belongs to
    pub(crate) hierarchy: Vec<InternedSystemSet>,
    /// The sets that the node depends on (must run before or after)
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) ambiguous_with: Ambiguity,
}

/// An edge to be added to the dependency graph
pub(crate) struct Dependency {
    pub(crate) kind: DependencyKind,
    pub(crate) set: InternedSystemSet,
    pub(crate) options: TypeIdMap<Box<dyn Any>>,
}

/// Specifies what kind of edge should be added to the dependency graph
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub(crate) enum DependencyKind {
    /// A node that should be preceded
    Before,
    /// A node that should be succeeded
    After,
}

/// Configures ambiguity detection for a single system
#[derive(Clone, Debug, Default)]
pub(crate) enum Ambiguity {
    #[default]
    Check,
}

/// A directed acyclic graph structure
pub struct Dag<N: GraphNodeId> {
    /// A directed graph
    graph: DiGraph<N>,
    /// A cached topological ordering of the graph
    topsort: Vec<N>,
}

impl<N: GraphNodeId> Default for Dag<N> {
    fn default() -> Self {
        Self {
            graph: DiGraph::default(),
            topsort: Vec::new(),
        }
    }
}

/// Values returned by [`ScheduleGraph::process_config`]
pub(super) struct ProcessConfigsResult {
    /// All nodes contained inside this `process_configs` call's [`ScheduleConfigs`] hierarchy,
    /// if `ancestor_chained` is true
    nodes: Vec<NodeId>,
    /// True if and only if all nodes are "densily chained", meaning that all nested nodes
    /// are linearly chained in the order they are defined
    densely_chained: bool,
}

/// Trait used by [`ScheduleGraph::process_config`] to process a single [`ScheduleConfig`]
pub(super) trait ProcessScheduleConfig: Schedulable + Sized {
    /// Process a single [`ScheduleConfig`]
    fn process_config(schedule_graph: &mut ScheduleGraph, config: ScheduleConfig<Self>) -> NodeId;
}

impl ProcessScheduleConfig for ScheduleSystem {
    fn process_config(schedule_graph: &mut ScheduleGraph, config: ScheduleConfig<Self>) -> NodeId {
        NodeId::System(schedule_graph.add_system_inner(config))
    }
}

impl ProcessScheduleConfig for InternedSystemSet {
    fn process_config(schedule_graph: &mut ScheduleGraph, config: ScheduleConfig<Self>) -> NodeId {
        NodeId::Set(schedule_graph.configure_set_inner(config))
    }
}

/// Used to select the appropriate reporting function
pub enum ReportCycles {
    /// When sets contain themselves
    Hierarchy,
    /// When the graph is no longer a DAG
    Dependency,
}

/// Stores the results of the graph analysis
pub(crate) struct CheckGraphResults<N: GraphNodeId> {
    /// Boolean reachability matrix for the graph
    pub(crate) reachable: FixedBitSet,
    /// Pairs of nodes that have a path connecting them
    pub(crate) connected: HashSet<(N, N)>,
    /// Pairs of nodes that don't have a path connecting them
    pub(crate) disconnected: Vec<(N, N)>,
    /// Edges that are redundant because a longer path exists
    pub(crate) transitive_edges: Vec<(N, N)>,
    /// Variant of the graph with no transitive edges
    pub(crate) transitive_reduction: DiGraph<N>,
    /// Variant of the graph with all possible transitive edges
    pub(crate) transitive_closure: DiGraph<N>,
}

impl<N: GraphNodeId> Default for CheckGraphResults<N> {
    fn default() -> Self {
        Self {
            reachable: FixedBitSet::new(),
            connected: HashSet::default(),
            disconnected: Vec::new(),
            transitive_edges: Vec::new(),
            transitive_reduction: DiGraph::default(),
            transitive_closure: DiGraph::default(),
        }
    }
}

/// Converts 2D row-major pair of indices into a 1D array index.
pub(crate) fn index(row: usize, col: usize, num_cols: usize) -> usize {
    debug_assert!(col < num_cols);
    (row * num_cols) + col
}

/// Converts a 1D array index into a 2D row-major pair of indices.
pub(crate) fn row_col(index: usize, num_cols: usize) -> (usize, usize) {
    (index / num_cols, index % num_cols)
}

/// Processes a DAG and computes its:
/// - transitive reduction (along with the set of removed edges)
/// - transitive closure
/// - reachability matrix (as a bitset)
/// - pairs of nodes connected by a path
/// - pairs of nodes not connected by a path
///
pub(crate) fn check_graph<N: GraphNodeId>(
    graph: &DiGraph<N>,
    topological_order: &[N],
) -> CheckGraphResults<N> {
    if graph.node_count() == 0 {
        return CheckGraphResults::default();
    }

    let n = graph.node_count();

    // Build a copy of the graph where the nodes and edges appear in topsorted order
    let mut map = <HashMap<_, _>>::with_capacity_and_hasher(n, Default::default());
    let mut topsorted = DiGraph::<N>::default();
    // Iterate nodes in topological order
    for (i, &node) in topological_order.iter().enumerate() {
        map.insert(node, i);
        topsorted.add_node(node);
        // Insert nodes as successors to their predecessors
        for pred in graph.neighbors_directed(node, Direction::Incoming) {
            topsorted.add_edge(pred, node);
        }
    }

    let mut reachable = FixedBitSet::with_capacity(n * n);
    let mut connected = <HashSet<_>>::default();
    let mut disconnected = Vec::new();

    let mut transitive_edges = Vec::new();
    let mut transitive_reduction = DiGraph::default();
    let mut transitive_closure = DiGraph::default();

    let mut visited = FixedBitSet::with_capacity(n);

    // Iterate nodes in topological order
    for node in topsorted.nodes() {
        transitive_reduction.add_node(node);
        transitive_closure.add_node(node);
    }

    // Iterate nodes in reverse topological order
    for a in topsorted.nodes().rev() {
        let index_a = *map.get(&a).unwrap();
        for b in topsorted.neighbors_directed(a, Direction::Outgoing) {
            let index_b = *map.get(&b).unwrap();
            debug_assert!(index_a < index_b);
            if !visited[index_b] {
                // Edge <a, b> is not redundant
                transitive_reduction.add_edge(a, b);
                transitive_closure.add_edge(a, b);
                reachable.insert(index(index_a, index_b, n));

                let successors = transitive_closure
                    .neighbors_directed(b, Direction::Outgoing)
                    .collect::<Vec<_>>();
                for c in successors {
                    let index_c = *map.get(&c).unwrap();
                    debug_assert!(index_b < index_c);
                    if !visited[index_c] {
                        visited.insert(index_c);
                        transitive_closure.add_edge(a, c);
                        reachable.insert(index(index_a, index_c, n));
                    }
                }
            } else {
                // Edge <a, b> is redundant
                transitive_edges.push((a, b));
            }
        }

        visited.clear();
    }

    // Partition pairs of nodes into "connected by path" and "not connected by path"
    for i in 0..(n - 1) {
        // Reachable is upper triangular because the nodes were topsorted
        for index in index(i, i + 1, n)..=index(i, n - 1, n) {
            let (a, b) = row_col(index, n);
            let pair = (topological_order[a], topological_order[b]);
            if reachable[index] {
                connected.insert(pair);
            } else {
                disconnected.push(pair);
            }
        }
    }

    CheckGraphResults {
        reachable,
        connected,
        disconnected,
        transitive_edges,
        transitive_reduction,
        transitive_closure,
    }
}
