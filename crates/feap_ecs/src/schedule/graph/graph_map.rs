use alloc::vec::Vec;
use core::{
    fmt::Debug,
    hash::{BuildHasher, Hash},
};
use feap_core::{collections::HashSet, hash::FixedHasher};
use indexmap::IndexMap;
use smallvec::SmallVec;

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

/// A `Graph` with undirected edges of some [`GraphNodeId`] `N`
///
/// For example, an edge between *1* and *2* is equivalent to an edge between *2* and *1*
pub type UnGraph<N, S = FixedHasher> = Graph<false, N, S>;

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

    /// Returns an iterator over the nodes of the graph
    pub fn nodes(&self) -> impl DoubleEndedIterator<Item = N> + ExactSizeIterator<Item = N> + '_ {
        self.nodes.keys().copied()
    }

    /// Return the number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns an iterator of all nodes with an edge starting from `a`
    pub fn neighbors(&self, a: N) -> impl DoubleEndedIterator<Item = N> + '_ {
        let iter = match self.nodes.get(&a) {
            Some(neigh) => neigh.iter(),
            None => [].iter(),
        };

        iter.copied()
            .map(N::Adjacent::into)
            .filter_map(|(n, dir)| (!DIRECTED || dir == Direction::Outgoing).then_some(n))
    }

    /// Returns an iterator of all neighbors that have an edge between them and `a`, in the specified direction
    pub fn neighbors_directed(
        &self,
        a: N,
        dir: Direction,
    ) -> impl DoubleEndedIterator<Item = N> + '_ {
        let iter = match self.nodes.get(&a) {
            Some(neigh) => neigh.iter(),
            None => [].iter(),
        };

        iter.copied()
            .map(N::Adjacent::into)
            .filter_map(move |(n, d)| (!DIRECTED || d == dir || n == a).then_some(n))
    }

    /// Return an iterator over all edges of the graph with their weight in arbitrary order
    pub fn all_edges(&self) -> impl ExactSizeIterator<Item = (N, N)> + '_ {
        self.edges.iter().copied().map(N::Edge::into)
    }

    pub(crate) fn to_index(&self, ix: N) -> usize {
        self.nodes.get_index_of(&ix).unwrap()
    }

    /// Converts from one [`GraphNodeId`] type to another.
    pub fn try_into<T: GraphNodeId + TryFrom<N>>(self) -> Result<Graph<DIRECTED, T, S>, T::Error>
    where
        S: Default,
    {
        // Converts the node key and every adjacency list entry from `N` to `T`
        fn try_convert_node<N: GraphNodeId, T: GraphNodeId + TryFrom<N>>(
            (key, adj): (N, Vec<N::Adjacent>),
        ) -> Result<(T, Vec<T::Adjacent>), T::Error> {
            let key = key.try_into()?;
            let adj = adj
                .into_iter()
                .map(|node| {
                    let (id, dir) = node.into();
                    Ok(T::Adjacent::from((id.try_into()?, dir)))
                })
                .collect::<Result<_, T::Error>>()?;
            Ok((key, adj))
        }

        // Unpacks the edge pair, converts the nodes from `N` to `T`, and repacks them
        fn try_convert_edge<N: GraphNodeId, T: GraphNodeId + TryFrom<N>>(
            edge: N::Edge,
        ) -> Result<T::Edge, T::Error> {
            let (a, b) = edge.into();
            Ok(T::Edge::from((a.try_into()?, b.try_into()?)))
        }

        let nodes = self
            .nodes
            .into_iter()
            .map(try_convert_node::<N, T>)
            .collect::<Result<_, T::Error>>()?;
        let edges = self
            .edges
            .into_iter()
            .map(try_convert_edge::<N, T>)
            .collect::<Result<_, T::Error>>()?;
        Ok(Graph { nodes, edges })
    }

    /// Add node `n` from the grapph
    pub fn add_node(&mut self, n: N) {
        self.nodes.entry(n).or_default();
    }

    /// Remove a node `n` from the graph
    pub fn remove_node(&mut self, n: N) {
        let Some(links) = self.nodes.swap_remove(&n) else {
            return;
        };

        let links = links.into_iter().map(N::Adjacent::into);

        for (succ, dir) in links {
            let edge = if dir == Direction::Outgoing {
                Self::edge_key(n, succ)
            } else {
                Self::edge_key(succ, n)
            };
            // Remove all successor links
            todo!()
        }
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
                    .entry(b)
                    .or_insert_with(|| Vec::with_capacity(1))
                    .push(N::Adjacent::from((a, Direction::Incoming)));
            }
        }
    }
}

impl<N: GraphNodeId, S: BuildHasher> DiGraph<N, S> {
    /// Iterate over all *Strongly Connected Components* in this graph
    pub(crate) fn iter_sccs(&self) -> impl Iterator<Item = SmallVec<[N; 4]>> + '_ {
        super::tarjan_scc::new_tarjan_scc(self)
    }
}
