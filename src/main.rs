use std::env;
use rpg::crc::crc24;

fn main() {
    let args: Vec<String> = env::args().collect();

    for (i, arg) in args.iter().enumerate() {
        if arg == "--crc" {
            match env::args().nth(i + 1) {
                Some(input) => run_crc(&input),
                None => panic!("No input argument given for CRC."),
            }
        }
    }
}

fn run_crc(input: &str) {
    let output = crc24::crc_octets(input.as_bytes());
    println!("CRC-24 called.");
    println!("Input:  `{}`\noutput: `{}`", input, output);
}
