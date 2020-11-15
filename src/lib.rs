pub mod aes;
pub mod radix64;

// use radix64::crc24::Crc24;

pub fn run_crc(_input: &str) {
    // let crc = Crc24::from_payload(input.as_bytes());
    println!("CRC-24 called.");
    // println!("Input:  `{}`\noutput: `{}`", input, crc.encoded);
}
