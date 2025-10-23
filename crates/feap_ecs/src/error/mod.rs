mod feap_error;
mod handler;

pub use {handler::{DefaultErrorHandler, ErrorHandler, ErrorContext}, feap_error::FeapError};
