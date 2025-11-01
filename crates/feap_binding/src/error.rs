use std::{borrow::Cow, fmt::Display};

/// Represents an internal error that occurred, with an explanation
#[derive(Clone, Debug)]
pub struct Error {
    /// Describes the kind of error
    pub(crate) kind: ErrorKind,
    /// More explanation of error
    pub(crate) message: Cow<'static, str>,
}

/// Represents the types of errors that may occur
#[derive(Clone, Debug)]
pub(crate) enum ErrorKind {
    /// Error occurred while performing I/O
    IOError,
    /// Environment variable not found, with the var in question as extra info
    EnvVarNotFound,
    /// Error occurred due to missing external tools.
    ToolNotFound,
    /// Error occurred while using external tools (ie: invocation of compiler).
    ToolExecError,
    /// One of the function arguments failed validation
    InvalidArgument,
    /// Invalid target.
    InvalidTarget,
    /// Unknown target
    UnknownTarget,
    /// `feap_binding` has been disabled by an environment variable
    Disabled,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<Cow<'static, str>>) -> Error {
        Error {
            kind,
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::new(ErrorKind::IOError, format!("{e}"))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}
