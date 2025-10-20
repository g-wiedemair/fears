use crate::schedule::graph::{DiGraph, GraphNodeId};
use alloc::vec::Vec;
use core::{hash::BuildHasher, num::NonZeroUsize};
use smallvec::SmallVec;

/// Create an iterator over *strongly connected components* using Algorithm 3 in
/// [A Space-Efficient Algorithm for Finding Strongly Connected Components][1] by David J. Pierce,
/// which is a memory-efficient variation of [Tarjan's alogrithm][2].
///
/// [1]: https://homepages.ecs.vuw.ac.nz/~djp/files/P05.pdf
/// [2]: https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm
///
/// Returns each strongly connected component (scc).
/// The order of node idds within each scc is arbitrary, but the order of
/// the sccs is their postorder (reverse topological sort).
pub(crate) fn new_tarjan_scc<N: GraphNodeId, S: BuildHasher>(
    graph: &DiGraph<N, S>,
) -> impl Iterator<Item = SmallVec<[N; 4]>> + '_ {
    // Create a list of all nodes we need to visit
    let unchecked_nodes = graph.nodes();

    // For each node we need to visit, we also need to visit its neighbors
    // Storing the iterator for each set of neighbors allows this list to be computed without
    // an additional allocation
    let nodes = graph
        .nodes()
        .map(|node| NodeData {
            root_index: None,
            neighbors: graph.neighbors(node),
        })
        .collect::<Vec<_>>();

    TarjanScc {
        graph,
        unchecked_nodes,
        index: 1,
        component_count: usize::MAX, // Will hold if component_count is initialized to number of nodes - 1 or higher
        nodes,
        stack: Vec::new(),
        visitation_stack: Vec::new(),
        start: None,
        index_adjustment: None,
    }
}

struct NodeData<Neighbors: Iterator<Item: GraphNodeId>> {
    root_index: Option<NonZeroUsize>,
    neighbors: Neighbors,
}

/// A state for computing the *strongly connected components* using [Tarjan's algorithm][1]
/// This is based on [`TarjanScc`] from [`petgraph`]
struct TarjanScc<'graph, N, Hasher, AllNodes, Neighbors>
where
    N: GraphNodeId,
    Hasher: BuildHasher,
    AllNodes: Iterator<Item = N>,
    Neighbors: Iterator<Item = N>,
{
    /// Source of truth [`DiGraph`]
    graph: &'graph DiGraph<N, Hasher>,
    /// An [`Iterator`] of [`GraphNodeId`]s from the `graph` which may not have been visited yet
    unchecked_nodes: AllNodes,
    /// The index of the next SCC
    index: usize,
    /// A count of potentially remaining SCCs
    component_count: usize,
    /// Information about each [`GraphNodeId`], including a possible SCC index and an
    /// [`Iterator`] of possibly unvisited neighbors
    nodes: Vec<NodeData<Neighbors>>,
    /// A stack of [`GraphNodeId`]s where an SCC will be found starting at the top of the stack
    stack: Vec<N>,
    /// A stack of [`GraphNodeId`]s which need to be visited to determine which SCC they belong to
    visitation_stack: Vec<(N, bool)>,
    /// An index into the `stack` indicating the starting point of an SCC
    start: Option<usize>,
    /// An adjustment to the `index` which will be applied once the current SCC is found
    index_adjustment: Option<usize>,
}

