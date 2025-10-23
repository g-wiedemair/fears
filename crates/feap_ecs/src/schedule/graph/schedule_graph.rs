use super::{
    check_graph, Ambiguity, CheckGraphResults, Dag, Dependency, DependencyKind, DiGraph, Direction,
    GraphNodeId, ProcessConfigsResult, ProcessScheduleConfig, ReportCycles, UnGraph,
};
use crate::{
    component::ComponentId,
    schedule::{
        config::{Schedulable, ScheduleConfig, ScheduleConfigs}, error::{ScheduleBuildError, ScheduleBuildWarning}, executor::SystemSchedule, node::{NodeId, SystemKey, SystemSetKey, SystemSets, Systems}, pass::ScheduleBuildPassObj,
        BoxedCondition,
        Chain,
        GraphInfo,
        InternedScheduleLabel,
        InternedSystemSet,
        IntoScheduleConfigs,
    },
    system::ScheduleSystem,
    world::World,
};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec,
    vec::Vec,
};
use core::any::TypeId;
use feap_core::collections::{HashMap, HashSet};
use fixedbitset::FixedBitSet;

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
    /// Map of systems in each set
    set_systems: HashMap<SystemSetKey, Vec<SystemKey>>,
    ambiguous_with: UnGraph<NodeId>,
    conflicting_systems: Vec<(SystemKey, SystemKey, Vec<ComponentId>)>,
    pub(crate) changed: bool,
    settings: ScheduleBuildSettings,
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
            set_systems: HashMap::default(),
            ambiguous_with: UnGraph::default(),
            conflicting_systems: Vec::new(),
            changed: false,
            settings: ScheduleBuildSettings::default(),
            passes: BTreeMap::default(),
        }
    }

    /// Returns the name of the node with the given [`NodeId`].
    /// Resolves anonymous sets to a string that describes their contents
    pub fn get_node_name(&self, id: &NodeId) -> String {
        self.get_node_name_inner(id, self.settings.report_sets)
    }

    #[inline]
    fn get_node_name_inner(&self, id: &NodeId, report_sets: bool) -> String {
        todo!()
    }

    #[track_caller]
    pub(crate) fn configure_sets<M>(
        &mut self,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) {
        self.process_configs(sets.into_configs(), false);
    }

    /// Add a single `ScheduleConfig` to the graph, including its dependencies and conditions
    pub(super) fn configure_set_inner(
        &mut self,
        config: ScheduleConfig<InternedSystemSet>,
    ) -> SystemSetKey {
        let key = self.system_sets.insert(config.node, config.conditions);

        // graph update are immediate
        self.update_graphs(NodeId::Set(key), config.metadata);

        key
    }

    /// Adds the config nodes to the graph
    #[track_caller]
    pub(crate) fn process_configs<
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
    pub(super) fn add_system_inner(&mut self, config: ScheduleConfig<ScheduleSystem>) -> SystemKey {
        let key = self.systems.insert(config.node, config.conditions);

        // graph updates are immediate
        self.update_graphs(NodeId::System(key), config.metadata);

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
        let mut warnings = Vec::new();

        // Check hierarchy for cycles
        self.hierarchy.topsort =
            self.topsort_graph(&self.hierarchy.graph, ReportCycles::Hierarchy)?;

        let hier_results = check_graph(&self.hierarchy.graph, &self.hierarchy.topsort);
        if let Some(warning) =
            self.optionally_check_hierarchy_conflicts(&hier_results.transitive_edges)?
        {
            todo!()
        }

        // Remove redundant edges
        self.hierarchy.graph = hier_results.transitive_reduction;

        // Check dependencies for cycles
        self.dependency.topsort =
            self.topsort_graph(&self.dependency.graph, ReportCycles::Dependency)?;

        // Check for systems or system sets depending on sets they belong to
        let dep_results = check_graph(&self.dependency.graph, &self.dependency.topsort);
        self.check_for_cross_dependencies(&dep_results, &hier_results.connected)?;

        // Map all system sets to their systems
        // Go in reverse topological order (bottom-up) for efficiency
        let (set_systems, set_system_bitsets) =
            self.map_sets_to_systems(&self.hierarchy.topsort, &self.hierarchy.graph);
        self.check_order_but_intersect(&dep_results.connected, &set_system_bitsets)?;

        // Check that there are no edges to system-type sets that have multiple instances
        self.check_system_type_set_ambiguity(&set_systems)?;

        let mut dependency_flattened = self.get_dependency_flattened(&set_systems);

        // Modify graph with build passes
        let mut passes = core::mem::take(&mut self.passes);
        for pass in passes.values_mut() {
            todo!()
        }
        self.passes = passes;

        // topsort
        let mut dependency_flattened_dag = Dag {
            topsort: self.topsort_graph(&dependency_flattened, ReportCycles::Dependency)?,
            graph: dependency_flattened,
        };

        let flat_results = check_graph(
            &dependency_flattened_dag.graph,
            &dependency_flattened_dag.topsort,
        );

        // Remove redundant edges
        dependency_flattened_dag.graph = flat_results.transitive_reduction;

        // Flatten: combine `in_set` with `ambiguous_with` information
        let ambiguous_with_flattened = self.get_ambiguous_with_flattened(&set_systems);
        self.set_systems = set_systems;

        // Check for conflicts
        let conflicting_systems = self.get_conflicting_systems(
            &flat_results.disconnected,
            &ambiguous_with_flattened,
            ignored_ambiguities,
        );
        if let Some(warning) = self.optionally_check_conflicts(&conflicting_systems)? {
            todo!()
        }
        self.conflicting_systems = conflicting_systems;

        Ok((
            self.build_schedule_inner(dependency_flattened_dag, hier_results.reachable),
            warnings,
        ))
    }

    fn build_schedule_inner(
        &self,
        dependency_flattened_dag: Dag<SystemKey>,
        hier_results_reachable: FixedBitSet,
    ) -> SystemSchedule {
        let dg_system_ids = dependency_flattened_dag.topsort;
        let dg_system_idx_map = dg_system_ids
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect::<HashMap<_, _>>();

        let hg_systems = self
            .hierarchy
            .topsort
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, id)| Some((i, id.as_system()?)))
            .collect::<Vec<_>>();

        let (hg_set_with_conditions_idxs, hg_set_ids): (Vec<_>, Vec<_>) = self
            .hierarchy
            .topsort
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, id)| {
                // Ignore system sets that have no conditions
                let key = id.as_set()?;
                self.system_sets.has_conditions(key).then_some((i, key))
            })
            .unzip();

        let sys_count = self.systems.len();
        let set_with_conditions_count = hg_set_ids.len();
        let hg_node_count = self.hierarchy.graph.node_count();

        // Get the number of dependencies and the immediate dependents of each system
        // (needed by multi_threaded executor to run systems in the correct order)
        let mut system_dependencies = Vec::with_capacity(sys_count);
        let mut system_dependents = Vec::with_capacity(sys_count);
        for &sys_key in &dg_system_ids {
            let num_dependencies = dependency_flattened_dag
                .graph
                .neighbors_directed(sys_key, Direction::Incoming)
                .count();
            let dependents = dependency_flattened_dag
                .graph
                .neighbors_directed(sys_key, Direction::Outgoing)
                .map(|dep_id| dg_system_idx_map[&dep_id])
                .collect::<Vec<_>>();

            system_dependencies.push(num_dependencies);
            system_dependents.push(dependents);
        }

        // Get the rows and columns of the hierarchy graph's reachability matrix
        // (needed to we can evaluate conditions in the correct order)
        let mut systems_in_sets_with_conditions =
            vec![FixedBitSet::with_capacity(sys_count); set_with_conditions_count];
        for (i, &row) in hg_set_with_conditions_idxs.iter().enumerate() {
            todo!()
        }

        let mut sets_with_conditions_of_systems =
            vec![FixedBitSet::with_capacity(set_with_conditions_count); sys_count];
        for &(col, sys_key) in &hg_systems {
            let i = dg_system_idx_map[&sys_key];
            let bitset = &mut sets_with_conditions_of_systems[i];
            for (idx, &row) in hg_set_with_conditions_idxs
                .iter()
                .enumerate()
                .take_while(|&(_idx, &row)| row < col)
            {
                todo!()
            }
        }

        SystemSchedule {
            systems: Vec::with_capacity(sys_count),
            system_conditions: Vec::with_capacity(sys_count),
            set_conditions: Vec::with_capacity(set_with_conditions_count),
            system_ids: dg_system_ids,
            set_ids: hg_set_ids,
            // system_dependencies,
            // system_dependents,
            sets_with_conditions_of_systems,
            // systems_in_sets_with_conditions,
        }
    }

    /// Return a map from system set `NodeId` to a list of system `NodeId`s that are included in the set
    /// Also return a map from system set `NodeId` to a `FixedBitSet` of system `NodeId`s that are included,
    /// where the bitset order is the same as `self.systems`
    fn map_sets_to_systems(
        &self,
        hierarchy_topsort: &[NodeId],
        hierarchy_graph: &DiGraph<NodeId>,
    ) -> (
        HashMap<SystemSetKey, Vec<SystemKey>>,
        HashMap<SystemSetKey, HashSet<SystemKey>>,
    ) {
        let mut set_systems: HashMap<SystemSetKey, Vec<SystemKey>> =
            HashMap::with_capacity_and_hasher(self.system_sets.len(), Default::default());
        let mut set_system_sets: HashMap<SystemSetKey, HashSet<SystemKey>> =
            HashMap::with_capacity_and_hasher(self.system_sets.len(), Default::default());
        for &id in hierarchy_topsort.iter().rev() {
            let NodeId::Set(set_key) = id else {
                continue;
            };

            let mut systems = Vec::new();
            let mut system_set = HashSet::with_capacity(self.systems.len());

            for child in hierarchy_graph.neighbors_directed(id, Direction::Outgoing) {
                match child {
                    NodeId::System(key) => {
                        systems.push(key);
                        system_set.insert(key);
                    }
                    NodeId::Set(key) => {
                        let child_systems = set_systems.get(&key).unwrap();
                        let child_system_set = set_system_sets.get(&key).unwrap();
                        systems.extend_from_slice(child_systems);
                        system_set.extend(child_system_set.iter());
                    }
                }
            }

            set_systems.insert(set_key, systems);
            set_system_sets.insert(set_key, system_set);
        }

        (set_systems, set_system_sets)
    }

    fn get_dependency_flattened(
        &mut self,
        set_systems: &HashMap<SystemSetKey, Vec<SystemKey>>,
    ) -> DiGraph<SystemKey> {
        // Flatten: combine `in_set` with `before` and `after` information
        let mut dependency_flattening = self.dependency.graph.clone();
        let mut temp = Vec::new();
        for (&set, systems) in set_systems {
            for pass in self.passes.values_mut() {
                todo!()
            }
            if systems.is_empty() {
                todo!()
            } else {
                for a in
                    dependency_flattening.neighbors_directed(NodeId::Set(set), Direction::Incoming)
                {
                    todo!()
                }
                for b in
                    dependency_flattening.neighbors_directed(NodeId::Set(set), Direction::Outgoing)
                {
                    todo!()
                }
            }

            dependency_flattening.remove_node(NodeId::Set(set));
            for (a, b) in temp.drain(..) {
                dependency_flattening.add_edge(a, b);
            }
        }

        // By this point, we should have removed all system sets from the graph
        dependency_flattening
            .try_into::<SystemKey>()
            .unwrap_or_else(|n| {
                unreachable!(
                    "Flattened dependency graph has a leftover system set {}",
                    self.get_node_name(&NodeId::Set(n))
                )
            })
    }

    fn get_ambiguous_with_flattened(
        &self,
        set_systems: &HashMap<SystemSetKey, Vec<SystemKey>>,
    ) -> UnGraph<NodeId> {
        let mut ambiguous_with_flattened = UnGraph::default();
        for (lhs, rhs) in self.ambiguous_with.all_edges() {
            todo!()
        }

        ambiguous_with_flattened
    }

    fn get_conflicting_systems(
        &self,
        flat_results_disconnected: &Vec<(SystemKey, SystemKey)>,
        ambiguous_with_flattened: &UnGraph<NodeId>,
        ignored_ambiguities: &BTreeSet<ComponentId>,
    ) -> Vec<(SystemKey, SystemKey, Vec<ComponentId>)> {
        let mut conflicting_systems = Vec::new();
        for &(a, b) in flat_results_disconnected {
            todo!()
        }

        conflicting_systems
    }

    /// Updates the `SystemSchedule` from the `ScheduleGraph`
    pub(crate) fn update_schedule(
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

        // Move systems into new schedule
        for &key in &schedule.system_ids {
            let system = self.systems.node_mut(key).unwrap().inner.take().unwrap();
            let conditions = core::mem::take(self.systems.get_conditions_mut(key).unwrap());
            schedule.systems.push(system);
            schedule.system_conditions.push(conditions);
        }

        for &key in &schedule.set_ids {
            todo!()
        }

        Ok(warnings)
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

        if sccs_with_cycles.is_empty() {
            // Reverse to get topological order
            top_sorted_nodes.reverse();
            Ok(top_sorted_nodes)
        } else {
            todo!()
        }
    }

    /// If [`ScheduleBuildSettings::hierarchy_detection`] is [`LogLevel::Ignore`] this check is skipped
    fn optionally_check_hierarchy_conflicts(
        &self,
        transitive_edges: &[(NodeId, NodeId)],
    ) -> Result<Option<ScheduleBuildWarning>, ScheduleBuildError> {
        match (
            self.settings.hierarchy_detection,
            !transitive_edges.is_empty(),
        ) {
            (LogLevel::Warn, true) => Ok(Some(ScheduleBuildWarning::HierarchyRedundancy(
                transitive_edges.to_vec(),
            ))),
            (LogLevel::Error, true) => {
                Err(ScheduleBuildWarning::HierarchyRedundancy(transitive_edges.to_vec()).into())
            }
            _ => Ok(None),
        }
    }

    fn check_for_cross_dependencies(
        &self,
        dep_results: &CheckGraphResults<NodeId>,
        hier_results_connected: &HashSet<(NodeId, NodeId)>,
    ) -> Result<(), ScheduleBuildError> {
        for &(a, b) in &dep_results.connected {
            if hier_results_connected.contains(&(a, b)) || hier_results_connected.contains(&(b, a))
            {
                return Err(ScheduleBuildError::CrossDependency(a, b));
            }
        }

        Ok(())
    }

    fn check_order_but_intersect(
        &self,
        dep_results_connected: &HashSet<(NodeId, NodeId)>,
        set_system_sets: &HashMap<SystemSetKey, HashSet<SystemKey>>,
    ) -> Result<(), ScheduleBuildError> {
        // Check that there is no ordering between system sets that intersect
        for &(a, b) in dep_results_connected {
            let (NodeId::Set(a_key), NodeId::Set(b_key)) = (a, b) else {
                continue;
            };

            todo!()
        }

        Ok(())
    }

    fn check_system_type_set_ambiguity(
        &self,
        set_systems: &HashMap<SystemSetKey, Vec<SystemKey>>,
    ) -> Result<(), ScheduleBuildError> {
        for (&key, systems) in set_systems {
            let set = &self.system_sets[key];
            if set.system_type().is_some() {
                todo!()
            }
        }

        Ok(())
    }

    /// If [`ScheduleBuildSettings::ambiguity_detection`] is [`LogLevel::Ignore`], this check is skipped
    fn optionally_check_conflicts(
        &self,
        conflicts: &[(SystemKey, SystemKey, Vec<ComponentId>)],
    ) -> Result<Option<ScheduleBuildWarning>, ScheduleBuildError> {
        match (self.settings.ambiguity_detection, !conflicts.is_empty()) {
            (LogLevel::Warn, true) => todo!(),
            (LogLevel::Error, true) => todo!(),
            _ => Ok(None),
        }
    }
}

/// Specifies how schedule construction should respond to detecting a certain kind of issue
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    /// Occurrences are completely ignored
    Ignore,
    /// Occurrences are logged only,
    Warn,
    /// Occurrences are logged and result in errors
    Error,
}

/// Specifies miscellaneous settings for schedule construction
#[derive(Clone, Debug)]
pub struct ScheduleBuildSettings {
    /// Determines whether the presence of ambiguities (systems with conflicting access but indeterminate order)
    /// is only logged or also results in a warning or error
    pub ambiguity_detection: LogLevel,
    /// Determines whether the presence of redundant edges in the hierarchy of system sets is only
    /// logged or also results in a [`HierarchyRedundancy`] warning or error
    pub hierarchy_detection: LogLevel,
    /// If set to true, report all system sets the conflicting systems are part of
    pub report_sets: bool,
}

impl Default for ScheduleBuildSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleBuildSettings {
    /// Default build settings
    pub const fn new() -> Self {
        Self {
            ambiguity_detection: LogLevel::Ignore,
            hierarchy_detection: LogLevel::Warn,
            report_sets: true,
        }
    }
}
