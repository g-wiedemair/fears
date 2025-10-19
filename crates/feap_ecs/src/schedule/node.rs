use super::{
    BoxedCondition, InternedSystemSet,
    graph::{Direction, GraphNodeId},
};
use crate::{
    query::FilteredAccessSet,
    system::{ReadOnlySystem, ScheduleSystem},
    world::World,
};
use alloc::{boxed::Box, vec::Vec};
use core::fmt::Debug;
use feap_core::collections::HashMap;
use slotmap::{Key, KeyData, SecondaryMap, SlotMap, new_key_type};

new_key_type! {
    /// A unique identifier for a system in a [`ScheduleGraph`]
    pub struct SystemKey;
    /// A unique identifier for a system set in a [`ScheduleGraph`]
    pub struct SystemSetKey;
}

/// Unique indentifier for a system or system set stored in a [`ScheduleGraph`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeId {
    /// Identifier for a system
    System(SystemKey),
    /// Identifier for a system set
    Set(SystemSetKey),
}

impl NodeId {
    /// Returns `true` if the identified node is a system
    pub const fn is_system(&self) -> bool {
        matches!(self, NodeId::System(_))
    }
}

impl GraphNodeId for NodeId {
    type Adjacent = CompactNodeIdAndDirection;
    type Edge = CompactNodeIdPair;
}

/// Compact storage of a [`NodeId`] and a [`Direction`]
#[derive(Clone, Copy)]
pub struct CompactNodeIdAndDirection {
    key: KeyData,
    is_system: bool,
    direction: Direction,
}

impl Debug for CompactNodeIdAndDirection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl From<(NodeId, Direction)> for CompactNodeIdAndDirection {
    fn from((id, direction): (NodeId, Direction)) -> Self {
        let key = match id {
            NodeId::System(key) => key.data(),
            NodeId::Set(key) => key.data(),
        };
        let is_system = id.is_system();

        Self {
            key,
            is_system,
            direction,
        }
    }
}

impl From<CompactNodeIdAndDirection> for (NodeId, Direction) {
    fn from(value: CompactNodeIdAndDirection) -> Self {
        let node = match value.is_system {
            true => NodeId::System(value.key.into()),
            false => NodeId::Set(value.key.into()),
        };

        (node, value.direction)
    }
}

/// Compact storage of a [`NodeId`] pair
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct CompactNodeIdPair {
    key_a: KeyData,
    key_b: KeyData,
    is_system_a: bool,
    is_system_b: bool,
}

impl Debug for CompactNodeIdPair {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl From<(NodeId, NodeId)> for CompactNodeIdPair {
    fn from((a, b): (NodeId, NodeId)) -> Self {
        let key_a = match a {
            NodeId::System(index) => index.data(),
            NodeId::Set(index) => index.data(),
        };
        let is_system_a = a.is_system();

        let key_b = match b {
            NodeId::System(index) => index.data(),
            NodeId::Set(index) => index.data(),
        };
        let is_system_b = b.is_system();

        Self {
            key_a,
            key_b,
            is_system_a,
            is_system_b,
        }
    }
}

impl From<CompactNodeIdPair> for (NodeId, NodeId) {
    fn from(value: CompactNodeIdPair) -> Self {
        let a = match value.is_system_a {
            true => NodeId::System(value.key_a.into()),
            false => NodeId::Set(value.key_a.into()),
        };
        let b = match value.is_system_b {
            true => NodeId::System(value.key_b.into()),
            false => NodeId::Set(value.key_b.into()),
        };

        (a, b)
    }
}

/// A [`SystemWithAccess`] stored in a [`ScheduleGraph`]
pub(crate) struct SystemNode {
    pub(crate) inner: Option<SystemWithAccess>,
}

impl SystemNode {
    /// Creates a new [`SystemNode`]
    pub fn new(system: ScheduleSystem) -> Self {
        Self {
            inner: Some(SystemWithAccess::new(system)),
        }
    }

