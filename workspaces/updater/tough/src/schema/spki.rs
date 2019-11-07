//! Functions for extracting key data from and encoding key data to `SubjectPublicKeyInfo`
//! documents.
//!
//! For RSA, the TUF specification implies [1] the use of public keys in the `SubjectPublicKeyInfo`
//! format, while `ring` works in the `RSAPublicKey` format [2]. The former is just a wrapper
//! around the latter.
//!
//! The output of `openssl asn1parse -i` for a public key looks like:
//! ```plain
//!    0:d=0  hl=4 l= 418 cons: SEQUENCE
//!    4:d=1  hl=2 l=  13 cons:  SEQUENCE
//!    6:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
//!   17:d=2  hl=2 l=   0 prim:   NULL
//!   19:d=1  hl=4 l= 399 prim:  BIT STRING
//! ```
//!
//! [1]: https://github.com/theupdateframework/tuf/blob/49e75ffe5adfc1f883f53f658ace596d14dc0879/tests/repository_data/repository/metadata/root.json#L20
//! [2]: https://docs.rs/ring/0.14.6/ring/signature/index.html#signing-and-verifying-with-rsa-pkcs1-15-padding

use super::error::{self, Compat, Result};
use ring::io::der;
use snafu::{OptionExt, ResultExt};

pub(super) static OID_RSA_ENCRYPTION: &[u64] = &[1, 2, 840, 113_549, 1, 1, 1];
pub(super) static OID_EC_PUBLIC_KEY: &[u64] = &[1, 2, 840, 10_045, 2, 1];
pub(super) static OID_EC_PARAM_SECP256R1: &[u64] = &[1, 2, 840, 10_045, 3, 1, 7];

/// Wrap a bit string in a `SubjectPublicKeyInfo` document.
pub(super) fn encode(algorithm_oid: &[u64], parameters_oid: Option<&[u64]>, b: &[u8]) -> String {
    let mut alg_ident = asn1_tag(der::Tag::OID, asn1_encode_oid(algorithm_oid));
    alg_ident.extend(match parameters_oid {
        Some(oid) => asn1_tag(der::Tag::OID, asn1_encode_oid(oid)),
        None => asn1_tag(der::Tag::Null, Vec::new()),
    });
    let alg_ident = asn1_tag(der::Tag::Sequence, alg_ident);

    let mut bit_string = vec![0];
    bit_string.extend_from_slice(b);
    let bit_string = asn1_tag(der::Tag::BitString, bit_string);

    let mut sequence = alg_ident;
    sequence.extend(bit_string);

    let spki = asn1_tag(der::Tag::Sequence, sequence);

    pem::encode_config(
        &pem::Pem {
            tag: "PUBLIC KEY".to_owned(),
            contents: spki,
        },
        &pem::EncodeConfig {
            line_ending: pem::LineEnding::LF,
        },
    )
    .trim()
    .to_owned()
}

/// Extract the bit string from a PEM-encoded `SubjectPublicKeyInfo` document.
pub(super) fn decode(
    algorithm_oid: &[u64],
    parameters_oid: Option<&[u64]>,
    input: &str,
) -> Result<Vec<u8>> {
    let pem = pem::parse(input)
        .map_err(Compat)
        .context(error::PemDecode)?;
    Ok(untrusted::Input::from(&pem.contents)
        .read_all(ring::error::Unspecified, |input| {
            der::expect_tag_and_get_value(input, der::Tag::Sequence).and_then(|spki| {
                spki.read_all(ring::error::Unspecified, |input| {
                    der::expect_tag_and_get_value(input, der::Tag::Sequence).and_then(
                        |alg_ident| {
                            alg_ident.read_all(ring::error::Unspecified, |input| {
                                if der::expect_tag_and_get_value(input, der::Tag::OID)?
                                    != untrusted::Input::from(&asn1_encode_oid(algorithm_oid))
                                {
                                    return Err(ring::error::Unspecified);
                                }
                                if let Some(parameters_oid) = parameters_oid {
                                    if der::expect_tag_and_get_value(input, der::Tag::OID)?
                                        != untrusted::Input::from(&asn1_encode_oid(parameters_oid))
                                    {
                                        return Err(ring::error::Unspecified);
                                    }
                                } else {
                                    der::expect_tag_and_get_value(input, der::Tag::Null)?;
                                }
                                Ok(())
                            })
                        },
                    )?;
                    der::bit_string_with_no_unused_bits(input)
                })
            })
        })
        .ok()
        .context(error::SpkiDecode)?
        .as_slice_less_safe()
        .to_owned())
}

