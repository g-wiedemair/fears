mod messages;

pub use feap_ecs_macros::Message;
pub use messages::Messages;

use crate::change_detection::MaybeLocation;
use core::{fmt, marker::PhantomData};

/// A buffered message for pull-based event handling
///
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an `Message`",
    label = "invalid `Message`",
    note = "consider annotating `{Self}` with `#[derive(Message)]`"
)]
pub trait Message: Send + Sync + 'static {}

#[derive(Debug)]
pub(crate) struct MessageInstance<M: Message> {
    pub message_id: MessageId<M>,
    pub message: M,
}

pub struct MessageId<M: Message> {
    pub id: usize,
    pub caller: MaybeLocation,
    pub(super) _marker: PhantomData<M>,
}

impl<M: Message> Copy for MessageId<M> {}

impl<M: Message> Clone for MessageId<M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M: Message> fmt::Display for MessageId<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl<M: Message> fmt::Debug for MessageId<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "message<{}>#{}",
            core::any::type_name::<M>().split("::").last().unwrap(),
            self.id,
        )
    }
}
