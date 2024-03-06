//! This module supports reading from an input source that could be compressed or plain text.
//!
//! Currently gzip compression is supported.

use flate2::read::GzDecoder;
use retry_read::RetryRead;
use std::fs::File;
use std::io::{BufReader, Chain, Cursor, Read, Result, Take};
use std::path::Path;

/// "File magic" that indicates file type is stored in a few bytes at the start at the start of the
/// data.  For now we only need two bytes for gzip, but if adding new formats, we'd need to read
/// more.  (The simplest approach may be to read the max length for any format we need and compare
/// the appropriate prefix length.)
/// https://en.wikipedia.org/wiki/List_of_file_signatures
const MAGIC_LEN: usize = 2;

// We currently only support gzip, but it shouldn't be hard to add more.
/// These bytes are at the start of any gzip-compressed data.
const GZ_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// This helper takes a slice of bytes representing UTF-8 text, which can optionally be
/// compressed, and returns an uncompressed string.
#[allow(dead_code)]
pub fn expand_slice_maybe(input: &[u8]) -> Result<String> {
    let mut output = String::new();
    let mut reader = OptionalCompressionReader::new(Cursor::new(input));
    reader.read_to_string(&mut output)?;
    Ok(output)
}

/// This helper takes the path to a file containing UTF-8 text, which can optionally be compressed,
/// and returns an uncompressed string of all its contents.  File reads are done through BufReader.
pub fn expand_file_maybe<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let file = File::open(path)?;
    let mut output = String::new();
    let mut reader = OptionalCompressionReader::new(BufReader::new(file));
    reader.read_to_string(&mut output)?;
    Ok(output)
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// This type lets you wrap a `Read` whose data may or may not be compressed, and its `read()`
/// calls will uncompress the data if needed.
pub struct OptionalCompressionReader<R>(CompressionType<R>);

/// This represents the type of compression we've detected within a `Read`, or `Unknown` if we
/// haven't yet read any bytes to be able to detect it.
enum CompressionType<R> {
    /// This represents the starting state of the reader before we've read the magic bytes and
    /// detected any compression.
    ///
    /// We need ownership of the `Read` to construct one of the variants below, so we use an
    /// `Option` to allow `take`ing the value out, even if we only have a &mut reference in the
    /// `read` implementation.  This is safe because detection is a one-time process and we know we
    /// construct this with Some value.
    Unknown(Option<R>),

    /// We haven't found recognizable compression.
    None(Peek<R>),

    /// We found gzip compression.
    Gz(Box<GzDecoder<Peek<R>>>),
}

/// `Peek` lets us read the starting bytes (the "magic") of an input `Read` but maintain those
/// bytes in an internal buffer.  We Take the number of bytes we read (to handle reads shorter than
/// MAGIC_LEN) and Chain them together with the rest of the input, to represent the full input.
type Peek<T> = Chain<Take<Cursor<[u8; MAGIC_LEN]>>, T>;

impl<R: Read> OptionalCompressionReader<R> {
    /// Build a new `OptionalCompressionReader` before we know the input compression type.
    pub fn new(input: R) -> Self {
        Self(CompressionType::Unknown(Some(input)))
    }
}

/// Implement `Read` by checking whether we've detected compression type yet, and if not, detecting
/// it and then replacing ourselves with the appropriate type so we can continue reading.
impl<R: Read> Read for OptionalCompressionReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.0 {
            CompressionType::Unknown(ref mut input) => {
                // Take ownership of our `Read` object so we can store it in a new variant.
                let mut reader = input.take().expect(
                    "OptionalCompressionReader constructed with None input; programming error",
                );

                // Read the "magic" that tells us the compression type.
                let mut magic = [0u8; MAGIC_LEN];
                let count = reader.retry_read(&mut magic)?;

                // We need to return all of the bytes, but we just consumed MAGIC_LEN of them.
                // This chains together those initial bytes with the remainder so we have them all.
                let magic_read = Cursor::new(magic).take(count as u64);
                let full_input = magic_read.chain(reader);

                // Detect compression type based on the magic bytes.
                if count == MAGIC_LEN && magic == GZ_MAGIC {
                    // Use a gzip decoder if gzip compressed.
                    self.0 = CompressionType::Gz(Box::new(GzDecoder::new(full_input)))
                } else {
                    // We couldn't detect any compression; just read the input.
                    self.0 = CompressionType::None(full_input)
                }

                // We've replaced Unknown with a known compression type; defer to that for reading.
                self.read(buf)
            }

            // After initial detection, we just perform standard reads on the reader we prepared.
            CompressionType::None(ref mut r) => r.read(buf),
            CompressionType::Gz(ref mut r) => r.read(buf),
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use std::io::Cursor;

    lazy_static! {
        /// Some plain text strings and their gzip encodings.
        static ref DATA: &'static [(&'static str, &'static [u8])] = &[
            ("", &hex!("1f8b 0808 3863 3960 0003 656d 7074 7900 0300 0000 0000 0000 0000")),
            ("4", &hex!("1f8b 0808 6f63 3960 0003 666f 7572 0033 0100 381b b6f3 0100 0000")),
            ("42", &hex!("1f8b 0808 7c6b 3960 0003 616e 7377 6572 0033 3102 0088 b024 3202 0000 00")),
            ("hi there", &hex!("1f8b 0808 d24f 3960 0003 6869 7468 6572 6500 cbc8 5428 c948 2d4a 0500 ec76 a3e3 0800 0000")),
        ];
    }

    #[test]
    fn test_plain() {
        for (plain, _gz) in *DATA {
            let input = Cursor::new(plain);
            let mut output = String::new();
            OptionalCompressionReader::new(input)
                .read_to_string(&mut output)
                .unwrap();
            assert_eq!(output, *plain);
        }
    }

    #[test]
    fn test_gz() {
        for (plain, gz) in *DATA {
            let input = Cursor::new(gz);
            let mut output = String::new();
            OptionalCompressionReader::new(input)
                .read_to_string(&mut output)
                .unwrap();
            assert_eq!(output, *plain);
        }
    }

    #[test]
    fn test_helper_plain() {
        for (plain, _gz) in *DATA {
            assert_eq!(expand_slice_maybe(plain.as_bytes()).unwrap(), *plain);
        }
    }

    #[test]
    fn test_helper_gz() {
        for (plain, gz) in *DATA {
            assert_eq!(expand_slice_maybe(gz).unwrap(), *plain);
        }
    }

    #[test]
    fn test_magic_prefix() {
        // Confirm that if we give a prefix of valid magic, but not the whole thing, we just get
        // that input back.
        let input = Cursor::new(&[0x1f]);
        let mut output = Vec::new();
        let count = OptionalCompressionReader::new(input)
            .read_to_end(&mut output)
            .unwrap();
        assert_eq!(count, 1);
        assert_eq!(output, &[0x1f]);
    }
}
