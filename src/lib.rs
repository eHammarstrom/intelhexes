#![allow(clippy::needless_range_loop)]

use std::io::{Error, ErrorKind, Result};
use std::io::{Read, Write, BufWriter};

pub mod ringbuffer;

const COLON: usize = 1;

const HDR_LEN: usize = COLON;
const HDR_LEN_SZ: usize = 2;
const HDR_ADDR: usize = HDR_LEN + HDR_LEN_SZ;
const HDR_ADDR_SZ: usize = 4;
const HDR_TYPE: usize = HDR_ADDR + HDR_ADDR_SZ;
const HDR_TYPE_SZ: usize = 2;

const RECORD_HEADER_SZ: usize = COLON + HDR_LEN_SZ + HDR_ADDR_SZ + HDR_TYPE_SZ;
const CHECKSUM_SZ: usize = 2;

const EXT_LINEAR_ADDR_UPPER_ADDR: usize = COLON + 8;
const EXT_LINEAR_ADDR_UPPER_ADDR_SZ: usize = 4;
const EXT_SEGMENT_ADDR_UPPER_ADDR: usize = COLON + 8;
const EXT_SEGMENT_ADDR_UPPER_ADDR_SZ: usize = 4;
const DATA_START: usize = RECORD_HEADER_SZ;

#[derive(Debug, PartialEq, Eq)]
enum RecordType {
    Data,
    EndOfFile,
    ExtendedSegmentAddr,
    StartSegmentAddr,
    ExtendedLinearAddr,
    StartLinearAddr,

    Unknown,
}

impl From<u8> for RecordType {
    fn from(b: u8) -> RecordType {
        match b {
            0x00 => RecordType::Data,
            0x01 => RecordType::EndOfFile,
            0x02 => RecordType::ExtendedSegmentAddr,
            0x03 => RecordType::StartSegmentAddr,
            0x04 => RecordType::ExtendedLinearAddr,
            0x05 => RecordType::StartLinearAddr,
            _ => RecordType::Unknown,
        }
    }
}

impl RecordType {
    fn fixed_size(self) -> usize {
        match self {
            RecordType::EndOfFile => COLON + 10,
            RecordType::ExtendedSegmentAddr => COLON + 14,
            RecordType::StartSegmentAddr => COLON + 18,
            RecordType::ExtendedLinearAddr => COLON + 14,
            RecordType::StartLinearAddr => COLON + 18,
            _ => 0,
        }
    }
}

#[inline]
fn hex_to_u8(bytes: &[u8]) -> u8 {
    let s = unsafe { std::str::from_utf8_unchecked(bytes) };
    u8::from_str_radix(s, 16).unwrap()
}

#[inline]
fn hex_to_u16(bytes: &[u8]) -> u16 {
    let s = unsafe { std::str::from_utf8_unchecked(bytes) };
    u16::from_str_radix(s, 16).unwrap()
}

#[inline]
fn maybe_fetch<R: Read, const SZ: usize>(
    rb: &mut ringbuffer::RingBuffer<SZ>,
    reader: &mut R,
    need: usize,
) -> Result<()> {
    if rb.len() >= need {
        return Ok(());
    }

    rb.fill(reader)?;

    if rb.len() < need {
        Err(Error::new(
            ErrorKind::Other,
            "Expected more bytes to be available",
        ))
    } else {
        Ok(())
    }
}

