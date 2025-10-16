pub use feap_ecs_macros::Resource;

/// A type that can be inserted into a [`World`] as a singleton
///
/// You can access resource data in systems using the [`Res`] and [`ResMut`] system parameters
/// Only one resource of each type can be stored in a [`World`] at any given time
///
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a `Resource`",
    label = "invalid `Resource`",
    note = "consider annotating `{Self}` with `#[derive(Resource)]`"
)]
pub trait Resource: Send + Sync + 'static {}
