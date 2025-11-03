mod algorithms;
mod dummy;
mod error;
mod key;
mod padding;
mod rsa;
mod signature;

use crate::{key::RsaPrivateKey, key::RsaPublicKey, rsa::RsaSign};
use chrono::Datelike;
use crypto_bigint::BoxedUint;
use std::{env, fs, process::exit};

// Constant factors for keys used in feap
const P: &str = "D309FABB6FAB18A517348D00D7770E405066128C6C3379CD22EC13EE2D3B89FC986D4DE1BA38E3109EB8825D760AA56A38E6DF2A3547D0427823C9380FA09ACD";
const Q: &str = "F0EDB7BC115B6D299C86D89F049F466263176C8383715501520F14C7ED85B9AD44A25111BEE5C85DF0830204847A08C2EA8D31C1BE609901F5DA5DEAFB90FA8B";
const N: &str = "C69D52C40415EF2A074B20475D0243EFCF7035887DBEFEC8BBA10735824D6F9E524A7EF96DD843E382B70B7A16B24D1A7EFB6AB5FEC45A16D048215A1D27BADF8A082003B65B54028C6D7559372430030867F38EAFB80DBECCACB7CD14BE2E525D3B6684208ACD98126A97EB63A20BB41BCB213671141577F9663B3365B03F4F";
const E: &str = "10001";
const D: &str = "69019A5589F772C7DCAD4A769064F7381D8B2CB26A1105B16909BCBEFC9226362539BFA1EE024DFA460CB2A3ACC63DDF894D3160E13E3C871D3D556CC8474E139C78E9021CDC265CE0EF5B42DB321B4EF5F632FB245FC881D74B47E494ECB3BFBC6BE909C08A7A114A8B24201D13EA73C66E796D789BE9A222CD36F66271DF39";

// Issuer
const ISSUER: &str = "license@feap.at";
// Feap version
const FENDA_VERSION: &str = "Fenda5.4";

#[allow(dead_code)]
fn create_keys() -> RsaPrivateKey {
    let mut rng = rand::rng();
    let bits = 1024;
    RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate a key")
}

fn create_keys_from_components() -> RsaPrivateKey {
    RsaPrivateKey::from_components(
        BoxedUint::from_str_radix_vartime(N, 16).unwrap(),
        BoxedUint::from_str_radix_vartime(E, 16).unwrap(),
        BoxedUint::from_str_radix_vartime(D, 16).unwrap(),
        vec![
            BoxedUint::from_str_radix_vartime(P, 16).unwrap(),
            BoxedUint::from_str_radix_vartime(Q, 16).unwrap(),
        ],
    )
    .expect("Failed to generate a key")
}

fn print_help() {
    println!("Usage: feap_license [options]");
    println!("Options:");
    println!("  -h, --help          Print this help message");
    println!("  -f, --file          Read computer profile from file");
    println!("  -d, --date          expiration date of the license in format DD-MM-YYYY")
}

fn main() {
    let mut args = env::args();
    let mut file_path = "".to_string();
    let mut date = "".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-f" | "--file" => {
                file_path = args.next().expect("Missing file path after '-f'");
            }
            "-d" | "--date" => {
                date = args.next().expect("Missing date after '-d'");
            }
            _ => {
                // do nothing
            }
        }
    }

    if file_path.is_empty() {
        print_help();
        exit(1);
    }
    if date.is_empty() {
        let current_date = chrono::Utc::now();
        let year = current_date.year();
        date = format!("31-12-{year}");
    }

    let profile_data = fs::read_to_string(file_path).expect("Failed to read file");
    let profile = profile_data
        .lines()
        .filter(|line| line.starts_with("ComputerName") || line.starts_with("Profile"))
        .map(|line| line.split_once("=").unwrap().1)
        .collect::<String>();

    let mut profile_str = profile.split_whitespace();
    let profile_name = profile_str.next().expect("Failed to read profile name");
    let profile_uid = profile_str.next().expect("Failed to read profile uid");

    let priv_key = create_keys_from_components();
    let pub_key = RsaPublicKey::from(&priv_key);

    let data = format!(
        "{}  {}                      {}",
        profile_uid, FENDA_VERSION, date
    );
    let enc_data = priv_key
        .sign(RsaSign, data.as_bytes())
        .expect("failed to encrypt");

    // Verify
    assert!(pub_key.verify(RsaSign, data.as_bytes(), &enc_data).is_ok());

    // Write license file
    let license = &enc_data
        .iter()
        .map(|word| format!("{:02X}", word))
        .collect::<String>();

    let license_data = format!(
        "\
    ISSUER {ISSUER} {N} {E}\n\
    LICENSE {profile_name} {FENDA_VERSION} {date} {ISSUER} {license}\n\
    "
    );
    fs::write("license.dat", license_data).expect("Failed to write license file");
}

