pub mod crc;
pub mod radix64;

use crc::crc24;

pub fn run_crc(input: &str) {
    let output = crc24::crc_octets(input.as_bytes());
    println!("CRC-24 called.");
    println!("Input:  `{}`\noutput: `{}`", input, output);
}
