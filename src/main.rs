use intelhexes::print_human;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: intelhexes FILE");
        std::process::exit(1);
    }

    let f = match std::fs::File::open(&args[1]) {
        Ok(f) => f,
        Err(e) => std::process::exit(e.raw_os_error().unwrap_or(1)),
    };

    let exit_code = match print_human(f, std::io::stdout()) {
        Ok(_) => 0,
        Err(e) => e.raw_os_error().unwrap_or(1),
    };

    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works_nrf() {
        let hex = include_bytes!("../hex-examples/sniffer_nrf52840dk_nrf52840_7cc811f.hex");
        assert!(print_human(std::io::Cursor::new(hex), std::io::stdout()).is_ok());
    }

    #[test]
    fn it_works_nina() {
        let hex = include_bytes!("../hex-examples/NINA-W15X-SW-4.0.0-006.hex");
        assert!(print_human(std::io::Cursor::new(hex), std::io::stdout()).is_ok());
    }
}
