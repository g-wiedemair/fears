use crate::error::{Error, Result};
use crypto_bigint::{BoxedUint, NonZero, Odd, Resize};
use crypto_primes::{
    hazmat::{SetBits, SmallFactorsSieveFactory},
    is_prime,
    sieve_and_find, Flavor,
};
use rand::CryptoRng;

pub struct RsaPrivateKeyComponents {
    pub n: Odd<BoxedUint>,
    pub e: BoxedUint,
    pub d: BoxedUint,
    pub primes: Vec<BoxedUint>,
}

/// Generates a multi-prime RSA keypair of the given bit size, public exponent,
/// and the given random source, as suggested in [1]. Although the public
/// keys are compatible (actually, indistinguishable) from the 2-prime case,
/// the private keys are not. Thus it may not be possible to export multi-prime
/// private keys in certain formats or to subsequently import them into other code.
/// [1]: https://patents.google.com/patent/US4405829A/en
pub(crate) fn generate_multi_prime_key_with_exp<R: CryptoRng + ?Sized>(
    rng: &mut R,
    nprimes: usize,
    bit_size: usize,
    exp: BoxedUint,
) -> Result<RsaPrivateKeyComponents> {
    if nprimes < 2 {
        return Err(Error::NprimesTooSmall);
    }

    if bit_size < 64 {
        let prime_limit = (1u64 << (bit_size / nprimes) as u64) as f64;

        // pi approximates the number of primes less than prime_limit
        let mut pi = prime_limit / ((bit_size / nprimes) as f64 * core::f64::consts::LN_2 - 1.);
        // Generated primes start with 0b11, so we can only use a quarter of them
        pi /= 4f64;
        // Use a factor of two to ensure that key generation terminates in a reasonable amount of time
        pi /= 2f64;

        if pi < nprimes as f64 {
            return Err(Error::TooFewPrimes);
        }
    }

    let mut primes = vec![BoxedUint::zero(); nprimes];
    let n_final: Odd<BoxedUint>;
    let d_final: BoxedUint;

    'next: loop {
        let mut todo = bit_size;
        // `generate_prime_with_rng` should set the top two bits in each prime.
        // Thus each prime has the form
        //   p_i = 2^bitlen(p_i) x 0.11... (in base 2).
        // And the product is:
        //   P = 2^todo x a
        // where a is the product of nprimes numbers of the form 0.11...
        //
        // If a < 1/2 (which can happen for nprimes > 2), we need to
        // shift todo to compensate for lost bits: the mean value of 0.11...
        // is 7/8, so todo + shift - nprimes * log2(7/8) ~= bits - 1/2
        // will give good results
        if nprimes >= 7 {
            todo += (nprimes - 2) / 5;
        }

        for (i, prime) in primes.iter_mut().enumerate() {
            let bits = (todo / (nprimes - i)) as u32;
            *prime = generate_prime_with_rng(rng, bits);
            todo -= prime.bits() as usize;
        }

        // Makes sure that primes is pairwise unequal
        for (i, prime1) in primes.iter().enumerate() {
            for prime2 in primes.iter().take(i) {
                if prime1 == prime2 {
                    continue 'next;
                }
            }
        }

        let n = compute_modulus(&primes);

        if n.bits() as usize != bit_size {
            // This should never happen for nprimes == 2
            continue 'next;
        }

        if let Ok(d) = compute_private_exponent_euler_totient(&primes, &exp) {
            n_final = n;
            d_final = d;
            break;
        }
    }

    Ok(RsaPrivateKeyComponents {
        n: n_final,
        e: exp,
        d: d_final,
        primes,
    })
}

fn compute_private_exponent_euler_totient(
    primes: &[BoxedUint],
    exp: &BoxedUint,
) -> Result<BoxedUint> {
    if primes.len() < 2 {
        return Err(Error::NprimesTooSmall);
    }
    let bits = primes[0].bits_precision();
    let mut totient = BoxedUint::one_with_precision(bits);

    for prime in primes {
        totient *= prime - &BoxedUint::one();
    }
    let exp = exp.resize_unchecked(totient.bits_precision());

    // This ensures that `exp` is not a factor of any `(prime - 1)`
    let totient = NonZero::new(totient).expect("known");
    match exp.invert_mod(&totient).into_option() {
        Some(res) => Ok(res),
        None => Err(Error::InvalidPrime),
    }
}

fn compute_modulus(primes: &[BoxedUint]) -> Odd<BoxedUint> {
    let mut primes = primes.iter();
    let mut out = primes.next().expect("must at least be one prime").clone();
    for p in primes {
        out *= p;
    }
    Odd::new(out).expect("modulus must be odd")
}

fn generate_prime_with_rng<R: CryptoRng + ?Sized>(rng: &mut R, bit_length: u32) -> BoxedUint {
    let factory = SmallFactorsSieveFactory::new(Flavor::Any, bit_length, SetBits::TwoMsb)
        .unwrap_or_else(|err| panic!("Error creating the sieve: {err}"));

    sieve_and_find(rng, factory, |_rng, candidate| {
        is_prime(Flavor::Any, candidate)
    })
    .unwrap_or_else(|err| panic!("Error generating random candidates: {err}"))
    .expect("Will produce a result eventually")
}

/// The following (deterministic) algorithm also recovers the prime factors `p` and `q` of a modulus `n`, given the
/// public exponent `e` and private exponent `d` using the method descirbed in
/// [NIST 800-56B Appendix C.2](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-56Br2.pdf).
pub fn recover_primes(
    _n: &NonZero<BoxedUint>,
    _e: &BoxedUint,
    _d: &BoxedUint,
) -> Result<(BoxedUint, BoxedUint)> {
    todo!()
}
