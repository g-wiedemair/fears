use crate::{
    algorithms::{generate_multi_prime_key_with_exp, recover_primes},
    dummy::DummyRng,
    error::{Error, Result},
    padding::PaddingScheme,
    signature::SignatureScheme,
};
use core::{cmp::Ordering, fmt, hash};
use crypto_bigint::{
    modular::BoxedMontyForm, modular::BoxedMontyParams, BoxedUint, Integer, NonZero, Odd, Resize,
};
use rand::CryptoRng;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Represents the public part of an RSA key
#[derive(Debug, Clone)]
pub struct RsaPublicKey {
    /// Modulus: product of prime numbers `p` and `q`
    n: NonZero<BoxedUint>,
    /// Public exponent, typically `0x10001`
    e: BoxedUint,

    n_params: BoxedMontyParams,
}

impl Eq for RsaPublicKey {}

impl PartialEq for RsaPublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.n == other.n && self.e == other.e
    }
}

impl hash::Hash for RsaPublicKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        // Domain separator for RSA private keys
        state.write(b"RsaPublicKey");
        hash::Hash::hash(&self.n, state);
        hash::Hash::hash(&self.e, state);
    }
}

/// Represents the whole RSA key, public and private parts
#[derive(Clone)]
pub struct RsaPrivateKey {
    /// Public components of the private key
    pubkey: RsaPublicKey,
    /// Private exponent
    pub(crate) d: BoxedUint,
    /// Prime factors of N, contains >= 2 elements
    pub(crate) primes: Vec<BoxedUint>,
    /// Precomputed values to speed up private operations
    pub(crate) precomputed: Option<PrecomputedValues>,
}

impl fmt::Debug for RsaPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precomputed = if self.precomputed.is_some() {
            "Some(...)"
        } else {
            "None"
        };
        f.debug_struct("RsaPrivateKey")
            .field("pubkey", &self.pubkey)
            // .field("d", &"...")
            .field("d", &self.d)
            // .field("primes", &"&[...]")
            .field("primes", &self.primes)
            .field("precomputed", &precomputed)
            .finish()
    }
}

impl Eq for RsaPrivateKey {}
impl PartialEq for RsaPrivateKey {
    #[inline]
    fn eq(&self, other: &RsaPrivateKey) -> bool {
        self.pubkey == other.pubkey && self.d == other.d && self.primes == other.primes
    }
}

impl AsRef<RsaPublicKey> for RsaPrivateKey {
    fn as_ref(&self) -> &RsaPublicKey {
        &self.pubkey
    }
}

impl hash::Hash for RsaPrivateKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        // Domain separator for RSA private keys
        state.write(b"RsaPrivateKey");
        hash::Hash::hash(&self.pubkey, state);
    }
}

impl Drop for RsaPrivateKey {
    fn drop(&mut self) {
        self.d.zeroize();
        self.primes.zeroize();
        self.precomputed.zeroize();
    }
}

impl ZeroizeOnDrop for RsaPrivateKey {}

#[derive(Clone)]
pub(crate) struct PrecomputedValues {
    /// D mod (P-1)
    pub(crate) dp: BoxedUint,
    /// D mod (Q-1)
    pub(crate) dq: BoxedUint,
    ///Q^-1 mod P
    pub(crate) qinv: BoxedMontyForm,

    /// Montgomery params for `p`
    pub(crate) p_params: BoxedMontyParams,
    /// Montgomery params for `q`
    pub(crate) q_params: BoxedMontyParams,
}

impl ZeroizeOnDrop for PrecomputedValues {}

impl Zeroize for PrecomputedValues {
    fn zeroize(&mut self) {
        self.dp.zeroize();
        self.dq.zeroize();
        // TODO: once these have landed in crypto-bigint
        // self.p_params.zeroize();
        // self.q_params.zeroize();
    }
}

