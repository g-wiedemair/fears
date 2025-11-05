pub mod database;
pub mod project;
mod error;

//--------------------------------------------------------------------------------------------------
// Bindings to fenda

unsafe extern "C" {
    /// Definition of cpu storage
    pub fn getstorage();

    /// Fenda main control program
    pub fn fmacro(input: &i32);
}
