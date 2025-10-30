use crate::{
    error::Result,
    key::{RsaPrivateKey, RsaPublicKey},
};
use crypto_bigint::rand_core::TryCryptoRng;

/// Digital signature scheme
pub trait SignatureScheme {
    /// Sign the given digest
    fn sign<Rng: TryCryptoRng + ?Sized>(
        self,
        rng: Option<&mut Rng>,
        priv_key: &RsaPrivateKey,
        hashed: &[u8],
    ) -> Result<Vec<u8>>;

    /// Verify a signed message
    ///
    /// `hashed` must be the result of hashing the input using the hashing function passed in through `hash`
    fn verify(self, pub_key: &RsaPublicKey, hashed: &[u8], sig: &[u8]) -> Result<()>;
}
