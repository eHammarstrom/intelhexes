#![allow(clippy::needless_range_loop)]

mod datawriter;
mod ringbuffer;
mod helpers;

use std::cmp;
use std::io::{BufWriter, Read, Write};
use std::io::{Error, ErrorKind, Result};

use datawriter::*;
use helpers::*;

const COLON: usize = 1;

// header offsets
const HDR_LEN: usize = COLON;
const HDR_LEN_SZ: usize = 2;
const HDR_ADDR: usize = HDR_LEN + HDR_LEN_SZ;
const HDR_ADDR_SZ: usize = 4;
const HDR_TYPE: usize = HDR_ADDR + HDR_ADDR_SZ;
const HDR_TYPE_SZ: usize = 2;

const RECORD_HEADER_SZ: usize = COLON + HDR_LEN_SZ + HDR_ADDR_SZ + HDR_TYPE_SZ;
const CHECKSUM_SZ: usize = 2;

const DATA_ROW_HEX_SZ: usize = 32;

// data row offsets
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

#[derive(Default, Clone, Copy)]
struct DataRow {
    addr: u16,
    data: [u8; 32],
    len: u8,
}

impl DataRow {
    fn new(addr: u16, bytes: &[u8]) -> DataRow {
        let mut d = DataRow {
            addr,
            data: [0u8; 32],
            len: bytes.len() as u8,
        };
        d.data[..d.len as usize].copy_from_slice(bytes);
        d
    }
}

struct DataRowCache<'a, W: Write, DWR: DataWriter<W>, const LINES: usize> {
    cache: [DataRow; LINES],
    read_idx: usize,
    write_idx: usize,
    data_writer: &'a mut DWR,
    _data_writer_writer: std::marker::PhantomData<W>,
}

impl<'a, W: Write, DWR: DataWriter<W>, const LINES: usize> DataRowCache<'a, W, DWR, LINES> {
    fn new(data_writer: &mut DWR) -> DataRowCache<W, DWR, LINES>
    {
        DataRowCache {
            cache: [DataRow::default(); LINES],
            read_idx: 0,
            write_idx: 0,
            data_writer,
            _data_writer_writer: std::marker::PhantomData,
        }
    }

    fn push(&mut self, addr: u16, bytes: &[u8]) {
        self.cache[self.write_idx] = DataRow::new(addr, bytes);
        self.write_idx = (self.write_idx + 1) % LINES;
    }

    fn push_front(&mut self, addr: u16, bytes: &[u8]) {
        self.read_idx = (self.read_idx.wrapping_sub(1)) % LINES;
        self.cache[self.read_idx] = DataRow::new(addr, bytes);
    }

    fn peek(&self) -> &DataRow {
        return &self.cache[self.read_idx];
    }

    fn pop(&mut self) -> DataRow {
        let d = self.cache[self.read_idx];
        self.read_idx = (self.read_idx + 1) % LINES;
        d
    }

    fn available(&self) -> usize {
        let mut bytes = 0;

        for i in 0..self.len() {
            let idx = (self.read_idx + i) % LINES;
            bytes += self.cache[idx].len as usize;
        }

        bytes
    }

    fn len(&self) -> usize {
        if self.read_idx > self.write_idx {
            LINES - self.read_idx + self.write_idx
        } else {
            self.write_idx - self.read_idx
        }
    }

    fn build_and_print_row(&mut self, writer: &mut W, addr_offset: i64) -> Result<usize> {
        let avail = self.available();

        /* Not possible, return available bytes */
        if avail < DATA_ROW_HEX_SZ {
            return Ok(avail);
        }

        let mut buf: [u8; DATA_ROW_HEX_SZ] = [0u8; DATA_ROW_HEX_SZ];
        let mut len = 0;
        let print_addr = self.peek().addr;

        while len < DATA_ROW_HEX_SZ as u8 {
            let d = self.pop();
            let to_write = cmp::min(d.len as usize, DATA_ROW_HEX_SZ - len as usize);

            buf[len as usize..len as usize + to_write].copy_from_slice(&d.data[..to_write]);
            len += to_write as u8;

            if to_write as u8 != d.len {
                let num_pushback_bytes = (d.data[to_write..].len() / 2) as u16;
                self.push_front(d.addr + num_pushback_bytes, &d.data[to_write..]);
            }
        }

        /* Row address correction should not be applied here since this was an aligned
         * data row that was just missing some data
         */
        self.data_writer.write(writer, addr_offset + print_addr as i64, &buf[..])?;

        Ok(self.available())
    }

