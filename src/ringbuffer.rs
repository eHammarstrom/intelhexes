use std::fs::File;
use std::io::Read;
use std::io::Cursor;
use std::io::Seek;
use std::io::BufRead;
use std::io::{Result, ErrorKind};
use std::cmp;
use std::mem;

#[derive(Debug)]
pub enum RBErrorKind {
    Empty,
    NotEnoughBytesAvail,
}

pub type RBResult<T> = std::result::Result<T, RBErrorKind>;

#[derive(Debug)]
pub struct RingBuffer<const SIZE: usize> {
    buf: [u8; SIZE],
    read_idx: usize,
    bytes_avail: usize,
}

impl<const SIZE: usize> RingBuffer<SIZE> {
    pub fn new() -> RingBuffer<SIZE> {
        RingBuffer {
            buf: [0u8; SIZE],
            read_idx: 0,
            bytes_avail: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        SIZE
    }

    pub fn len(&self) -> usize {
        self.bytes_avail
    }

    pub fn empty(&self) -> bool {
        self.bytes_avail == 0
    }

    /// Returns bytes consumed or an I/O error
    pub fn fill<R: Read>(&mut self, buf: &mut R) -> Result<usize> {
        let post_write_idx = self.read_idx + self.bytes_avail;
        let mut written = 0;

        // Push bytes to the back of the buffer
        if post_write_idx < SIZE {
            let can_read = SIZE - post_write_idx;
            written += buf.read(&mut self.buf[post_write_idx..post_write_idx + can_read])?;
            self.bytes_avail += written;
        }

        let write_idx = (self.read_idx + self.bytes_avail) % SIZE;
        if write_idx < self.read_idx {
            written += buf.read(&mut self.buf[write_idx..self.read_idx])?;
            self.bytes_avail += written;
        }

        Ok(written)
    }

    pub fn consume(&mut self, len: usize) -> RBResult<()> {
        if len > self.bytes_avail {
            Err(RBErrorKind::Empty)
        } else {
            self.bytes_avail -= len;
            self.read_idx = (self.read_idx + len) % SIZE;
            Ok(())
        }
    }

    pub fn peek(&self, len: usize) -> RBResult<&[u8]> {
        if len > SIZE - self.read_idx {
            Err(RBErrorKind::NotEnoughBytesAvail)
        } else {
            Ok(&self.buf[self.read_idx..self.read_idx + len])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ringbuffer::*;

    #[test]
    fn it_works() {
        let mut buf = std::io::Cursor::new(include_bytes!("../sniffer_nrf52840dk_nrf52840_7cc811f.hex"));
        let mut rb: RingBuffer<128> = RingBuffer::new();

        assert_eq!(rb.bytes_avail, 0);

        let bytes_read = rb.fill(&mut buf).expect("to be able to read the cursor");
        assert_eq!(bytes_read, 128);

        let ans = [58u8, 49, 48, 48, 48, 48, 48, 48, 48, 57];
        let peek = rb.peek(ans.len()).expect("to be able to peek after a fill");
        assert_eq!(peek, &ans);
        assert_eq!(rb.len(), 128);

        rb.consume(ans.len()).expect("to be able to consume the peeked bytes");

        assert!(!rb.empty());
        assert_eq!(rb.len(), 118);

        let bytes_read = rb.fill(&mut buf).expect("to be able to read the cursor");
        assert_eq!(bytes_read, 10);
        assert_eq!(rb.len(), 128);

        // Verify the fill by looking at the start of the internal buffer
        let ans = [48u8, 48, 48, 55, 51, 13, 10, 58, 49, 48];
        assert_eq!(&rb.buf[..ans.len()], &ans);

        let ans = [48u8, 69, 65, 48, 51, 50, 48, 48, 53, 51];
        let peek = rb.peek(ans.len()).expect("to be able to peek after a fill");
        assert_eq!(peek, &ans);

        // Fill should not fill anything now that we are full
        let bytes_read = rb.fill(&mut buf).expect("to be able to read the cursor");
        assert_eq!(bytes_read, 0);
        assert_eq!(rb.len(), 128);

        // We should have the same ans as previously
        let peek = rb.peek(ans.len()).expect("to be able to peek after a fill");
        assert_eq!(peek, &ans);


        // We should have the same start of the internal buffer as previously
        let ans = [48u8, 48, 48, 55, 51, 13, 10, 58, 49, 48];
        assert_eq!(&rb.buf[..ans.len()], &ans);
    }
}
