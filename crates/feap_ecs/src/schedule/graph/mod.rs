mod graph_map;
mod tarjan_scc;

pub use graph_map::{DiGraph, Direction, GraphNodeId};

use super::{
    BoxedCondition, Chain, InternedScheduleLabel, InternedSystemSet, IntoScheduleConfigs,
    config::{Schedulable, ScheduleConfig, ScheduleConfigs},
    error::{ScheduleBuildError, ScheduleBuildWarning},
    executor::SystemSchedule,
    node::{NodeId, SystemKey, SystemSetKey, SystemSets, Systems},
    pass::ScheduleBuildPassObj,
};
use crate::{component::ComponentId, system::ScheduleSystem, world::World};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};
use core::any::{Any, TypeId};
use feap_utils::map::TypeIdMap;

/// Metadata about how the node fits in the schedule graph
#[derive(Default)]
pub struct GraphInfo {
    /// The sets that the node belongs to
    pub(crate) hierarchy: Vec<InternedSystemSet>,
    /// The sets that the node depends on (must run before or after)
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) ambiguous_with: Ambiguity,
}

/// Metadata for a [`Schedule`]
/// The order isn't optimized
#[derive(Default)]
pub struct ScheduleGraph {
    /// Container of systems in the schedule
    pub systems: Systems,
    /// Container of system sets in the schedule
    pub system_sets: SystemSets,
    /// Directed acyclic graph of the hierarchy (which systems/sets are children of which sets)
    hierarchy: Dag<NodeId>,
    /// Directed acyclic graph of the dependency (which systems/sets have to run before which other others
    dependency: Dag<NodeId>,

    pub(super) changed: bool,
    passes: BTreeMap<TypeId, Box<dyn ScheduleBuildPassObj>>,
}

impl ScheduleGraph {
    /// Creates an empty [`ScheduleGraph`] with default settings
    pub fn new() -> Self {
        Self {
            systems: Systems::default(),
            system_sets: SystemSets::default(),
            hierarchy: Dag::default(),
            dependency: Dag::default(),
            changed: false,
            passes: BTreeMap::default(),
        }
    }

    /// Adds the config nodes to the graph
    #[track_caller]
    pub(super) fn process_configs<
        T: ProcessScheduleConfig + Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>,
    >(
        &mut self,
        configs: ScheduleConfigs<T>,
        collect_nodes: bool,
    ) -> ProcessConfigsResult {
        match configs {
            ScheduleConfigs::ScheduleConfig(config) => self.process_config(config, collect_nodes),
            ScheduleConfigs::Configs {
                metadata,
                mut configs,
                collective_conditions,
            } => {
                self.apply_collective_conditions(&mut configs, collective_conditions);

                let is_chained = matches!(metadata, Chain::Chained(_));

                // Densely chained if
                // - chained and all configs in the chain are densely chained, or
                // - unchained with a single densely chained config
                let mut densely_chained = is_chained || configs.len() == 1;
                let mut configs = configs.into_iter();
                let mut nodes = Vec::new();

                let Some(first) = configs.next() else {
                    return ProcessConfigsResult {
                        nodes: Vec::new(),
                        densely_chained,
                    };
                };
                let mut previous_result = self.process_configs(first, collect_nodes || is_chained);
                densely_chained &= previous_result.densely_chained;

                for current in configs {
                    let current_result = self.process_configs(current, collect_nodes || is_chained);
                    densely_chained &= current_result.densely_chained;

                    if let Chain::Chained(chain_options) = &metadata {
                        // If the current result is densely chained, we only need to chain the first node
                        let current_nodes = if current_result.densely_chained {
                            &current_result.nodes[..1]
                        } else {
                            &current_result.nodes
                        };
                        // If the previous result was densely chained, we only need to chain the last node
                        let previous_nodes = if previous_result.densely_chained {
                            &previous_result.nodes[previous_result.nodes.len() - 1..]
                        } else {
                            &previous_result.nodes
                        };

                        for previous_node in previous_nodes {
                            for current_node in current_nodes {
                                self.dependency
                                    .graph
                                    .add_edge(*previous_node, *current_node);

                                for pass in self.passes.values_mut() {
                                    pass.add_dependency(
                                        *previous_node,
                                        *current_node,
                                        chain_options,
                                    );
                                }
                            }
                        }
                    }
                    if collect_nodes {
                        todo!()
                    }

                    previous_result = current_result;
                }
                if collect_nodes {
                    todo!()
                }

                ProcessConfigsResult {
                    nodes,
                    densely_chained,
                }
            }
        }
    }

    fn process_config<T: ProcessScheduleConfig + Schedulable>(
        &mut self,
        config: ScheduleConfig<T>,
        collect_nodes: bool,
    ) -> ProcessConfigsResult {
        ProcessConfigsResult {
            densely_chained: true,
            nodes: collect_nodes
                .then_some(T::process_config(self, config))
                .into_iter()
                .collect(),
        }
    }

    fn apply_collective_conditions<
        T: ProcessScheduleConfig + Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>,
    >(
        &mut self,
        configs: &mut [ScheduleConfigs<T>],
        collective_conditions: Vec<BoxedCondition>,
    ) {
        if !collective_conditions.is_empty() {
            todo!()
        }
    }

    /// Add a [`ScheduleConfig`] to the graph, including its dependencies and conditions
    fn add_system_inner(&mut self, config: ScheduleConfig<ScheduleSystem>) -> SystemKey {
        let key = self.systems.insert(config.node, config.conditions);

        // graph updates are immediate
        self.update_graphs(NodeId::System(key), config.metadata);

        key
    }

