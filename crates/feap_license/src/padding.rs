use crate::{
    error::Result,
    key::{RsaPrivateKey, RsaPublicKey},
};
use crypto_bigint::BoxedUint;
use rand::TryCryptoRng;
use zeroize::Zeroizing;

/// Padding scheme used for encryption
pub trait PaddingScheme {
    /// Encrypt the given message using the given public key
    fn encrypt<Rng: TryCryptoRng + ?Sized>(
        self,
        rng: &mut Rng,
        pub_key: &RsaPublicKey,
        msg: &[u8],
    ) -> Result<Vec<u8>>;

    fn decrypt<Rng: TryCryptoRng + ?Sized>(
        self,
        rng: Option<&mut Rng>,
        priv_key: &RsaPrivateKey,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>>;
}

/// Converts input to the new vector of the given length, using BE and with 0s left padded
#[inline]
pub(crate) fn uint_to_zeroizing_no_pad(input: BoxedUint, padded_len: usize) -> Result<Vec<u8>> {
    let leading_zeros = input.leading_zeros() as usize / 8;

    let m = Zeroizing::new(input);
    let m = Zeroizing::new(m.to_be_bytes());

    no_pad(&m[..], padded_len)
}

/// Converts input to the new vector of the given length, using BE and with 0s left padded
#[inline]
pub(crate) fn uint_to_no_pad(input: BoxedUint, padded_len: usize) -> Result<Vec<u8>> {
    let leading_zeros = input.leading_zeros() as usize / 8;
    no_pad(&input.to_be_bytes()[leading_zeros..], padded_len)
}

/// Returns a new vector of the given length, without padding
#[inline]
fn no_pad(input: &[u8], _padded_len: usize) -> Result<Vec<u8>> {
    let mut out = vec![0u8; input.len()];
    out[..input.len()].copy_from_slice(input);
    Ok(out)
}
