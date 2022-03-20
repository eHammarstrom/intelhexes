use std::io::{Result, Write};

use crate::helpers::*;

pub trait DataWriter<W: Write> {
    fn write(&mut self, writer: &mut W, addr: i64, buf: &[u8]) -> Result<()>;
}

pub struct HexDataWriter {}

impl HexDataWriter {
    pub fn new() -> HexDataWriter {
        HexDataWriter {}
    }

    fn write_row<W: Write>(
        mut writer: W,
        addr: i64,
        hex_buf: &[u8],
        hex_len: usize,
        str_buf: &[u8],
        str_len: usize,
    ) -> Result<()> {
        writeln!(
            writer,
            "{:#010X}  {:<48}  |{:<16}|",
            addr,
            unsafe { std::str::from_utf8_unchecked(&hex_buf[..hex_len]) },
            unsafe { std::str::from_utf8_unchecked(&str_buf[..str_len]) }
        )
    }
}

impl<W: Write> DataWriter<W> for HexDataWriter {
    fn write(&mut self, writer: &mut W, addr: i64, buf: &[u8]) -> Result<()> {
        let mut hex_buf = [0u8; 64];
        let mut hex_len = 0;

        for i in 0..buf.len() {
            hex_buf[hex_len] = buf[i];
            hex_len += 1;

            if i < 31 {
                if (i + 1) % 16 == 0 {
                    hex_buf[hex_len] = b' ';
                    hex_len += 1;
                }
                if (i + 1) % 2 == 0 {
                    hex_buf[hex_len] = b' ';
                    hex_len += 1;
                }
            }
        }

        let mut str_buf = [0u8; 16];
        let mut str_len = 0;
        for (i, bs) in buf.chunks(2).enumerate() {
            str_buf[i] = hex_to_u8(&bs[..2]);
            if str_buf[i] >= 127 || str_buf[i] <= 31 {
                str_buf[i] = b'.';
            }
            str_len += 1;
        }

        HexDataWriter::write_row(writer, addr, &hex_buf, hex_len, &str_buf, str_len)?;

        Ok(())
    }
}

pub struct BinDataWriter {
    prev_addr: i64,
    prev_bytes_written: i64,
    fill_byte: u8,
}

impl BinDataWriter {
    pub fn new(fill_byte: u8) -> BinDataWriter {
        BinDataWriter {
            prev_addr: 0,
            prev_bytes_written: 0,
            fill_byte,
        }
    }
}

impl<W: Write> DataWriter<W> for BinDataWriter {
    fn write(&mut self, writer: &mut W, addr: i64, buf: &[u8]) -> Result<()> {
        let mut byte_buf = [0u8; 16];
        let mut byte_buf_len = 0;
        let fill_bytes_to_write = addr - self.prev_addr - self.prev_bytes_written;

        if addr < self.prev_addr {
            eprintln!("Expected increasing address order; found {:#010x} followed by {:#010x}",
                self.prev_addr, addr);
            return Err(std::io::ErrorKind::Unsupported.into());
        }

        // Only fill between addresses, not from 0 up to start address
        if self.prev_addr != 0 {
            for _ in 0..fill_bytes_to_write {
                writer.write(&[ self.fill_byte ])?;
            }
        }

        for (i, bs) in buf.chunks(2).enumerate() {
            byte_buf[i] = hex_to_u8(&bs[..2]);
            byte_buf_len += 1;
        }

        writer.write(&byte_buf[..byte_buf_len])?;

        self.prev_addr = addr;
        self.prev_bytes_written = byte_buf_len as i64;

        Ok(())
    }
}
