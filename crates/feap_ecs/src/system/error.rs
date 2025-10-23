use crate::error::FeapError;
use core::any::Any;

/// Running system failed
#[derive(Debug)]
pub enum RunSystemError {
    /// System returned an error or failed required parameter validation
    Failed(FeapError),
}

impl<E: Any> From<E> for RunSystemError
where
    FeapError: From<E>,
{
    fn from(_: E) -> RunSystemError {
        todo!()
    }
}