    /// Obtain a mutable reference to the [`SystemWithAccess`] represented by this node
    pub fn get_mut(&mut self) -> Option<&mut SystemWithAccess> {
        self.inner.as_mut()
    }
}

/// A [`ScheduleSystem`] stored alongside the access returned from [`System::initialize`]
pub struct SystemWithAccess {
    /// The system itself
    pub system: ScheduleSystem,
    /// The access returned by [`System::initialize`]
    pub access: FilteredAccessSet,
}

impl SystemWithAccess {
    /// Constructs a new [`SystemWithAccess`] from a [`ScheduleSystem`]
    /// The `access` will initially be empty
    pub fn new(system: ScheduleSystem) -> Self {
        Self {
            system,
            access: FilteredAccessSet::new(),
        }
    }
}

/// A [`BoxedCondition`] stored alongside the access returned from [`System::initialize`]
pub struct ConditionWithAccess {
    /// The condition itself
    pub condition: BoxedCondition,
    /// The access returned by [`System::initialize`]
    /// This will be empty if the system has not been initialized yet
    pub access: FilteredAccessSet,
}

impl ConditionWithAccess {
    /// Constructs a new [`ConditionWithAccess`] from a [`BoxedCondition`]
    /// The `access` will initially be empty
    pub const fn new(condition: BoxedCondition) -> Self {
        Self {
            condition,
            access: FilteredAccessSet::new(),
        }
    }
}

/// Container for systems in a schedule
#[derive(Default)]
pub struct Systems {
    /// List of systems in the schedule
    nodes: SlotMap<SystemKey, SystemNode>,
    /// List of conditions for each system, in the same order as `nodes`
    conditions: SecondaryMap<SystemKey, Vec<ConditionWithAccess>>,
    /// Systems and their conditions that have not been initialized yet
    uninit: Vec<SystemKey>,
}

impl Systems {
    /// Inserts a new system into the container, along with its conditions,
    /// and queues it to be initialized later in [`System::initialize`]
    ///
    /// We have to defer initialization of systems in the container until we have
    /// `&mut World` access, so we store these in a list.
    pub fn insert(
        &mut self,
        system: ScheduleSystem,
        conditions: Vec<Box<dyn ReadOnlySystem<In = (), Out = bool>>>,
    ) -> SystemKey {
        let key = self.nodes.insert(SystemNode::new(system));
        self.conditions.insert(
            key,
            conditions
                .into_iter()
                .map(ConditionWithAccess::new)
                .collect(),
        );
        self.uninit.push(key);
        key
    }

    /// Initializes all systems and their conditions that have not been initialized yet
    pub fn initialize(&mut self, world: &mut World) {
        for key in self.uninit.drain(..) {
            let Some(system) = self.nodes.get_mut(key).and_then(|node| node.get_mut()) else {
                continue;
            };
            system.access = system.system.initialize(world);
            let Some(conditions) = self.conditions.get_mut(key) else {
                continue;
            };
            for condition in conditions {
                condition.access = condition.condition.initialize(world);
            }
        }
    }

    /// Returns `true` if all systems in this container have been initialized
    pub fn is_initialized(&self) -> bool {
        self.uninit.is_empty()
    }
}

/// Container for system sets in a schedule
#[derive(Default)]
pub struct SystemSets {
    /// List of system sets in the schedule
    sets: SlotMap<SystemSetKey, InternedSystemSet>,
    /// List of conditions for each system set, in the same order as `sets`
    conditions: SecondaryMap<SystemSetKey, Vec<ConditionWithAccess>>,
    /// Map from system sets to their keys
    ids: HashMap<InternedSystemSet, SystemSetKey>,
    /// System sets that have not been initialized yet
    uninit: Vec<UninitializedSet>,
}

/// A system set's conditions that have not been initialized yet
struct UninitializedSet {}

impl SystemSets {
    /// Inserts conditions for a system set into the container, and queues the
    /// newly added conditions to be initialized later in [`SystemSets::initialize`]
    ///
    pub fn insert(
        &mut self,
        set: InternedSystemSet,
        new_conditions: Vec<Box<dyn ReadOnlySystem<In = (), Out = bool>>>,
    ) -> SystemSetKey {
        let key = self.get_key_or_insert(set);
        if !new_conditions.is_empty() {
            todo!()
        }
        key
    }

    /// Returns the key for the given system set, inserting it into this
    /// container if it does not already exist
    pub fn get_key_or_insert(&mut self, set: InternedSystemSet) -> SystemSetKey {
        *self.ids.entry(set).or_insert_with(|| {
            let key = self.sets.insert(set);
            self.conditions.insert(key, Vec::new());
            key
        })
    }

    /// Initializes all system sets conditions that have not been initialized yet.
    /// Because a system set's conditions may be appended to multiple times, we
    /// track which conditions were added since the last initialization and only initialize these
    pub fn initialize(&mut self, world: &mut World) {
        for uninit in self.uninit.drain(..) {
            todo!()
        }
    }

    /// Returns `true` if all system sets conditions in this container have been initialized
    pub fn is_initialized(&self) -> bool {
        self.uninit.is_empty()
    }
}
