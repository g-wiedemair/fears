use alloc::vec::Vec;
use core::{
    fmt::Debug,
    hash::{BuildHasher, Hash},
};
use feap_core::{collections::HashSet, hash::FixedHasher};
use indexmap::IndexMap;

/// Types that can be used as node identifiers in a [`DiGraph`]/[`UnGraph`]
pub trait GraphNodeId: Copy + Eq + Hash + Ord + Debug {
    /// The type that packs und unpacks this [`GraphNodeId`] with a [`Direction`]
    /// This is used to save space in the graph's adjacency list
    type Adjacent: Copy + Debug + From<(Self, Direction)> + Into<(Self, Direction)>;
    /// The type that packs and unpacks this [`GraphNodeId`] with another
    /// [`GraphNodeId`]. This is used to save space in the graph's edge list
    type Edge: Copy + Eq + Hash + Debug + From<(Self, Self)> + Into<(Self, Self)>;
}

/// Edge direction
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    /// An `Outgoing` edge is an outward edge *from* the current node
    Outgoing = 0,
    /// An `Incoming` edge is an inbound edge *to* the current node
    Incoming = 1,
}

/// A `Graph` with directed edges of some [`GraphNodeId`] `N`
///
/// For example, an edge from *1* to *2* is distinct from an edge from *2* to *1*
pub type DiGraph<N, S = FixedHasher> = Graph<true, N, S>;

/// `Graph<DIRECTED>` is a graph datastructure using an associative array
/// of its node weights of some [`GraphNodeId`]
///
/// It uses a combined adjacency list and sparse adjacency matrix
/// representation, using **O(|N| + |E|)** space, and allows testing for edge
/// existence in constant time.
///
/// `Graph` is parameterized over:
/// - Constant generic bool `DIRECTED` determines whether the graph edges are directed or undirected
/// - The `GraphNodeId` type `N`, which is used as the node weight
/// - The `BuildHasher` `S`
#[derive(Clone)]
pub struct Graph<const DIRECTED: bool, N: GraphNodeId, S = FixedHasher>
where
    S: BuildHasher,
{
    nodes: IndexMap<N, Vec<N::Adjacent>, S>,
    edges: HashSet<N::Edge, S>,
}

impl<const DIRECTED: bool, N, S> Default for Graph<DIRECTED, N, S>
where
    N: GraphNodeId,
    S: BuildHasher + Default,
{
    fn default() -> Self {
        Self::with_capacity(0, 0)
    }
}

impl<const DIRECTED: bool, N: GraphNodeId, S: BuildHasher> Graph<DIRECTED, N, S> {
    /// Creates a new `Graph` with estimated capacity
    pub fn with_capacity(nodes: usize, edges: usize) -> Self
    where
        S: Default,
    {
        Self {
            nodes: IndexMap::with_capacity_and_hasher(nodes, S::default()),
            edges: HashSet::with_capacity_and_hasher(edges, S::default()),
        }
    }

    /// Use their natural order to map the node pair (a, b) to a canonical edge it
    #[inline]
    fn edge_key(a: N, b: N) -> N::Edge {
        let (a, b) = if DIRECTED || a <= b { (a, b) } else { (b, a) };
        N::Edge::from((a, b))
    }

    /// Add node `n` from the grapph
    pub fn add_node(&mut self, n: N) {
        self.nodes.entry(n).or_default();
    }

    /// Add an edge connecting `a` and `b` to the graph
    /// For a directed graph, the edge is directed form `a` to `b`
    pub fn add_edge(&mut self, a: N, b: N) {
        if self.edges.insert(Self::edge_key(a, b)) {
            self.nodes
                .entry(a)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(N::Adjacent::from((b, Direction::Outgoing)));
            if a != b {
                // self loops don't have the Incoming entry
                self.nodes
                .entry(b).or_insert_with(|| Vec::with_capacity(1))
                    .push(N::Adjacent::from((a, Direction::Incoming)));
            }
        }
    }
    
}
