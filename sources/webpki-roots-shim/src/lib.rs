#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

#[macro_use]
extern crate log;

use pem::Pem;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use webpki::trust_anchor_util::cert_der_as_trust_anchor;
use webpki::{TLSServerTrustAnchors, TrustAnchor};

lazy_static::lazy_static! {
    pub static ref TLS_SERVER_ROOTS: TLSServerTrustAnchors<'static> = {
        debug!("webpki-roots-shim activated, {} certificates", ROOTS.len());
        debug!("certificate source: {}", CERT_PATH.display());
        TLSServerTrustAnchors(&ROOTS)
    };

    static ref CERT_PATH: Cow<'static, Path> = match std::env::var_os("SSL_CERT_FILE") {
        Some(var) => PathBuf::from(var).into(),
        None => Path::new("/etc/pki/tls/certs/ca-bundle.crt").into(),
    };

    static ref ROOTS_PEM: Vec<Pem> = tls_server_roots_pem();
    static ref ROOTS: Vec<TrustAnchor<'static>> = tls_server_roots(&ROOTS_PEM);
}

fn tls_server_roots_pem() -> Vec<Pem> {
    match std::fs::read(&*CERT_PATH) {
        Ok(data) => {
            let mut v = match pem::parse_many(&data) {
                Ok(v) => v,
                Err(err) => {
                    error!("failed to parse {}: {}", CERT_PATH.display(), err);
                    Vec::new()
                }
            };
            v.shrink_to_fit();
            v
        }
        Err(err) => {
            error!("failed to read {}: {}", CERT_PATH.display(), err);
            Vec::new()
        }
    }
}

fn tls_server_roots(pem: &[Pem]) -> Vec<TrustAnchor<'_>> {
    pem.iter()
        .filter_map(|pem| cert_der_as_trust_anchor(&pem.contents).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{tls_server_roots, tls_server_roots_pem};
    use webpki::TrustAnchor;

    static AMAZON_CA_3: TrustAnchor<'_> = TrustAnchor {
        subject: b"1\x0b0\t\x06\x03U\x04\x06\x13\x02US1\x0f0\r\x06\x03U\x04\n\x13\x06Amazon1\x190\x17\x06\x03U\x04\x03\x13\x10Amazon Root CA 3",
        spki: b"0\x13\x06\x07*\x86H\xce=\x02\x01\x06\x08*\x86H\xce=\x03\x01\x07\x03B\x00\x04)\x97\xa7\xc6A\x7f\xc0\r\x9b\xe8\x01\x1bV\xc6\xf2R\xa5\xba-\xb2\x12\xe8\xd2.\xd7\xfa\xc9\xc5\xd8\xaam\x1fs\x81;;\x98k9|3\xa5\xc5N\x86\x8e\x80\x17hbEW}DX\x1d\xb37\xe5g\x08\xebf\xde",
        name_constraints: None,
    };
    static AMAZON_CA_4: TrustAnchor<'_> = TrustAnchor {
        subject: b"1\x0b0\t\x06\x03U\x04\x06\x13\x02US1\x0f0\r\x06\x03U\x04\n\x13\x06Amazon1\x190\x17\x06\x03U\x04\x03\x13\x10Amazon Root CA 4",
        spki: b"0\x10\x06\x07*\x86H\xce=\x02\x01\x06\x05+\x81\x04\x00\"\x03b\x00\x04\xd2\xab\x8a7O\xa3S\r\xfe\xc1\x8a{K\xa8{FKc\xb0b\xf6-\x1b\xdb\x08q!\xd2\x00\xe8c\xbd\x9a\'\xfb\xf09n]\xea=\xa5\xc9\x81\xaa\xa3[ \x98E]\x16\xdb\xfd\xe8\x10m\xe3\x9c\xe0\xe3\xbd_\x84b\xf3pd3\xa0\xcb$/p\xba\x88\xa1*\xa0u\xf8\x81\xaeb\x06\xc4\x81\xdb9n)\xb0\x1e\xfa.\\",
        name_constraints: None,
    };

    #[test]
    fn test() {
        std::env::set_var(
            "SSL_CERT_FILE",
            format!("{}/tests/data/cacert.pem", env!("CARGO_MANIFEST_DIR")),
        );
        let roots = tls_server_roots_pem();
        let roots = tls_server_roots(&roots);

        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].subject, AMAZON_CA_3.subject);
        assert_eq!(roots[0].spki, AMAZON_CA_3.spki);
        assert_eq!(roots[0].name_constraints, AMAZON_CA_3.name_constraints);
        assert_eq!(roots[1].subject, AMAZON_CA_4.subject);
        assert_eq!(roots[1].spki, AMAZON_CA_4.spki);
        assert_eq!(roots[1].name_constraints, AMAZON_CA_4.name_constraints);
    }
}
