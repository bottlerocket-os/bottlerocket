use crate::error;
use sha2::{Digest, Sha256};
use std::io::{self, Read};
use url::Url;

pub(crate) struct DigestAdapter<T, D> {
    url: Url,
    reader: T,
    hash: Vec<u8>,
    digest: Option<D>,
}

impl<T: Read> DigestAdapter<T, Sha256> {
    pub(crate) fn sha256(reader: T, hash: &[u8], url: Url) -> Self {
        Self {
            url,
            reader,
            hash: hash.to_owned(),
            digest: Some(Sha256::new()),
        }
    }
}

impl<T: Read, D: Digest> Read for DigestAdapter<T, D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        assert!(
            self.digest.is_some(),
            "DigestAdapter::read called after end of file"
        );

        let size = self.reader.read(buf)?;
        if size == 0 {
            let result = std::mem::replace(&mut self.digest, None).unwrap().result();
            if result.as_slice() != self.hash.as_slice() {
                error::HashMismatch {
                    context: self.url.to_string(),
                    calculated: hex::encode(result),
                    expected: hex::encode(&self.hash),
                }
                .fail()?;
            }
            Ok(size)
        } else if let Some(digest) = &mut self.digest {
            digest.input(&buf[..size]);
            Ok(size)
        } else {
            unreachable!();
        }
    }
}

pub(crate) struct MaxSizeAdapter<T> {
    reader: T,
    specifier: &'static str,
    max_size: u64,
    counter: u64,
}

impl<T> MaxSizeAdapter<T> {
    pub(crate) fn new(reader: T, specifier: &'static str, max_size: u64) -> Self {
        Self {
            reader,
            specifier,
            max_size,
            counter: 0,
        }
    }
}

impl<T: Read> Read for MaxSizeAdapter<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = self.reader.read(buf)?;
        self.counter += size as u64;
        if self.counter > self.max_size {
            error::MaxSizeExceeded {
                max_size: self.max_size,
                specifier: self.specifier,
            }
            .fail()?;
        }
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{DigestAdapter, MaxSizeAdapter};
    use hex_literal::hex;
    use std::io::{Cursor, Read};
    use url::Url;

    #[test]
    fn test_max_size_adapter() {
        let mut reader = MaxSizeAdapter::new(Cursor::new(b"hello".to_vec()), "test", 5);
        let mut buf = Vec::new();
        assert!(reader.read_to_end(&mut buf).is_ok());
        assert_eq!(buf, b"hello");

        let mut reader = MaxSizeAdapter::new(Cursor::new(b"hello".to_vec()), "test", 4);
        let mut buf = Vec::new();
        assert!(reader.read_to_end(&mut buf).is_err());
    }

    #[test]
    fn test_digest_adapter() {
        let mut reader = DigestAdapter::sha256(
            Cursor::new(b"hello".to_vec()),
            &hex!("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
            Url::parse("file:///").unwrap(),
        );
        let mut buf = Vec::new();
        assert!(reader.read_to_end(&mut buf).is_ok());
        assert_eq!(buf, b"hello");

        let mut reader = DigestAdapter::sha256(
            Cursor::new(b"hello".to_vec()),
            &hex!("0ebdc3317b75839f643387d783535adc360ca01f33c75f7c1e7373adcd675c0b"),
            Url::parse("file:///").unwrap(),
        );
        let mut buf = Vec::new();
        assert!(reader.read_to_end(&mut buf).is_err());
    }
}
