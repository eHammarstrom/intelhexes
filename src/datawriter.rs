use std::io::{Result, Write};

use crate::helpers::*;

pub trait DataWriter<W: Write> {
    fn write(&mut self, writer: &mut W, addr: i64, buf: &[u8]) -> Result<()>;
}

pub struct HexDataWriter {
    addr_offset: i64,
    prev_addr: i64,
    prev_bytes_written: i64,
}

impl HexDataWriter {
    pub fn new() -> HexDataWriter {
        HexDataWriter {
            addr_offset: 0,
            prev_addr: 0,
            prev_bytes_written: 0,
        }
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

macro_rules! write_formatted_hex {
    ( $hex_buf:ident, $hex_len:ident, $start:ident, $end:ident, $buf:ident ) => {
        for i in $start..$end {
            $hex_buf[$hex_len] = $buf[i];
            $hex_len += 1;

            if i < 31 {
                if (i + 1) % 16 == 0 {
                    $hex_buf[$hex_len] = b' ';
                    $hex_len += 1;
                }
                if (i + 1) % 2 == 0 {
                    $hex_buf[$hex_len] = b' ';
                    $hex_len += 1;
                }
            }
        }
    }
}

impl<W: Write> DataWriter<W> for HexDataWriter {
    fn write(&mut self, writer: &mut W, addr: i64, buf: &[u8]) -> Result<()> {
        let mut bytes_written = 0;
        let mut hex_buf = [0u8; 64];
        let mut hex_len = 0;
        let mut str_buf = [0u8; 16];
        let mut str_len = 0;

        let mut addr = addr + self.addr_offset;

        println!(
            "{:#010X} {}",
            addr,
            unsafe{std::str::from_utf8_unchecked(buf)}
        );


        // Firstly we handle the following,
        // 1. Partial start of data row
        // 2. Address space gap
        if addr % 16 != 0 {
            // Partial start of data row

            let missing_bytes = (addr as usize) % 16;
            let missing_hex_chars = missing_bytes * 2;
            for i in 0..missing_hex_chars {
                hex_buf[hex_len] = b'-';
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
            for i in 0..missing_bytes {
                str_buf[i] = b' ';
                str_len += 1;
            }

            bytes_written = missing_hex_chars;
            let offset = -(missing_bytes as i64);
            self.addr_offset = self.addr_offset + offset;
            addr = addr + offset;
        } else if addr != 0 && addr != self.prev_addr + 16 {
            // Address space gap

            const EMPTY_HEXES: &'static str = "-- -- -- -- -- -- -- --  -- -- -- -- -- -- -- --";
            for _ in 0..((addr - self.prev_addr) / 16) {
                writeln!(writer, "{:#010X}  {:<48}  |{:<16}|", addr, EMPTY_HEXES, "");
            }
        }

        // Secondly we handle the actual data in the data row
        let end = buf.len();
        // write_formatted_hex!(hex_buf, hex_len, bytes_written, end, buf);
        for i in bytes_written..end {
            hex_buf[hex_len] = buf[i];
            hex_len += 1;
            bytes_written += 1;

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
        for bs in buf[bytes_written..].chunks(2) {
            str_buf[str_len] = hex_to_u8(&bs[..2]);
            if str_buf[str_len] >= 127 || str_buf[str_len] <= 31 {
                str_buf[str_len] = b'.';
            }
            str_len += 1;
        }

        // Lastly we handle the partial end of a data row
        if buf.len() < 32 {
            println!("PARTIAL END");
        }

        HexDataWriter::write_row(writer, addr, &hex_buf, hex_len, &str_buf, str_len)?;

        self.prev_addr = addr;
        println!("set self.prev_addr = {}", self.prev_addr);
        self.prev_bytes_written = bytes_written as i64;

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