impl Drop for PrecomputedValues {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl From<RsaPrivateKey> for RsaPublicKey {
    fn from(private_key: RsaPrivateKey) -> Self {
        (&private_key).into()
    }
}

impl From<&RsaPrivateKey> for RsaPublicKey {
    fn from(private_key: &RsaPrivateKey) -> Self {
        let n = PublicKeyParts::n(private_key);
        let e = PublicKeyParts::e(private_key);
        let n_params = PublicKeyParts::n_params(private_key);
        RsaPublicKey {
            n: n.clone(),
            e: e.clone(),
            n_params: n_params.clone(),
        }
    }
}

pub trait PublicKeyParts {
    /// Returns the modulus of the key
    fn n(&self) -> &NonZero<BoxedUint>;

    /// Returns the public exponent of the key
    fn e(&self) -> &BoxedUint;

    /// Returns the modulus size in bytes.
    /// Raw signatures and ciphertexts will have the same size
    fn size(&self) -> usize {
        (self.n().bits() as usize).div_ceil(8)
    }

    /// Returns the parameters for montgomery operations
    fn n_params(&self) -> &BoxedMontyParams;

    /// Returns precision (in bits) of `n
    fn n_bits_precision(&self) -> u32 {
        self.n().bits_precision()
    }
}

impl PublicKeyParts for RsaPublicKey {
    fn n(&self) -> &NonZero<BoxedUint> {
        &self.n
    }

    fn e(&self) -> &BoxedUint {
        &self.e
    }

    fn n_params(&self) -> &BoxedMontyParams {
        &self.n_params
    }
}

impl PublicKeyParts for RsaPrivateKey {
    fn n(&self) -> &NonZero<BoxedUint> {
        &self.pubkey.n
    }

    fn e(&self) -> &BoxedUint {
        &self.pubkey.e
    }

    fn n_params(&self) -> &BoxedMontyParams {
        &self.pubkey.n_params
    }
}

/// Components of an RSA private key
pub trait PrivateKeyParts: PublicKeyParts {
    /// Returns the private exponent of the key
    fn d(&self) -> &BoxedUint;

    /// Returns the prime factors
    fn primes(&self) -> &[BoxedUint];

    /// Returns the precomputed dp value, D mod (P-1)
    fn dp(&self) -> Option<&BoxedUint>;

    /// Returns the precomputed dq value, D mod (Q-1)
    fn dq(&self) -> Option<&BoxedUint>;

    /// Returns the precomputed qinv value, Q^-1 mod P
    fn qinv(&self) -> Option<&BoxedMontyForm>;

    /// Returns the params for `p` if precomputed.
    fn p_params(&self) -> Option<&BoxedMontyParams>;

    /// Returns the params for `q` if precomputed.
    fn q_params(&self) -> Option<&BoxedMontyParams>;
}

impl PrivateKeyParts for RsaPrivateKey {
    fn d(&self) -> &BoxedUint {
        &self.d
    }

    fn primes(&self) -> &[BoxedUint] {
        &self.primes
    }

    fn dp(&self) -> Option<&BoxedUint> {
        self.precomputed.as_ref().map(|p| &p.dp)
    }

    fn dq(&self) -> Option<&BoxedUint> {
        self.precomputed.as_ref().map(|p| &p.dq)
    }

    fn qinv(&self) -> Option<&BoxedMontyForm> {
        self.precomputed.as_ref().map(|p| &p.qinv)
    }

    fn p_params(&self) -> Option<&BoxedMontyParams> {
        self.precomputed.as_ref().map(|p| &p.p_params)
    }

    fn q_params(&self) -> Option<&BoxedMontyParams> {
        self.precomputed.as_ref().map(|p| &p.q_params)
    }
}

impl RsaPrivateKey {
    /// Default exponent for RSA keys
    const EXP: u64 = 65537;

    /// Minimum size of the modulus `n` in bits.
    const MIN_SIZE: u32 = 1024;

    /// Generates a new RSA key pair with a modulus of the given bit size using the passed in `rng`
    pub fn new<R: CryptoRng + ?Sized>(rng: &mut R, bit_size: usize) -> Result<Self> {
        Self::new_with_exp(rng, bit_size, Self::EXP.into())
    }

    /// Generates a new RSA key pair of the given bit size and the public exponent
    pub fn new_with_exp<R: CryptoRng + ?Sized>(
        rng: &mut R,
        bit_size: usize,
        exp: BoxedUint,
    ) -> Result<RsaPrivateKey> {
        if bit_size < Self::MIN_SIZE as usize {
            return Err(Error::ModulusTooSmall);
        }

        let components = generate_multi_prime_key_with_exp(rng, 2, bit_size, exp)?;
        RsaPrivateKey::from_components(
            components.n.get(),
            components.e,
            components.d,
            components.primes,
        )
    }