impl<'graph, N: GraphNodeId, S: BuildHasher, A: Iterator<Item = N>, Neighbors: Iterator<Item = N>>
    TarjanScc<'graph, N, S, A, Neighbors>
{
    /// Returns `Some` for each strongly connected component (scc).
    /// The order of node ids within each scc is arbitrary, but the order of
    /// the SCCs is their postorder (reverse topological sort).
    fn next_scc(&mut self) -> Option<&[N]> {
        // Cleanup from possible previous iteration
        if let (Some(start), Some(index_adjustment)) =
            (self.start.take(), self.index_adjustment.take())
        {
            self.stack.truncate(start);
            self.index -= index_adjustment;
            self.component_count -= 1;
        }

        loop {
            // If there are items on the visitation stack, then we haven't finished visiting
            // the node at the bottom of the stack yet.
            // Must visit all nodes in the stack from top to bottom before visiting the next node
            while let Some((v, v_is_local_root)) = self.visitation_stack.pop() {
                // If this visitation finds a complete SCC, return it immediately
                if let Some(start) = self.visit_once(v, v_is_local_root) {
                    return Some(&self.stack[start..]);
                }
            }

            // Get the next node to check, otherwise we're done and can return None
            let Some(node) = self.unchecked_nodes.next() else {
                break None;
            };

            let visited = self.nodes[self.graph.to_index(node)].root_index.is_some();

            // If this node hasn't already been visited (e.g., it was the neighbor of a previously
            // checked node) add it to the visitation stack
            if !visited {
                self.visitation_stack.push((node, true));
            }
        }
    }

    // Attempt to find the starting point on the stack for a new SCC without visiting nieghbors.
    // If a visitation is required, this will return `None` and mark the required neighbor and the
    // current node as in need of visitation again.
    // if no SCC can be found in the current visitation stack, returns `None`
    fn visit_once(&mut self, v: N, mut v_is_local_root: bool) -> Option<usize> {
        let node_v = &mut self.nodes[self.graph.to_index(v)];

        if node_v.root_index.is_none() {
            let v_index = self.index;
            node_v.root_index = NonZeroUsize::new(v_index);
            self.index += 1;
        }

        while let Some(w) = self.nodes[self.graph.to_index(v)].neighbors.next() {
            // If a neighbor hasn't been visited yet...
            if self.nodes[self.graph.to_index(w)].root_index.is_none() {
                // Push the current node and the neighbor back onto the visitation stack.
                // On the next execution of `visit_once`, the neighbor will be visited.
                self.visitation_stack.push((v, v_is_local_root));
                self.visitation_stack.push((w, true));

                return None;
            }

            if self.nodes[self.graph.to_index(w)].root_index
                < self.nodes[self.graph.to_index(v)].root_index
            {
                self.nodes[self.graph.to_index(v)].root_index =
                    self.nodes[self.graph.to_index(w)].root_index;
                v_is_local_root = false;
            }
        }

        if !v_is_local_root {
            todo!()
        }

        // Pop the stack and generate an SCC
        let mut index_adjustment = 1;
        let c = NonZeroUsize::new(self.component_count);
        let nodes = &mut self.nodes;
        let start = self
            .stack
            .iter()
            .rposition(|&w| {
                if nodes[self.graph.to_index(v)].root_index
                    > nodes[self.graph.to_index(w)].root_index
                {
                    true
                } else {
                    nodes[self.graph.to_index(w)].root_index = c;
                    index_adjustment += 1;
                    false
                }
            })
            .map(|x| x + 1)
            .unwrap_or_default();
        nodes[self.graph.to_index(v)].root_index = c;
        self.stack.push(v);

        self.start = Some(start);
        self.index_adjustment = Some(index_adjustment);

        Some(start)
    }
}

impl<'graph, N: GraphNodeId, S: BuildHasher, A: Iterator<Item = N>, NeighBors: Iterator<Item = N>>
    Iterator for TarjanScc<'graph, N, S, A, NeighBors>
{
    // It is expected that the `DiGraph` is sparse, and as such wont have many large SCCs
    // Returning a `SmallVec` allows this iterator to skip allocation in cases where that
    // assumption holds
    type Item = SmallVec<[N; 4]>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = SmallVec::from_slice(self.next_scc()?);
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // There can be no more than the number of nodes in a graph worth of SCCs
        (0, Some(self.nodes.len()))
    }
}
