use crate::{
    schedule::{BoxedCondition, Chain, GraphInfo, InternedSystemSet, SystemSet},
    system::{BoxedSystem, IntoSystem, ScheduleSystem},
};
use alloc::{boxed::Box, vec, vec::Vec};
use variadics_please::all_tuples;

/// Stores data to differentiate different schedulable structs
pub trait Schedulable {
    /// Additional data used to configure independent scheduling
    type Metadata;
    /// Additional data used to configure a schedulable group
    type GroupMetadata;

    /// Initializes a configuration from this node
    fn into_config(self) -> ScheduleConfig<Self>
    where
        Self: Sized;
}

impl Schedulable for ScheduleSystem {
    type Metadata = GraphInfo;
    type GroupMetadata = Chain;

    fn into_config(self) -> ScheduleConfig<Self> {
        let sets = self.default_system_sets().clone();
        ScheduleConfig {
            node: self,
            metadata: GraphInfo {
                hierarchy: sets,
                ..Default::default()
            },
            conditions: Vec::new(),
        }
    }
}

impl Schedulable for InternedSystemSet {
    type Metadata = GraphInfo;
    type GroupMetadata = Chain;

    fn into_config(self) -> ScheduleConfig<Self> {
        assert!(
            self.system_type().is_none(),
            "configuring system type sets is not allowed"
        );

        ScheduleConfig {
            node: self,
            metadata: GraphInfo::default(),
            conditions: Vec::new(),
        }
    }
}

/// Stores configuration for a single generic node (a system or a system set)
/// The configuration includes the node itself, scheduling metadata
/// (hierarchy: in which sets is the node contained,
/// dependencies: before/after which other nodes should this node run)
/// and the run conditions associated with this node
pub struct ScheduleConfig<T: Schedulable> {
    pub(crate) node: T,
    pub(crate) metadata: T::Metadata,
    pub(crate) conditions: Vec<BoxedCondition>,
}

/// Single or nested configurations for [`Schedulable`]s
pub enum ScheduleConfigs<T: Schedulable> {
    /// Configuration for a single [`Schedulable`]
    ScheduleConfig(ScheduleConfig<T>),
    /// Configuration for a tuple of nested `Configs` instances
    Configs {
        /// Configuration for each element of the tuple
        configs: Vec<ScheduleConfigs<T>>,
        /// Run conditions applied to everything in the tuple
        collective_conditions: Vec<BoxedCondition>,
        /// Metadata to be applied to all elements in the tuple
        metadata: T::GroupMetadata,
    },
}

impl<T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>> ScheduleConfigs<T> {
    fn chain_inner(mut self) -> Self {
        match &mut self {
            Self::ScheduleConfig(_) => { /* no op */ }
            Self::Configs { metadata, .. } => {
                metadata.set_chained();
            }
        };
        self
    }
}

/// Types that can convert into a [`ScheduleConfig`]
/// This trait is implemented for "systems"
/// It is a common entry point for system configurations
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not describe a valid system configuration",
    label = "invalid system configuration"
)]
pub trait IntoScheduleConfigs<T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>, Marker>:
    Sized
{
    /// Convert into a [`ScheduleConfigs`]
    fn into_configs(self) -> ScheduleConfigs<T>;

    /// Treat this collection as a sequence of systems
    ///
    /// Ordering constraints will be applied between the successive elements
    /// If the preceeding node on an edge has deferred parameters, an [`ApplyDeferred`]
    /// will be inserted on the edge.
    fn chain(self) -> ScheduleConfigs<T> {
        self.into_configs().chain()
    }
}

impl<T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>> IntoScheduleConfigs<T, ()>
    for ScheduleConfigs<T>
{
    fn into_configs(self) -> Self {
        self
    }

    fn chain(self) -> ScheduleConfigs<T> {
        self.chain_inner()
    }
}

impl<F, Marker> IntoScheduleConfigs<ScheduleSystem, Marker> for F
where
    F: IntoSystem<(), (), Marker>,
{
    fn into_configs(self) -> ScheduleConfigs<ScheduleSystem> {
        let boxed_system = Box::new(IntoSystem::into_system(self));
        ScheduleConfigs::ScheduleConfig(ScheduleSystem::into_config(boxed_system))
    }
}

impl IntoScheduleConfigs<ScheduleSystem, ()> for BoxedSystem<(), ()> {
    fn into_configs(self) -> ScheduleConfigs<ScheduleSystem> {
        todo!()
    }
}

impl<S: SystemSet> IntoScheduleConfigs<InternedSystemSet, ()> for S {
    fn into_configs(self) -> ScheduleConfigs<InternedSystemSet> {
        ScheduleConfigs::ScheduleConfig(InternedSystemSet::into_config(self.intern()))
    }
}

#[doc(hidden)]
pub struct ScheduleConfigTupleMarker;

macro_rules! impl_node_type_collection {
    ($(#[$meta:meta])* $(($param: ident, $sys: ident)),*) => {
        $(#[$meta])*
        impl<$($param, $sys),*, T: Schedulable<Metadata = GraphInfo, GroupMetadata = Chain>> IntoScheduleConfigs<T, (ScheduleConfigTupleMarker, $($param,)*)> for ($($sys,)*)
        where
            $($sys: IntoScheduleConfigs<T, $param>),*
        {
            #[expect(
                clippy::allow_attributes,
                reason = "We are inside a macro, and as such, `non_snake_case` is not guaranteed to apply."
            )]
            #[allow(
                non_snake_case,
                reason = "Variable names are provided by the macro caller, not by us."
            )]
            fn into_configs(self) -> ScheduleConfigs<T> {
                let ($($sys,)*) = self;
                ScheduleConfigs::Configs {
                    metadata: Default::default(),
                    configs: vec![$($sys.into_configs(),)*],
                    collective_conditions: Vec::new(),
                }
            }
        }
    };
}

all_tuples!(impl_node_type_collection, 1, 20, P, S);