#[cfg(test)]
mod tests {
    use crate::{D, E, N, P, Q};
    use crate::{
        create_keys_from_components,
        key::RsaPublicKey,
        rsa::{RsaEncrypt, RsaSign},
    };
    use num_bigint::{BigInt, Sign};
    use num_traits::Num;
    use std::ops::{Mul, Sub};

    #[test]
    fn check_rsa_sign() {
        let priv_key = create_keys_from_components();
        let pub_key = RsaPublicKey::from(&priv_key);

        // Sign
        let data = b"Hello World!";
        let enc_data = priv_key
            .sign(RsaSign, &data[..])
            .expect("failed to encrypt");
        assert_ne!(&data[..], &enc_data[..]);

        // Verify
        assert!(pub_key.verify(RsaSign, data, &enc_data).is_ok());

        // Test with fenda decrypter
        let sec_cert = BigInt::from_bytes_be(Sign::Plus, &enc_data);
        let e = BigInt::from_str_radix(E, 16).unwrap();
        let n = BigInt::from_str_radix(N, 16).unwrap();
        let cert = sec_cert.modpow(&e, &n);
        let words = cert.to_u16_digits();
        let result = words
            .into_iter()
            .map(|word| {
                let hi = word >> 8;
                let lo = word & 0xff;
                format!(
                    "{:?}{:?}",
                    std::char::from_u32(hi as u32).unwrap_or(' '),
                    std::char::from_u32(lo as u32).unwrap_or(' '),
                )
            })
            .rev()
            .collect::<String>()
            .replace("'", "");
        assert_eq!(&result[..], "Hello World!");
    }

    #[test]
    fn check_rsa_encrypt() {
        let mut rng = rand::rng();
        let priv_key = create_keys_from_components();
        let pub_key = RsaPublicKey::from(&priv_key);

        // Encrypt
        let data = b"Hello World!";
        let enc_data = pub_key
            .encrypt(&mut rng, RsaEncrypt, &data[..])
            .expect("failed to encrypt");
        assert_ne!(&data[..], &enc_data[..]);

        // Decrypt
        let dec_data = priv_key
            .decrypt(RsaEncrypt, &enc_data)
            .expect("failed to decrypt");
        assert_eq!(&data[..], &dec_data[..]);
    }

    #[test]
    fn check_simple_rsa_with_real_key() {
        let p = BigInt::from_str_radix(P, 16).unwrap();
        let q = BigInt::from_str_radix(Q, 16).unwrap();
        let n = p.clone() * q.clone();
        let t = p.sub(BigInt::from(1u32)).mul(q.sub(BigInt::from(1u32)));
        let e = BigInt::from_str_radix(E, 16).unwrap();
        assert!(e < t);
        assert_ne!(e.modpow(&BigInt::from(1), &t), BigInt::from(0u32));

        let d = BigInt::from_str_radix(D, 16).unwrap();
        assert_eq!((&d * &e).modpow(&BigInt::from(1), &t), BigInt::from(1u32));

        let message = BigInt::from(99u32);
        let enc_data = message.modpow(&e, &n);

        let dec_data = enc_data.modpow(&d, &n);
        assert_eq!(dec_data, message);
    }

    /*
       RSA Encryption
        - Select two Prime Numbers
          P = 7, Q = 19
          Product N = P*Q = 133
          Totient T = (P-1)*(Q-1) = 108
        - Public Key
          - must be a prime
          - must be less than the totient
          - must NOT be a factor of the totient
          E = 29
        - Private Key
          D => (D*E) mod T = 1
          D = 41
        - Encryption
          M^E mod N
        - Decryption
          M^D mod N
    */
    #[test]
    fn check_simple_rsa() {
        let p = BigInt::from(7u32);
        let q = BigInt::from(19u32);
        let n = p.clone() * q.clone();
        let t = p.sub(BigInt::from(1u32)).mul(q.sub(BigInt::from(1u32)));
        let e = BigInt::from(29u32);
        assert!(e < t);
        assert_ne!(e.modpow(&BigInt::from(1), &t), BigInt::from(0u32));

        let d = BigInt::from(41u32);
        assert_eq!((&d * &e).modpow(&BigInt::from(1), &t), BigInt::from(1u32));

        let message = BigInt::from(99u32);
        let enc_data = message.modpow(&e, &n);
        assert_eq!(enc_data, BigInt::from(92u32));

        let dec_data = enc_data.modpow(&d, &n);
        assert_eq!(dec_data, message);
    }

    #[derive(Debug)]
    struct Issuer {
        pub name: String,
        pub n: BigInt,
        pub e: u32,
    }

