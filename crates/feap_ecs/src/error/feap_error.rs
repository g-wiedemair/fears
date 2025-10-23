use alloc::boxed::Box;
use core::fmt::Debug;
use core::{error::Error, fmt::Display};

/// The builtin "universal" Feap error type.
/// This has a blanket [`From`] impl for any type that implements Rust's [`Error`],
/// meaning it can be used as a "catch all" error.
///
/// When used with the `backtrace` Cargo feature, it will capture a backtrace when the error is constructed (generally in the [`From`] impl]).
/// When printed, the backtrace will be displayed. By default, the backtrace will be trimmed down to filter out noise. To see the full backtrace,
/// set the `FEAP_BACKTRACE=full` environment variable.
pub struct FeapError {
    inner: Box<InnerFeapError>,
}

impl FeapError {
    fn format_backtrace(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "backtrace")]
        {
            let f = _f;
            let backtrace = &self.inner.backtrace;
            if let std::backtrace::BacktraceStatus::Captured = backtrace.status() {
                let full_backtrace = std::env::var("FEAP_BACKTRACE").is_ok_and(|val| val == "full");
                
                todo!()
            }
            todo!()
        }
        Ok(())
    }
}

/// This type exists (rather than having a `BevyError(Box<dyn InnerBevyError)`) to make [`BevyError`] use a "thin pointer" instead of
/// a "fat pointer", which reduces the size of our Result by a usize. This does introduce an extra indirection, but error handling is a "cold path".
/// We don't need to optimize it to that degree.
struct InnerFeapError {
    error: Box<dyn Error + Send + Sync + 'static>,
    #[cfg(feature = "backtrace")]
    backtrace: std::backtrace::Backtrace,
}

impl<E> From<E> for FeapError
where
    Box<dyn Error + Send + Sync + 'static>: From<E>,
{
    #[cold]
    fn from(error: E) -> Self {
        FeapError {
            inner: Box::new(InnerFeapError {
                error: error.into(),
                #[cfg(feature = "backtrace")]
                backtrace: std::backtrace::Backtrace::capture(),
            }),
        }
    }
}

impl Display for FeapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.inner.error)?;
        self.format_backtrace(f)?;
        Ok(())
    }
}

impl Debug for FeapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{:?}", self.inner.error)?;
        self.format_backtrace(f)?;
        Ok(())
    }
}
