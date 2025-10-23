use crate::{
    message::{Message, MessageInstance},
    resource::Resource,
};
use alloc::vec::Vec;

/// A message collection that represents the messages that occurred within the last two
/// [`Messages::update`] calls
#[derive(Debug, Resource)]
pub struct Messages<E: Message> {
    /// Holds the oldest still active messages
    pub(crate) messages_a: MessageSequence<E>,
    /// Holds the newer messages
    pub(crate) messages_b: MessageSequence<E>,
    pub(crate) message_count: usize,
}

#[derive(Debug)]
pub(crate) struct MessageSequence<E: Message> {
    pub(crate) messages: Vec<MessageInstance<E>>,
    pub(crate) start_message_count: usize,
}
