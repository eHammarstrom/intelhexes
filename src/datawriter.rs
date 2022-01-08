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

        writeln!(
            writer,
            "{:#010X}  {:<48}  |{:<16}|",
            addr,
            unsafe { std::str::from_utf8_unchecked(&hex_buf[..hex_len]) },
            unsafe { std::str::from_utf8_unchecked(&str_buf[..str_len]) }
        )?;

        Ok(())
    }
}

pub struct BinDataWriter {
    prev_addr: u16,
    fill_byte: u8,
}

impl BinDataWriter {
    pub fn new(fill_byte: u8) -> BinDataWriter {
        BinDataWriter {
            prev_addr: 0,
            fill_byte,
        }
    }
}

impl<W: Write> DataWriter<W> for BinDataWriter {
    fn write(&mut self, writer: &mut W, addr: i64, buf: &[u8]) -> Result<()> {
        Ok(())
    }
}
