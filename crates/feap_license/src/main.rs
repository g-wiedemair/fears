extern crate core;

fn main() {
    // Read the

    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_traits::Num;

    #[derive(Debug)]
    struct Issuer {
        name: String,
        n: BigInt,
        e: u32,
    }

    struct U16Digits<'a> {
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

    trait ConvertU16 {
        fn to_u16_digits(&self) -> Vec<u16>;
    }

    impl ConvertU16 for BigInt {
        fn to_u16_digits(&self) -> Vec<u16> {
            let (_, data) = self.to_u32_digits();
            U16Digits::new(&data[..]).collect()
        }
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
        ISSUER license@feap.at CDC3_D1F3_D813_457B_81B1_6BD4_90AD_96A1_A6F8 11\n\
        LICENSE HERODOT Fenda5.4 31-12-2020 license@feap.at 24D7_9A13\n\
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
        assert_eq!(checksum, 0xf224b759);
    }
}
