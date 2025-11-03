use crate::{
    error::{Error, Result},
    key::{self, PrivateKeyParts, PublicKeyParts, RsaPrivateKey, RsaPublicKey},
    padding::{PaddingScheme, uint_to_no_pad, uint_to_zeroizing_no_pad},
    signature::SignatureScheme,
};
use core::cmp::Ordering;
use crypto_bigint::{
    BoxedUint, NonZero, Resize,
    modular::{BoxedMontyForm, BoxedMontyParams},
    rand_core::TryCryptoRng,
};

/// Encryption using RSA
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(dead_code)]
pub struct RsaEncrypt;

impl PaddingScheme for RsaEncrypt {
    fn encrypt<Rng: TryCryptoRng + ?Sized>(
        self,
        rng: &mut Rng,
        pub_key: &RsaPublicKey,
        msg: &[u8],
    ) -> Result<Vec<u8>> {
        encrypt(rng, pub_key, msg)
    }

    fn decrypt<Rng: TryCryptoRng + ?Sized>(
        self,
        rng: Option<&mut Rng>,
        priv_key: &RsaPrivateKey,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        decrypt(rng, priv_key, ciphertext)
    }
}

/// Encryptes the given message with RSA
#[inline]
fn encrypt<R: TryCryptoRng + ?Sized>(
    _rng: &mut R,
    pub_key: &RsaPublicKey,
    msg: &[u8],
) -> Result<Vec<u8>> {
    key::check_public(pub_key)?;

    let int = BoxedUint::from_be_slice(&msg, pub_key.n_bits_precision())?;
    uint_to_no_pad(rsa_encrypt(pub_key, &int)?, pub_key.size())
}

/// Decrypts a plaintext using RSA
#[inline]
fn decrypt<R: TryCryptoRng + ?Sized>(
    rng: Option<&mut R>,
    priv_key: &RsaPrivateKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    key::check_public(priv_key)?;

    let ciphertext = BoxedUint::from_be_slice(ciphertext, priv_key.n_bits_precision())?;
    let em = rsa_decrypt_and_check(priv_key, rng, &ciphertext)?;
    let em = uint_to_zeroizing_no_pad(em, priv_key.size())?;

    rsa_unpad(em, priv_key.size())
}

/// ⚠️ Raw RSA encryption of m with the public key. No padding is performed.
///
/// # ☢️️ WARNING: HAZARDOUS API ☢️
///
/// Use this function with great care! Raw RSA should never be used without an appropriate padding
/// or signature scheme. See the [crate::hazmat] for more information.
#[inline]
pub fn rsa_encrypt<K: PublicKeyParts>(key: &K, m: &BoxedUint) -> Result<BoxedUint> {
    let res = pow_mod_params(m, key.e(), key.n_params());
    Ok(res)
}

/// ⚠️ Performs raw RSA decryption with no padding.
///
/// Returns a plaintext `BoxedUint`. Performs RSA blinding if an `Rng` is passed.  This will also
/// check for errors in the CRT computation.
///
/// `c` must have the same `bits_precision` as the RSA key modulus.
///
/// # ☢️️ WARNING: HAZARDOUS API ☢️
///
/// Use this function with great care! Raw RSA should never be used without an appropriate padding
/// or signature scheme. See the [crate::hazmat] for more information.
#[inline]
pub fn rsa_decrypt_and_check<R: TryCryptoRng + ?Sized>(
    priv_key: &impl PrivateKeyParts,
    rng: Option<&mut R>,
    c: &BoxedUint,
) -> Result<BoxedUint> {
    let m = rsa_decrypt(rng, priv_key, c)?;

    // m^e should match the original message
    let check = rsa_encrypt(priv_key, &m)?;
    if c != &check {
        return Err(Error::Internal);
    }

    Ok(m)
}

