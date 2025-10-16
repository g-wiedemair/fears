use crate::world::FromWorld;

/// A system local [`SystemParam`]
///
/// A local may only be accessed by the system itself and is therefore not visible to other systems
/// If two or more systems specify the same local type each will have their own unique local.
///
#[derive(Debug)]
pub struct Local<'s, T: FromWorld + Send + 'static>(pub(crate) &'s mut T);