    fn dump_cache(&mut self, writer: &mut W, addr_offset: i64) -> Result<()> {
        /* Build a full row if possible */
        self.build_and_print_row(writer, addr_offset)?;
        /* Dump remaining rows. Logically it should only be 1 */
        while self.read_idx != self.write_idx {
            let row = self.cache[self.read_idx];
            self.read_idx = (1 + self.read_idx) % LINES;
            self.data_writer.write(
                writer,
                addr_offset + row.addr as i64,
                &row.data[..row.len as usize],
            )?;
        }

        Ok(())
    }
}

fn process<R: Read, W: Write, DWR: DataWriter<BufWriter<W>>>(
    mut reader: R,
    writer: W,
    data_writer: &mut DWR,
) -> Result<()> {
    const BUF_SZ: usize = 4096;

    let mut rb: ringbuffer::RingBuffer<BUF_SZ> = ringbuffer::RingBuffer::new();
    let mut addr_offset: i64 = 0;

    let mut writer = BufWriter::new(writer);

    rb.fill(&mut reader)?;

    let mut data_cache: DataRowCache<_, _, 8> = DataRowCache::new(data_writer);
    let mut row_addr_correction: i64 = 0;

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

                /* Build and dump a full row if possible */
                let avail = data_cache.build_and_print_row(&mut writer, addr_offset)?;

                /* If the cache is empty then we have no more row correction to do */
                if avail == 0 {
                    row_addr_correction = 0;
                }

                if avail > 0 && avail < DATA_ROW_HEX_SZ {
                    /* We have partial data in the cache that needs more data
                     * to complete a full line
                     */
                    data_cache.push(addr, &buf[DATA_START..sz - CHECKSUM_SZ]);
                } else if data_len % 32 != 0 {
                    data_cache.push(addr, &buf[DATA_START..sz - CHECKSUM_SZ]);
                    /* Adjust addresses of following data lines since we now
                     * adjusted the data contents of one line from partial
                     * to full
                     */
                    row_addr_correction -= (data_len / 2) as i64;
                } else {
                    data_cache.push((addr_offset + row_addr_correction + addr as i64) as u16, &buf[DATA_START..sz - CHECKSUM_SZ]);
                }

                rb.consume(sz).unwrap();
            }
            // This record affects the following data addresses
            RecordType::ExtendedLinearAddr => {
                let sz = RecordType::ExtendedLinearAddr.fixed_size();

                maybe_fetch(&mut rb, &mut reader, sz)?;

                let buf = rb.wrapping_peek(sz).unwrap();

                /* Dump cache before changing section */
                data_cache.dump_cache(&mut writer, addr_offset + row_addr_correction)?;

                row_addr_correction = 0;
                addr_offset = (hex_to_u16(
                    &buf[EXT_LINEAR_ADDR_UPPER_ADDR
                        ..EXT_LINEAR_ADDR_UPPER_ADDR + EXT_LINEAR_ADDR_UPPER_ADDR_SZ],
                ) as i64)
                    << 16;

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

                /* Dump cache before changing section */
                data_cache.dump_cache(&mut writer, addr_offset + row_addr_correction)?;

                row_addr_correction = 0;
                addr_offset = (segment_addr as i64) << 4;

                rb.consume(sz).unwrap();
            }
            // Skip records that do not affect output
            rt @ (RecordType::StartLinearAddr | RecordType::StartSegmentAddr) => {
                let sz = rt.fixed_size();

                maybe_fetch(&mut rb, &mut reader, sz)?;

                rb.consume(sz).unwrap();
            }
            RecordType::EndOfFile => {
                data_cache.dump_cache(&mut writer, addr_offset)?;
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

pub fn hex2dump<R: Read, W: Write>(reader: R, writer: W) -> Result<()> {
    let mut hex_writer = HexDataWriter::new();
    process(reader, writer, &mut hex_writer)
}

pub fn hex2bin<R: Read, W: Write>(reader: R, writer: W, fill_byte: u8) -> Result<()> {
    let mut hex_writer = BinDataWriter::new(fill_byte);
    process(reader, writer, &mut hex_writer)
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::io::Result;

    fn is_equal(output: File, truth: File) -> Result<()> {
        let output_lines = BufReader::new(output).lines();
        let truth_lines = BufReader::new(truth).lines();

        for (o_line, t_line) in output_lines.zip(truth_lines) {
            if o_line.is_err() && t_line.is_err() {
                break;
            }

            assert!(o_line.is_ok());
            assert!(t_line.is_ok());

            let o_line = o_line.unwrap();
            let t_line = t_line.unwrap();

            assert_eq!(o_line, t_line);
        }

        Ok(())
    }

    fn load_test(p: &'static str, test_type: &'static str) -> Result<(File, File, File)> {
        let reader_path = format!("test/{}.in", p);
        let writer_path = format!("test/{}/{}.out", test_type, p);
        let truth_path = format!("test/{}/{}.truth", test_type, p);

        let reader = File::open(reader_path)?;
        let writer = File::create(writer_path)?;
        let truth = File::open(truth_path)?;

        Ok((reader, writer, truth))
    }

    fn run_test(test: &'static str, folder: &'static str, f: fn(File, &mut File) -> Result<()>)
    {
        let (reader, mut writer, truth) = load_test(test, folder).expect("to find test files");

        assert!(f(reader, &mut writer).is_ok());

        let output = File::open(format!("test/{}/{}.out", folder, test)).unwrap();

        let output_metadata = output.metadata().unwrap();
        let truth_metadata = truth.metadata().unwrap();

        assert_eq!(output_metadata.len(), truth_metadata.len());

        is_equal(output, truth).expect("comparison to succeed");
    }

    fn run_hex_test(test: &'static str) {
        run_test(test, "hex", |infile, outfile| { hex2dump(infile, outfile) });
    }

    fn run_bin_test(test: &'static str) {
        run_test(test, "bin", |infile, outfile| { hex2bin(infile, outfile, 0xff) });
    }

    #[test]
    fn it_works_on_partial_lines() {
        run_hex_test("partial_line");
    }

    #[test]
    fn it_equals_py_hex2dump_output_nrf() {
        run_hex_test("sniffer_nrf52840dk_nrf52840_7cc811f");
    }

    #[test]
    fn it_equals_py_hex2dump_output_nina() {
        run_hex_test("NINA-W15X-SW-4.0.0-006");
    }

    #[test]
    fn it_equals_py_hex2dump_output_addrspace_gap_start() {
        run_hex_test("addrspace-gap-start");
    }

    #[test]
    fn it_equals_py_hex2dump_output_addrspace_gap_mid() {
        run_hex_test("addrspace-gap-mid");
    }

    #[test]
    fn it_equals_py_hex2dump_output_addrspace_gap_mid_partial() {
        run_hex_test("addrspace-gap-mid-partial");
    }

    #[test]
    fn it_equals_py_hex2bin_output_nrf() {
        run_bin_test("sniffer_nrf52840dk_nrf52840_7cc811f");
    }

    #[test]
    fn it_equals_py_hex2bin_output_nina() {
        run_bin_test("NINA-W15X-SW-4.0.0-006");
    }

    #[test]
    fn it_equals_py_hex2bin_output_addrspace_gap_start() {
        run_bin_test("addrspace-gap-start");
    }

    #[test]
    fn it_equals_py_hex2bin_output_addrspace_gap_mid() {
        run_bin_test("addrspace-gap-mid");
    }

    #[test]
    fn it_equals_py_hex2bin_output_addrspace_gap_mid_partial() {
        run_bin_test("addrspace-gap-mid-partial");
    }
}