    /// Constructs an RSA key pair from individual components:
    /// - `n`: RSA modulus
    /// - `e`: public exponent
    /// - `d`: private exponent
    /// - `primes`: prime factors of `n`, typically two primes `p` and `q`.
    ///        If no `primes` are provided, a prime factor recovery algorithm will be employed
    pub fn from_components(
        n: BoxedUint,
        e: BoxedUint,
        d: BoxedUint,
        mut primes: Vec<BoxedUint>,
    ) -> Result<RsaPrivateKey> {
        let n = Odd::new(n).into_option().ok_or(Error::InvalidModulus)?;

        // The modulus may come in padded with zeros, shorten it to ensure optimal performance
        let n_bits = n.bits_vartime();
        let n = n.resize_unchecked(n_bits);

        let n_params = BoxedMontyParams::new(n.clone());
        let n_c = NonZero::new(n.get())
            .into_option()
            .ok_or(Error::InvalidModulus)?;

        match primes.len() {
            0 => {
                // Recover `p` and `q` from `d`
                let (p, q) = recover_primes(&n_c, &e, &d)?;
                primes.push(p);
                primes.push(q);
            }
            1 => return Err(Error::NprimesTooSmall),
            _ => {
                // Check that the product of primes matches the modulus
                if &primes.iter().fold(BoxedUint::one(), |acc, p| acc * p) != n_c.as_ref() {
                    return Err(Error::InvalidModulus);
                }
            }
        }

        // The primes may come in padded with zeros too, so we need to shorten them as well
        let primes = primes
            .into_iter()
            .map(|p| {
                let p_bits = p.bits();
                p.resize_unchecked(p_bits)
            })
            .collect();

        let mut k = RsaPrivateKey {
            pubkey: RsaPublicKey {
                n: n_c,
                e,
                n_params,
            },
            d,
            primes,
            precomputed: None,
        };

        // Always validate the key, to ensure precompute can't fail
        k.validate()?;

        // Precompute when possible, ignore otherwise
        k.precompute().ok();

        Ok(k)
    }

    /// Performs some calculations to speed up private key operations
    pub fn precompute(&mut self) -> Result<()> {
        if self.precomputed.is_some() {
            return Ok(());
        }

        let d = &self.d;
        let p = self.primes[0].clone();
        let q = self.primes[1].clone();

        let p_odd = Odd::new(p.clone())
            .into_option()
            .ok_or(Error::InvalidPrime)?;
        let p_params = BoxedMontyParams::new(p_odd);
        let q_odd = Odd::new(q.clone())
            .into_option()
            .ok_or(Error::InvalidPrime)?;
        let q_params = BoxedMontyParams::new(q_odd);

        let x = NonZero::new(p.wrapping_sub(&BoxedUint::one()))
            .into_option()
            .ok_or(Error::InvalidPrime)?;
        let dp = d.rem_vartime(&x);

        let x = NonZero::new(q.wrapping_sub(&BoxedUint::one()))
            .into_option()
            .ok_or(Error::InvalidPrime)?;
        let dq = d.rem_vartime(&x);

        // Note: `p` and `q` may have different `bits_precision`
        let q_mod_p = match p.bits_precision().cmp(&q.bits_precision()) {
            Ordering::Less => (&q
                % NonZero::new(p.clone())
                    .expect("`p` is non-zero")
                    .resize_unchecked(q.bits_precision()))
            .resize_unchecked(p.bits_precision()),
            Ordering::Greater => {
                (&q).resize_unchecked(p.bits_precision())
                    % &NonZero::new(p.clone()).expect("`p` is non-zero")
            }
            Ordering::Equal => &q % NonZero::new(p.clone()).expect("`p` is non-zero"),
        };

        let q_mod_p = BoxedMontyForm::new(q_mod_p, p_params.clone());
        let qinv = q_mod_p.invert().into_option().ok_or(Error::InvalidPrime)?;

        debug_assert_eq!(dp.bits_precision(), p.bits_precision());
        debug_assert_eq!(dq.bits_precision(), q.bits_precision());
        debug_assert_eq!(qinv.bits_precision(), p.bits_precision());
        debug_assert_eq!(p_params.bits_precision(), p.bits_precision());
        debug_assert_eq!(q_params.bits_precision(), q.bits_precision());

        self.precomputed = Some(PrecomputedValues {
            dp,
            dq,
            qinv,
            p_params,
            q_params,
        });

        Ok(())
    }

