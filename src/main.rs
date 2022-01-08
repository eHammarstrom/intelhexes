use intelhexes::{hex2dump, hex2bin};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::result;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Produce a human-readable dump from the intel HEX input file
    #[structopt(long)]
    hex2dump: bool,

    /// Produce a binary from the intel HEX input file
    #[structopt(long)]
    hex2bin: bool,

    /// Byte used to fill empty address space when producing a binary
    #[structopt(long)]
    fill_byte: Option<u8>,

    /// Output file, stdout if unspecified
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Input file
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let output_file: Box<dyn io::Write> = opt
        .output
        .map(fs::File::open)
        .map(result::Result::ok)
        .flatten()
        .map(|f| Box::new(f) as Box<dyn io::Write>)
        .unwrap_or(Box::new(io::stdout()));

    let input_file = fs::File::open(opt.file).expect("Invalid input file path");

    let exit_code = if opt.hex2dump {
        match hex2dump(input_file, output_file) {
            Ok(_) => 0,
            Err(e) => e.raw_os_error().unwrap_or(1),
        }
    } else if opt.hex2bin {
        let fill_byte = opt.fill_byte.unwrap_or(0);
        match hex2bin(input_file, output_file, fill_byte) {
            Ok(_) => 0,
            Err(e) => e.raw_os_error().unwrap_or(1),
        }
    } else {
        println!("No operations specified, bye!");
        0
    };

    std::process::exit(exit_code);
}