pub fn hex2dump<R: Read, W: Write>(mut reader: R, writer: W) -> Result<()> {
    const BUF_SZ: usize = 4096;

    let mut rb: ringbuffer::RingBuffer<BUF_SZ> = ringbuffer::RingBuffer::new();
    let mut addr_offset: u32 = 0;

    let mut writer = BufWriter::new(writer);

    rb.fill(&mut reader)?;

    loop {
        maybe_fetch(&mut rb, &mut reader, RECORD_HEADER_SZ)?;

        let buf = rb.wrapping_peek(RECORD_HEADER_SZ).unwrap();

        let record_type = RecordType::from(hex_to_u8(&buf[HDR_TYPE..HDR_TYPE + HDR_TYPE_SZ]));

        match record_type {
            RecordType::Unknown => {
                eprintln!(
                    "Found unknown record type: {:#02x}",
                    hex_to_u8(&buf[HDR_TYPE..HDR_TYPE + HDR_TYPE_SZ])
                );
                eprintln!("Surrounding bytes: {:?}", unsafe {
                    std::str::from_utf8_unchecked(&buf[0..RECORD_HEADER_SZ])
                });
                return Err(Error::new(ErrorKind::Other, "Unknown record type"));
            }
            RecordType::Data => {
                let data_len = 2 * hex_to_u8(&buf[HDR_LEN..HDR_LEN + HDR_LEN_SZ]) as usize;
                let sz = CHECKSUM_SZ + RECORD_HEADER_SZ + data_len;

                maybe_fetch(&mut rb, &mut reader, sz)?;

                let buf = rb.wrapping_peek(sz).unwrap();

                let addr = hex_to_u16(&buf[HDR_ADDR..HDR_ADDR + HDR_ADDR_SZ]);

                let mut hex_buf = [0u8; 64];
                let mut hex_len = 0;
                for i in DATA_START..(sz - CHECKSUM_SZ) {
                    let abs_index = i - DATA_START + 1;

                    hex_buf[hex_len] = buf[i];
                    hex_len += 1;

                    if abs_index < 32 {
                        if abs_index % 16 == 0 {
                            hex_buf[hex_len] = b' ';
                            hex_len += 1;
                        }
                        if abs_index % 2 == 0 {
                            hex_buf[hex_len] = b' ';
                            hex_len += 1;
                        }
                    }
                }

                let mut str_buf = [0u8; 16];
                let mut str_len = 0;
                for (i, bs) in buf[DATA_START..(sz - CHECKSUM_SZ)].chunks(2).enumerate() {
                    str_buf[i] = hex_to_u8(&bs[..2]);
                    if str_buf[i] >= 127 || str_buf[i] <= 31 {
                        str_buf[i] = 46;
                    }
                    str_len += 1;
                }

                writeln!(
                    writer,
                    "{:#010x}  {:<48} |{:<16}|",
                    addr_offset + addr as u32,
                    unsafe { std::str::from_utf8_unchecked(&hex_buf[..hex_len]) },
                    unsafe { std::str::from_utf8_unchecked(&str_buf[..str_len]) }
                )?;

                rb.consume(sz).unwrap();
            }
            // This record affects the following data addresses
            RecordType::ExtendedLinearAddr => {
                let sz = RecordType::ExtendedLinearAddr.fixed_size();

                maybe_fetch(&mut rb, &mut reader, sz)?;

                let buf = rb.wrapping_peek(sz).unwrap();

                addr_offset = hex_to_u16(
                    &buf[EXT_LINEAR_ADDR_UPPER_ADDR
                        ..EXT_LINEAR_ADDR_UPPER_ADDR + EXT_LINEAR_ADDR_UPPER_ADDR_SZ],
                ) as u32;

                rb.consume(sz).unwrap();
            }
            // This record affects the following data addresses
            RecordType::ExtendedSegmentAddr => {
                let sz = RecordType::ExtendedSegmentAddr.fixed_size();

                maybe_fetch(&mut rb, &mut reader, sz)?;

                let buf = rb.wrapping_peek(sz).unwrap();

                let segment_addr = hex_to_u16(
                    &buf[EXT_SEGMENT_ADDR_UPPER_ADDR
                        ..EXT_SEGMENT_ADDR_UPPER_ADDR + EXT_SEGMENT_ADDR_UPPER_ADDR_SZ],
                );
                addr_offset = segment_addr as u32 * 10;

                rb.consume(sz).unwrap();
            }
            // Skip records that do not affect output
            rt @ (RecordType::StartLinearAddr | RecordType::StartSegmentAddr) => {
                let sz = rt.fixed_size();

                maybe_fetch(&mut rb, &mut reader, sz)?;

                rb.consume(sz).unwrap();
            }
            RecordType::EndOfFile => {
                writer.flush()?;
                break;
            }
        }

        loop {
            maybe_fetch(&mut rb, &mut reader, 1)?;

            match rb.peek(1).unwrap()[0] as char {
                '\r' => {
                    rb.consume(1).unwrap();
                }
                '\n' => {
                    rb.consume(1).unwrap();
                }
                _ => break,
            };
        }
    }

    Ok(())
}
