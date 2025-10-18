use core::{fmt::Debug, any::Any};
use feap_utils::map::TypeIdMap;
use super::node::NodeId;
use alloc::boxed::Box;

/// Object safe version of [`ScheduleBuildPass`]
pub(super) trait ScheduleBuildPassObj: Send + Sync + Debug {
    fn add_dependency(&mut self, from: NodeId, to: NodeId, all_options: &TypeIdMap<Box<dyn Any>>);
}