    /// Performs basic sanity checks on the key
    pub fn validate(&self) -> Result<()> {
        check_public(self)?;

        // Check that Product of primes == n
        let mut m = BoxedUint::one_with_precision(self.pubkey.n.bits_precision());
        let one = BoxedUint::one();
        for prime in &self.primes {
            if prime < &one {
                return Err(Error::InvalidPrime);
            }
            m = m.wrapping_mul(prime);
        }
        if m != *self.pubkey.n {
            return Err(Error::InvalidModulus);
        }

        // Check that de ≡ 1 mod p-1, for each prime.
        // This implies that e is coprime to each p-1 as e has a multiplicative
        // inverse. Therefore e is comprime to lcm(p-1,q-1,r-1,...) =
        // Check that de ≡ 1 mod p-1, for each prime.
        let de = self.d.mul(&self.pubkey.e);

        for prime in &self.primes {
            let x = NonZero::new(prime.wrapping_sub(&BoxedUint::one())).unwrap();
            let congruence = de.rem_vartime(&x);
            if !bool::from(congruence.is_one()) {
                return Err(Error::InvalidExponent);
            }
        }

        Ok(())
    }

    /// Sign the given digest
    pub fn sign<S: SignatureScheme>(&self, padding: S, digest_in: &[u8]) -> Result<Vec<u8>> {
        padding.sign(Option::<&mut DummyRng>::None, self, digest_in)
    }

    /// Decrypt the given message
    pub fn decrypt<P: PaddingScheme>(&self, padding: P, ciphertext: &[u8]) -> Result<Vec<u8>> {
        padding.decrypt(Option::<&mut DummyRng>::None, self, ciphertext)
    }
}

impl RsaPublicKey {
    /// Minimum value of the public exponent `e`.
    pub const MIN_PUB_EXPONENT: u64 = 2;

    /// Maximum value of the public exponent `e`.
    pub const MAX_PUB_EXPONENT: u64 = (1 << 33) - 1;

    /// Verify a signed message
    pub fn verify<S: SignatureScheme>(&self, scheme: S, hashed: &[u8], sig: &[u8]) -> Result<()> {
        scheme.verify(self, hashed, sig)
    }

    /// Encrypt the given message
    pub fn encrypt<R: CryptoRng + ?Sized, P: PaddingScheme>(
        &self,
        rng: &mut R,
        padding: P,
        msg: &[u8],
    ) -> Result<Vec<u8>> {
        padding.encrypt(rng, self, msg)
    }
}

/// Check that the public key is well formed and has an exponent within acceptable bounds
#[inline]
pub fn check_public(public_key: &impl PublicKeyParts) -> Result<()> {
    check_public_with_max_size(public_key.n(), public_key.e(), None)
}

/// Check that the public key is well formed and has an exponent within acceptable bounds
#[inline]
fn check_public_with_max_size(n: &BoxedUint, e: &BoxedUint, max_size: Option<usize>) -> Result<()> {
    if let Some(max_size) = max_size
        && n.bits_vartime() as usize > max_size
    {
        return Err(Error::ModulusTooLarge);
    }

    if e >= n || n.is_even().into() || n.is_zero().into() {
        return Err(Error::InvalidModulus);
    }

    if e.is_even().into() {
        return Err(Error::InvalidExponent);
    }

    if e < &BoxedUint::from(RsaPublicKey::MIN_PUB_EXPONENT) {
        return Err(Error::PublicExponentTooSmall);
    }

    if e > &BoxedUint::from(RsaPublicKey::MAX_PUB_EXPONENT) {
        return Err(Error::PublicExponentTooLarge);
    }

    Ok(())
}
