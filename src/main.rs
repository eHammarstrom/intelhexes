use std::fs::File;
use std::io::Read;
use std::io::Cursor;
use std::io::Seek;
use std::io::BufRead;
use std::cmp;

#[derive(Debug)]
enum RecordType {
    Data = 0x00,
    EndOfFile = 0x01,
    ExtendedLinearAddr = 0x04,
    StartLinearAddr = 0x05,
}

#[derive(Debug)]
struct DataRecord<'s> {
    len: u8,
    addr: u16,
    data: &'s [u8],
}

#[derive(Debug)]
struct EndOfFileRecord { }

#[derive(Debug)]
struct ExtendedLinearAddrRecord {
    addr: u32,
}

#[derive(Debug)]
struct StartLinearAddrRecord {
    addr: u32,
}

fn print_human<B: BufRead>(mut buf: B) {
    let print_buf = [0u8; 4096];
    let written = 0;

    loop {

    }
}

fn main() {
    let hex = include_bytes!("../sniffer_nrf52840dk_nrf52840_7cc811f.hex");
    print_human(Cursor::new(hex));
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        include_bytes!("../sniffer_nrf52840dk_nrf52840_7cc811f.hex");
    }
}
