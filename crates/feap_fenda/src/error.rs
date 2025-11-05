/// Alias for [`core::result::Result`]
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum Error {
    /// IO-Error
    IoError(std::io::Error),
}

impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, _f: &mut core::fmt::Formatter) -> core::fmt::Result {
        todo!()
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
