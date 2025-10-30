/// Alias for [`core::result::Result`]
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// Invalid prime value.
    InvalidPrime,

    /// Number of primes must be 2 or greater.
    NprimesTooSmall,

    /// Too few primes of a given length to generate an RSA key.
    TooFewPrimes,

    /// Invalid modulus.
    InvalidModulus,

    /// Modulus too small.
    ModulusTooSmall,

    /// Modulus too large.
    ModulusTooLarge,

    /// Invalid exponent.
    InvalidExponent,

    /// Public exponent too small.
    PublicExponentTooSmall,

    /// Public exponent too large.
    PublicExponentTooLarge,

    /// Invalid padding length.
    InvalidPadLen,

    /// Message too long.
    MessageTooLong,

    /// Decoding error
    Decode(crypto_bigint::DecodeError),

    /// Decryption error.
    Decryption,

    /// Verification error.
    Verification,

    /// Internal error.
    Internal,
}

impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, _f: &mut core::fmt::Formatter) -> core::fmt::Result {
        todo!()
    }
}

impl From<crypto_bigint::DecodeError> for Error {
    fn from(e: crypto_bigint::DecodeError) -> Self {
        Self::Decode(e)
    }
}
