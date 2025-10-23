use super::feap_error::FeapError;
use crate::{component::Tick, resource::Resource};
use core::fmt::Display;
// use  derive_more::derive::{Deref, DerefMut};
use feap_utils::debug_info::DebugName;

/// Context for a [`FeapError`] to aid in debugging
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorContext {
    /// The error occurred in a system
    System {
        /// The name of the system that failed
        name: DebugName,
        /// The last tick that the system was run
        last_run: Tick,
    },
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::System { name, .. } => {
                write!(f, "System `{name}` failed")
            }
        }
    }
}

impl ErrorContext {
    /// The name of the ECS construct that failed
    pub fn name(&self) -> DebugName {
        match self {
            Self::System { name, .. } => name.clone(),
        }
    }

    /// A string representation of the kind of ECS construct that failed
    pub fn kind(&self) -> &str {
        match self {
            Self::System { .. } => "system",
        }
    }
}

/// Defines how Feap reacts to errors
pub type ErrorHandler = fn(FeapError, ErrorContext);

/// Error handler to call when an error is not handled otherwise
/// Defaults to [`panic()`]
///

#[derive(Resource, Copy, Clone)]
pub struct DefaultErrorHandler(pub ErrorHandler);

impl Default for DefaultErrorHandler {
    fn default() -> Self {
        Self(panic)
    }
}

macro_rules! inner {
    ($call:path, $e:ident, $c:ident) => {
        $call!(
            "Encountered an error in {} `{}`: {}",
            $c.kind(),
            $c.name(),
            $e
        );
    };
}

/// Error handler that panics with the system error.
#[track_caller]
#[inline]
pub fn panic(error: FeapError, ctx: ErrorContext) {
    inner!(panic, error, ctx);
}