    #[track_caller]
    pub(super) fn configure_sets<M>(
        &mut self,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) {
        self.process_configs(sets.into_configs(), false);
    }

    /// Add a single `ScheduleConfig` to the graph, including its dependencies and conditions
    fn configure_set_inner(&mut self, config: ScheduleConfig<InternedSystemSet>) -> SystemSetKey {
        let key = self.system_sets.insert(config.node, config.conditions);

        // graph update are immediate
        self.update_graphs(NodeId::Set(key), config.metadata);

        key
    }

    /// Update the internal graphs (hierarchy, dependency, ambiguity) by adding a single [`GraphInfo`]
    fn update_graphs(&mut self, id: NodeId, graph_info: GraphInfo) {
        self.changed = true;

        let GraphInfo {
            hierarchy: sets,
            dependencies,
            ambiguous_with,
            ..
        } = graph_info;

        self.hierarchy.graph.add_node(id);
        self.dependency.graph.add_node(id);

        for key in sets
            .into_iter()
            .map(|set| self.system_sets.get_key_or_insert(set))
        {
            self.hierarchy.graph.add_edge(NodeId::Set(key), id);

            // ensure set also appears in dependency graph
            self.dependency.graph.add_node(NodeId::Set(key));
        }

        for (kind, key, options) in
            dependencies
                .into_iter()
                .map(|Dependency { kind, set, options }| {
                    (kind, self.system_sets.get_key_or_insert(set), options)
                })
        {
            let (lhs, rhs) = match kind {
                DependencyKind::Before => (id, NodeId::Set(key)),
                DependencyKind::After => (NodeId::Set(key), id),
            };
            self.dependency.graph.add_edge(lhs, rhs);
            for pass in self.passes.values_mut() {
                pass.add_dependency(lhs, rhs, &options);
            }

            // ensure set also appears in hierarchy graph
            self.hierarchy.graph.add_node(NodeId::Set(key));
        }

        match ambiguous_with {
            Ambiguity::Check => (),
        }
    }

    /// Initializes any newly-added systems and conditions by calling [`System::initialize`]
    pub fn initialize(&mut self, world: &mut World) {
        self.systems.initialize(world);
        self.system_sets.initialize(world);
    }

    /// Tries to topologically sort `graph`
    /// If the graph is acyclic, returns [`Ok`] with the list of [`NodeId`] in a valid
    /// topological order. If the graph contains cycles, returns [`Err`] with the list of
    /// strongly-connected components that contain cycles (also in a valid topological order)
    /// If the graph contain cycles, then an error is returned
    pub fn topsort_graph<N: GraphNodeId + Into<NodeId>>(
        &self,
        graph: &DiGraph<N>,
        report: ReportCycles,
    ) -> Result<Vec<N>, ScheduleBuildError> {
        // Check explicitly for self-edges
        if let Some((node, _)) = graph.all_edges().find(|(left, right)| left == right) {
            todo!()
        }

        // Tarjan's SCC algorithm returns elements in *reverse* topological order
        let mut top_sorted_nodes = Vec::with_capacity(graph.node_count());
        let mut sccs_with_cycles = Vec::new();
        
        for scc in graph.iter_sccs() {
            // A strongly-connected component is a group of nodes who can all reach other
            // through one or more paths. If an SCC contains more than one node, there must be
            // at least one cycle within them.
            top_sorted_nodes.extend_from_slice(&scc);
            if scc.len() > 1 {
                sccs_with_cycles.push(scc);
            }
        }
        
        todo!()
    }

    /// Builds an execution-optimized [`SystemSchedule`] from the current state of the graph.
    /// Also returns any warnings that were generated during the build process.
    ///
    /// This method also
    /// - checks for dependency or hierarchy cycles
    /// - checks for system access conflicts and reports ambiguities
    pub fn build_schedule(
        &mut self,
        world: &mut World,
        ignored_ambiguities: &BTreeSet<ComponentId>,
    ) -> Result<(SystemSchedule, Vec<ScheduleBuildWarning>), ScheduleBuildError> {
        // let mut warnings = Vec::new();

        // Check hierarchy for cycles
        self.hierarchy.topsort =
            self.topsort_graph(&self.hierarchy.graph, ReportCycles::Hierarchy)?;

        todo!()
    }

    /// Updates the `SystemSchedule` from the `ScheduleGraph`
    pub(super) fn update_schedule(
        &mut self,
        world: &mut World,
        schedule: &mut SystemSchedule,
        ignored_ambiguities: &BTreeSet<ComponentId>,
        schedule_label: InternedScheduleLabel,
    ) -> Result<Vec<ScheduleBuildWarning>, ScheduleBuildError> {
        if !self.systems.is_initialized() || !self.system_sets.is_initialized() {
            return Err(ScheduleBuildError::Uninitialized);
        }

        // Move systems out of old schedule
        for ((key, system), conditions) in schedule
            .system_ids
            .drain(..)
            .zip(schedule.systems.drain(..))
            .zip(schedule.system_conditions.drain(..))
        {
            todo!()
        }

        for (key, conditions) in schedule
            .set_ids
            .drain(..)
            .zip(schedule.set_conditions.drain(..))
        {
            todo!()
        }

        let (new_schedule, warnings) = self.build_schedule(world, ignored_ambiguities)?;
        *schedule = new_schedule;

        for warning in &warnings {
            log::warn!(
                "{:?} schedule built successfully, however: {}",
                schedule_label,
                warning.to_string(self, world)
            );
        }

        todo!()
    }
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