#[inline]
pub fn rsa_decrypt<R: TryCryptoRng + ?Sized>(
    rng: Option<&mut R>,
    priv_key: &impl PrivateKeyParts,
    c: &BoxedUint,
) -> Result<BoxedUint> {
    let n = priv_key.n();
    let d = priv_key.d();

    if c.bits_precision() != n.as_ref().bits_precision() {
        return Err(Error::Decryption);
    }

    if c >= n.as_ref() {
        return Err(Error::Decryption);
    }

    let ir = None;
    let n_params = priv_key.n_params();
    let bits = d.bits_precision();

    let c = if let Some(_rng) = rng {
        todo!()
    } else {
        c.try_resize(bits).ok_or(Error::Internal)?
    };

    let m = match (
        priv_key.dp(),
        priv_key.dq(),
        priv_key.qinv(),
        priv_key.p_params(),
        priv_key.q_params(),
    ) {
        (Some(dp), Some(dq), Some(qinv), Some(p_params), Some(q_params)) => {
            let p = &priv_key.primes()[0];
            let q = &priv_key.primes()[1];

            // precomputed: dP = (1/e) mod (p-1) = d mod (p-1)
            // precomputed: dQ = (1/e) mod (q-1) = d mod (q-1)

            // m1 = c^dP mod p
            let p_wide = p_params.modulus().resize_unchecked(c.bits_precision());
            let c_mod_dp = (&c % p_wide.as_nz_ref()).resize_unchecked(dp.bits_precision());
            let cp = BoxedMontyForm::new(c_mod_dp, p_params.clone());
            let mut m1 = cp.pow(dp);

            // m2 = c^dQ mod q
            let q_wide = q_params.modulus().resize_unchecked(c.bits_precision());
            let c_mod_dq = (&c % q_wide.as_nz_ref()).resize_unchecked(dq.bits_precision());
            let cq = BoxedMontyForm::new(c_mod_dq, q_params.clone());
            let m2 = cq.pow(dq).retrieve();

            // (m1 - m2) mod p = (m1 mod p) - (m2 mod p) mod p
            let m2_mod_p = match p_params.bits_precision().cmp(&q_params.bits_precision()) {
                Ordering::Less => {
                    let p_wide = NonZero::new(p.clone())
                        .expect("`p` is non-zero")
                        .resize_unchecked(q_params.bits_precision());
                    (&m2 % p_wide).resize_unchecked(p_params.bits_precision())
                }
                Ordering::Greater => (&m2).resize_unchecked(p_params.bits_precision()),
                Ordering::Equal => m2.clone(),
            };
            let m2r = BoxedMontyForm::new(m2_mod_p, p_params.clone());
            m1 -= &m2r;

            // precomputed: qInv = (1/q) mod p

            // h = qInv.(m1 - m2) mod p
            let h = (qinv * m1).retrieve();

            // m = m2 + h.q
            let m2 = m2.try_resize(n.bits_precision()).ok_or(Error::Internal)?;
            let hq = (h * q)
                .try_resize(n.bits_precision())
                .ok_or(Error::Internal)?;
            m2.wrapping_add(&hq)
        }
        _ => {
            // c^d (mod n)
            pow_mod_params(&c, d, n_params)
        }
    };

    match ir {
        Some(ref ir) => {
            // unblind
            let res = unblind(&m, ir, n_params);
            Ok(res)
        }
        None => Ok(m),
    }
}

/// Computes `base.pow_mod(exp, n)` with precomputed `n_params`
pub fn pow_mod_params(base: &BoxedUint, exp: &BoxedUint, n_params: &BoxedMontyParams) -> BoxedUint {
    let base = reduce_vartime(base, n_params);
    base.pow(exp).retrieve()
}

fn reduce_vartime(n: &BoxedUint, p: &BoxedMontyParams) -> BoxedMontyForm {
    let modulus = p.modulus().as_nz_ref().clone();
    let n_reduced = n.rem_vartime(&modulus).resize_unchecked(p.bits_precision());
    BoxedMontyForm::new(n_reduced, p.clone())
}

pub fn rsa_unpad(mut msg: Vec<u8>, _k: usize) -> Result<Vec<u8>> {
    msg.retain(|&c| c != 0);
    Ok(msg)
}

fn unblind(_m: &BoxedUint, _unblinder: &BoxedUint, _n_params: &BoxedMontyParams) -> BoxedUint {
    todo!()
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RsaSign;

impl SignatureScheme for RsaSign {
    fn sign<Rng: TryCryptoRng + ?Sized>(
        self,
        rng: Option<&mut Rng>,
        priv_key: &RsaPrivateKey,
        hashed: &[u8],
    ) -> Result<Vec<u8>> {
        sign(rng, priv_key, hashed)
    }

    fn verify(self, pub_key: &RsaPublicKey, hashed: &[u8], sig: &[u8]) -> Result<()> {
        verify(pub_key, hashed, &BoxedUint::from_be_slice_vartime(sig))
    }
}

/// Calculates the signature
#[inline]
fn sign<R: TryCryptoRng + ?Sized>(
    rng: Option<&mut R>,
    priv_key: &RsaPrivateKey,
    hashed: &[u8],
) -> Result<Vec<u8>> {
    let em = rsa_sign(hashed, priv_key.size())?;

    let em = BoxedUint::from_be_slice(&em, priv_key.n_bits_precision())?;
    uint_to_zeroizing_no_pad(rsa_decrypt_and_check(priv_key, rng, &em)?, priv_key.size())
}

#[inline]
fn rsa_sign(hashed: &[u8], k: usize) -> Result<Vec<u8>> {
    let hash_len = hashed.len();
    if k < hash_len {
        return Err(Error::MessageTooLong);
    }

    let mut em = vec![0u8; hash_len];
    em[..hash_len].copy_from_slice(hashed);

    Ok(em)
}

/// Verifies an RSA signature
#[inline]
fn verify(pub_key: &RsaPublicKey, hashed: &[u8], sig: &BoxedUint) -> Result<()> {
    let n = pub_key.n();
    if sig >= n.as_ref() || sig.bits_precision() != pub_key.n_bits_precision() {
        return Err(Error::Verification);
    }

    let em = uint_to_no_pad(rsa_encrypt(pub_key, sig)?, pub_key.size())?;

    rsa_sign_unpad(hashed, &em, pub_key.size())
}

#[inline]
fn rsa_sign_unpad(hashed: &[u8], em: &[u8], k: usize) -> Result<()> {
    let hash_len = hashed.len();
    if k < hash_len {
        return Err(Error::Verification);
    }

    match em[..hash_len].eq(hashed) {
        true => Ok(()),
        false => Err(Error::Verification),
    }
}