fn asn1_tag(tag: der::Tag, data: Vec<u8>) -> Vec<u8> {
    let mut v = vec![tag as u8];
    v.extend(asn1_encode_len(data.len()));
    v.extend(data);
    v
}

/// Encode a length in ASN.1.
#[allow(clippy::cast_possible_truncation)]
fn asn1_encode_len(n: usize) -> Vec<u8> {
    if n < 128 {
        vec![n as u8]
    } else {
        let n = n.to_be_bytes();
        let skip_bytes = n.iter().position(|b| *b != 0).unwrap_or_else(|| n.len());
        let mut v = vec![0_u8; n.len() - skip_bytes + 1];
        v[0] = 0x80 | (n.len() - skip_bytes) as u8;
        v[1..].copy_from_slice(&n[skip_bytes..]);
        v
    }
}

/// Encode an object identifier in ASN.1.
#[allow(clippy::cast_possible_truncation)]
fn asn1_encode_oid(oid: &[u64]) -> Vec<u8> {
    let mut v = Vec::new();
    v.push((oid.get(0).unwrap_or(&0) * 40 + oid.get(1).unwrap_or(&0)) as u8);
    for n in oid.iter().skip(2) {
        v.extend(&to_vlq(*n));
    }
    v
}

/// Encode an integer as a variable-length quality (used in ASN.1 encoding of object identifiers).
#[allow(clippy::cast_possible_truncation)]
fn to_vlq(n: u64) -> Vec<u8> {
    if n == 0 {
        return vec![0];
    }

    let significant_bytes = (std::mem::size_of::<u64>() * 8) as u32 - n.leading_zeros();
    let count = significant_bytes / 7 + std::cmp::min(1, significant_bytes % 7) - 1;
    let mut v = Vec::with_capacity(count as usize + 1);
    let mut n = n.rotate_right(count * 7);
    for _ in 0..count {
        v.push((n & 0x7f) as u8 | 0x80);
        n = n.rotate_left(7);
    }
    v.push((n & 0x7f) as u8);
    v
}

#[cfg(test)]
mod tests {
    use super::{asn1_encode_len, asn1_encode_oid, to_vlq, OID_RSA_ENCRYPTION};

    #[test]
    fn test_asn1_encode_len() {
        assert_eq!(asn1_encode_len(0x00), [0x00]);
        assert_eq!(asn1_encode_len(0x42), [0x42]);
        assert_eq!(asn1_encode_len(0x84), [0x81, 0x84]);
        assert_eq!(asn1_encode_len(0x01_23), [0x82, 0x01, 0x23]);
        assert_eq!(asn1_encode_len(0x04_56_78), [0x83, 0x04, 0x56, 0x78]);
        assert_eq!(asn1_encode_len(0xffff_ffff), [0x84, 0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn test_asn1_encode_oid() {
        assert_eq!(
            asn1_encode_oid(OID_RSA_ENCRYPTION),
            [0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01]
        );
    }

    #[test]
    fn test_to_vlq() {
        assert_eq!(to_vlq(0), [0x00]);
        assert_eq!(to_vlq(0x7f), [0x7f]);
        assert_eq!(to_vlq(0x80), [0x81, 0x00]);
        assert_eq!(to_vlq(0x2000), [0xc0, 0x00]);
        assert_eq!(to_vlq(0x3fff), [0xff, 0x7f]);
        assert_eq!(to_vlq(0x4000), [0x81, 0x80, 0x00]);
        assert_eq!(to_vlq(0x001f_ffff), [0xff, 0xff, 0x7f]);
        assert_eq!(to_vlq(0x0020_0000), [0x81, 0x80, 0x80, 0x00]);
        assert_eq!(to_vlq(0x0800_0000), [0xc0, 0x80, 0x80, 0x00]);
        assert_eq!(to_vlq(0x0fff_ffff), [0xff, 0xff, 0xff, 0x7f]);
    }
}
