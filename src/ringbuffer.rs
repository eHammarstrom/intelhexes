use std::cmp;
use std::io::Read;
use std::io::Result;

#[derive(Debug)]
pub enum RBErrorKind {
    Empty,
    PeekReachedWrap,
    NotEnoughBytesAvail,
}

pub type RBResult<T> = std::result::Result<T, RBErrorKind>;

#[derive(Debug)]
pub struct RingBuffer<const SIZE: usize> {
    buf: [u8; SIZE],
    peek_buffer: [u8; SIZE],
    read_idx: usize,
    bytes_avail: usize,
}

impl<const SIZE: usize> Default for RingBuffer<SIZE> {
    fn default() -> RingBuffer<SIZE> {
        RingBuffer {
            buf: [0u8; SIZE],
            peek_buffer: [0u8; SIZE],
            read_idx: 0,
            bytes_avail: 0,
        }
    }
}

impl<const SIZE: usize> RingBuffer<SIZE> {
    pub fn new() -> RingBuffer<SIZE> {
        RingBuffer::default()
    }

    pub fn capacity(&self) -> usize {
        SIZE
    }

    pub fn len(&self) -> usize {
        self.bytes_avail
    }

    pub fn is_empty(&self) -> bool {
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
            assert!(self.bytes_avail <= SIZE);
        }

        let write_idx = (self.read_idx + self.bytes_avail) % SIZE;
        if write_idx < self.read_idx {
            let write = buf.read(&mut self.buf[write_idx..self.read_idx])?;
            self.bytes_avail += write;
            written += write;
            assert!(self.bytes_avail <= SIZE);
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

    /// Copyless peek may reach end of buffer, the "wrap"
    pub fn peek(&self, len: usize) -> RBResult<&[u8]> {
        if len > SIZE - self.read_idx {
            Err(RBErrorKind::PeekReachedWrap)
        } else {
            Ok(&self.buf[self.read_idx..self.read_idx + len])
        }
    }

    /// 2 copies will occur if this peek reaches of the buffer "wrap".
    /// One copy of the bytes present at the end of the buffer, and one
    /// of the bytes present at the start of the buffer.
    pub fn wrapping_peek(&mut self, len: usize) -> RBResult<&[u8]> {
        if len > self.bytes_avail {
            return Err(RBErrorKind::NotEnoughBytesAvail);
        }

        let end_bytes = cmp::min(SIZE - self.read_idx, len);
        let start_bytes = self.bytes_avail - end_bytes;

        self.peek_buffer[..end_bytes]
            .copy_from_slice(&self.buf[self.read_idx..self.read_idx + end_bytes]);
        self.peek_buffer[end_bytes..end_bytes + start_bytes]
            .copy_from_slice(&self.buf[..start_bytes]);

        Ok(&self.peek_buffer[..len])
    }
}

#[cfg(test)]
mod tests {
    use crate::ringbuffer::*;

    #[test]
    fn it_works() {
        let mut buf = std::io::Cursor::new(include_bytes!(
            "../hex-examples/sniffer_nrf52840dk_nrf52840_7cc811f.hex"
        ));
        let mut rb: RingBuffer<128> = RingBuffer::new();

        assert_eq!(rb.bytes_avail, 0);

        let bytes_read = rb.fill(&mut buf).expect("to be able to read the cursor");
        assert_eq!(bytes_read, 128);

        let ans = [58u8, 49, 48, 48, 48, 48, 48, 48, 48, 57];
        let peek = rb.peek(ans.len()).expect("to be able to peek after a fill");
        assert_eq!(peek, &ans);
        assert_eq!(rb.len(), 128);

        rb.consume(ans.len())
            .expect("to be able to consume the peeked bytes");

        assert!(!rb.is_empty());
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

        // Currently we have consumed 10 bytes, so read_idx = 10, let's put it
        // at 118 so we have 10 bytes in the rb right-of read_idx
        rb.consume(108)
            .expect("to be able to consume 108 bytes after 10 bytes");
        assert_eq!(rb.read_idx, 118);

        let last_10_bytes = &rb.buf[118..];
        assert_eq!(last_10_bytes.len(), 10);
        let first_10_bytes = &rb.buf[..10];
        assert_eq!(first_10_bytes.len(), 10);
        let mut linear_bytes = Vec::new();
        linear_bytes.extend_from_slice(last_10_bytes);
        linear_bytes.extend_from_slice(first_10_bytes);
        // We need to be able to provide a peek wrapping the buffer
        let peek = rb
            .wrapping_peek(linear_bytes.len())
            .expect("to be able to peek across buffer wrap");
        assert_eq!(peek, &linear_bytes[..]);
    }
}
