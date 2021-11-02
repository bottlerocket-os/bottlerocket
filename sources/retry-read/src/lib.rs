//! This library provides a `RetryRead` trait with a `retry_read` function that's available for any
//! `Read` type.  `retry_read` retries after standard interruptions (unlike `read`) but also
//! returns the number of bytes read (unlike `read_exact`), and without needing to read to the end
//! of the input (unlike `read_to_end` and `read_to_string`).

use std::io::{ErrorKind, Read, Result};

/// Provides a way to retry standard read operations while also returning the number of bytes read.
pub trait RetryRead<R> {
    fn retry_read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

impl<R: Read> RetryRead<R> for R {
    // This implementation is based on stdlib Read::read_exact, but hitting EOF isn't a failure, we
    // just want to return the number of bytes we could read.
    /// Like `Read::read` but retries on ErrorKind::Interrupted, returning the number of bytes read.
    fn retry_read(&mut self, mut buf: &mut [u8]) -> Result<usize> {
        let mut count = 0;

        // Read until we have no more space in the output buffer
        while !buf.is_empty() {
            match self.read(buf) {
                // No bytes left, done
                Ok(0) => break,
                // Read n bytes, slide ahead n in the output buffer and read more
                Ok(n) => {
                    count += n;
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                // Retry on interrupt
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                // Other failures are fatal
                Err(e) => return Err(e),
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod test {
    use super::{ErrorKind, Read, Result, RetryRead};
    use std::io::{Error, Write};

    // Helper method for simple test cases, confirming we read the full given slice.
    fn test(data: &[u8]) {
        let mut output = vec![0; data.len()];
        let count = (&data[..]).retry_read(&mut output).unwrap();
        assert_eq!(count, data.len());
        assert_eq!(&data[..], &output);
    }

    #[test]
    fn zero_read() {
        test(&[]);
    }

    #[test]
    fn small_read() {
        test(&[0, 1, 2, 3, 42]);
    }

    #[test]
    fn large_read() {
        test(&[42; 9999]);
    }

    // Confirm we retry reads when interrupted.
    #[test]
    fn retried_read() {
        let mut reader = InterruptedReader::new(5);
        let mut output = vec![0; 5];
        let count = reader.retry_read(&mut output).unwrap();
        assert_eq!(count, 5);
        assert_eq!(output, vec![42, 42, 42, 42, 42]);
    }

    // Helper that implements Read, eventually returning the requested number of bytes, but returns
    // ErrorKind::Interrupted every other call.
    struct InterruptedReader {
        requested_reads: u64,
        finished_reads: u64,
        interrupt: bool,
    }

    impl InterruptedReader {
        fn new(requested_reads: u64) -> Self {
            Self {
                requested_reads,
                finished_reads: 0,
                interrupt: false,
            }
        }
    }

    impl Read for InterruptedReader {
        fn read(&mut self, mut buf: &mut [u8]) -> Result<usize> {
            if self.finished_reads > self.requested_reads {
                return Ok(0);
            }

            if self.interrupt {
                self.interrupt = false;
                Err(Error::new(ErrorKind::Interrupted, "you asked for it"))
            } else {
                self.interrupt = true;
                self.finished_reads += 1;
                buf.write(&[42])
            }
        }
    }
}
