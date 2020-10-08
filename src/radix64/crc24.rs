use super::coding::Radix64;

const CRC24_INIT: u32 = 0xB704CE;
const CRC24_POLY: u32 = 0x864CFB;

pub struct Crc24 {
    pub octets: u32,
    pub encoded: String,
}

impl Crc24 {
    pub fn new(input: &[u8]) -> Crc24 {
        let octets = Crc24::calculate_octets(input);

        Crc24 {
            octets,
            encoded: Radix64::encode(vec![
                (octets >> 16) as u8,
                (octets >> 8) as u8,
                (octets >> 0) as u8,
            ]).encoded,
        }
    }

    fn calculate_octets(input: &[u8]) -> u32 {
        let mut crc = CRC24_INIT;

        for octet in input.iter() {
            crc ^= (*octet as u32) << 16;

            for _ in 0..8 {
                crc <<= 1;

                if crc & 0x1000000 != 0 {
                    crc ^= CRC24_POLY;
                }
            }
        }

        crc & 0xFFFFFF
    }

    pub fn radix64_checksum(self) -> String {
        format!("={}", self.encoded)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_octets_for_hello_world() {
        let crc24 = Crc24::new(b"Hello World");

        assert_eq!(crc24.octets, 12201156);
        assert_eq!(crc24.radix64_checksum(), "=uizE");
    }

    #[test]
    fn crc_octets_for_empty_string() {
        let crc24 = Crc24::new(b"");

        assert_eq!(crc24.octets, 11994318);
        assert_eq!(crc24.radix64_checksum(), "=twTO");
    }

    #[test]
    fn crc_octets_for_long_string() {
        let input = b"A".repeat(2000);
        let crc24 = Crc24::new(&input);

        assert_eq!(crc24.octets, 11175483);
        assert_eq!(crc24.radix64_checksum(), "=qoY7");
    }
}