    impl Issuer {
        pub fn checksum(&mut self) -> u32 {
            let mut salt = (0x81a35b9d * self.e as u64) as u32;
            let words = self.n.to_u16_digits();
            let n = words.len();

            let lword = words
                .into_iter()
                .enumerate()
                .fold(0u32, |mut acc, (i, word)| {
                    if i & 1 > 0 {
                        acc = acc + ((word as u32) << 16);
                        salt ^= acc as u32;
                    } else {
                        acc = word as u32;
                    }
                    acc
                });

            if n & 1 > 0 {
                salt ^= lword as u32;
            }

            salt
        }
    }

    #[test]
    fn check_license() {
        let license = "\
            ISSUER license@feap.at CDC3D1F3D813457B81B16BD490AD96A1A6F8B57800BCF737D7279CDAD2761CCAAA312B13DEA8FFBCCB7005A281C570E7F08687642DD0FFA37C40BC7844795EEB730C2B5148C8B644BB54C27758A98FD9C9D317098C04C95EA1AFBE113D6ED9ED76C897C0D7F078067676153BCC676C5D16CDCBDED44420BEEBDAC040ED0CC81B 11\n\
            LICENSE HERODOT Fenda5.1 31-12-2017 license@feap.at 24D79A1343900646FABAF6D5F4E40D7395B910328E403AEB5E7D251EE10353B0DDDB5ED93DE5A5AB2023B1409C437C10F04C22383DEDEA3783C92FF10324B66D422DE68313A6A6877BA3A99ABFD23FBB04B98B35D52BAB8C5CD97665AA58D05046CF544AA664D334C0B95FE2EF46446EC7201C1DF48E3C8295E4A5B3306C589\n\
            ";

        let license_words = license.split_whitespace().collect::<Vec<&str>>();

        // Issuer
        assert_eq!(license_words[0], "ISSUER");
        let mut issuer = Issuer {
            name: license_words[1].into(),
            n: BigInt::from_str_radix(license_words[2], 16)
                .expect("Not a valid hexadecimal number"),
            e: u32::from_str_radix(license_words[3], 16).expect("Not a valid hexadecimal number"),
        };
        let checksum = issuer.checksum();
        assert_eq!(checksum, 0x391a99fa);

        // License
        assert_eq!(license_words[4], "LICENSE");
        let _key = license_words[5];
        let prod_id = license_words[6];
        let date = license_words[7];
        let license_issuer = license_words[8];
        assert_eq!(license_issuer, issuer.name);

        let sec_cert =
            BigInt::from_str_radix(license_words[9], 16).expect("Not a valid hexadecimal number");
        let cert = sec_cert.modpow(&BigInt::from(issuer.e), &issuer.n);

        let words = cert.to_u16_digits();
        let result = words
            .into_iter()
            .map(|word| {
                let hi = word >> 8;
                let lo = word & 0xff;
                format!(
                    "{:?}{:?}",
                    std::char::from_u32(hi as u32).unwrap_or(' '),
                    std::char::from_u32(lo as u32).unwrap_or(' '),
                )
            })
            .rev()
            .collect::<String>()
            .replace("'", "");

        assert_eq!(&result[..38], "{846ee340-7039-11de-9d20-806e6f6e6963}");
        assert_eq!(&result[40..48], prod_id);
        assert_eq!(&result[78..], date);
    }

    pub struct U16Digits<'a> {
        data: &'a [u32],
        next_is_lo: bool,
        last_hi_is_zero: bool,
    }

    impl<'a> U16Digits<'a> {
        #[inline]
        pub(super) fn new(data: &'a [u32]) -> Self {
            let last_hi_is_zero = data
                .last()
                .map(|&last| {
                    let last_hi = (last >> 16) as u16;
                    last_hi == 0
                })
                .unwrap_or(false);
            U16Digits {
                data,
                next_is_lo: true,
                last_hi_is_zero,
            }
        }
    }

    impl Iterator for U16Digits<'_> {
        type Item = u16;

        #[inline]
        fn next(&mut self) -> Option<u16> {
            match self.data.split_first() {
                Some((&first, data)) => {
                    let next_is_lo = self.next_is_lo;
                    self.next_is_lo = !next_is_lo;
                    if next_is_lo {
                        Some(first as u16)
                    } else {
                        self.data = data;
                        if data.is_empty() && self.last_hi_is_zero {
                            self.last_hi_is_zero = false;
                            None
                        } else {
                            Some((first >> 16) as u16)
                        }
                    }
                }
                None => None,
            }
        }
    }

    pub trait ConvertU16 {
        fn to_u16_digits(&self) -> Vec<u16>;
    }

    impl ConvertU16 for BigInt {
        fn to_u16_digits(&self) -> Vec<u16> {
            let (_, data) = self.to_u32_digits();
            U16Digits::new(&data[..]).collect()
        }
    }
}
