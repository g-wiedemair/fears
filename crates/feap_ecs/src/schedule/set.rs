use crate::define_label;
pub use feap_ecs_macros::SystemSet;

define_label!(
    /// System sets are tag-like labels that can be used to group systems together
    ///
    /// This allows you to share configuration (like run conditions) across multiple systems,
    /// and order systems or system sets relative to conceptual groups of systems.
    ///
    #[diagnostic::on_unimplemented(
        note = "consider annotating `{Self}` with `#[derive(SystemSet)]`"
    )]
    SystemSet,
    SYSTEM_SET_INTERNER,
    extra_methods: {},
    extra_methods_impl: {}
);
