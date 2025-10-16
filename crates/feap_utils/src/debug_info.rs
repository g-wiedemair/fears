use crate::cfg;
cfg::alloc! {
    use alloc::{fmt};
}

#[cfg(not(feature = "debug"))]
const FEATURE_DISABLED: &str = "Enable the debug feature to see the name";

/// Wrapper to help debugging ECS issues. This is used to display the names of systems, components, ...
/// * If the `debug` feature is enabled, the actual name will be used
/// * If it is disabled, a string mentioning the disabled feature will be used
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugName {
    #[cfg(feature = "debug")]
    name: Cow<'static, str>,
}

cfg::alloc! {
    impl fmt::Display for DebugName {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            #[cfg(feature = "debug")]
            f.write_str(self.name.as_ref())?;
            #[cfg(not(feature = "debug"))]
            f.write_str(FEATURE_DISABLED)?;
            Ok(())
        }
    }
}

impl DebugName {
    /// Creates a new `DebugName` from a type by using its [`core::any::type_name`]
    pub fn type_name<T>() -> Self {
        DebugName {
            #[cfg(feature = "debug")]
            name: Cow::Borrowed(type_name::<T>()),
        }
    }
}
