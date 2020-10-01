type Crc24 = u32;

const CRC24_INIT: Crc24 = 0xB704CE;
const CRC24_POLY: Crc24 = 0x864CFB;

pub fn crc_octets(input: &[u8]) -> Crc24 {
    let mut crc = CRC24_INIT;

    for octet in input.iter() {
        crc ^= (*octet as Crc24) << 16;

        for _ in 0..8 {
            crc <<= 1;

            if crc & 0x1000000 != 0 {
                crc ^= CRC24_POLY;
            }
        }
    }

    crc & 0xFFFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_octets_for_hello_world() {
        let input = "Hello World";
        assert_eq!(crc_octets(input.as_bytes()), 12201156);
    }

    #[test]
    fn crc_octets_for_empty_string() {
        let input = "";
        assert_eq!(crc_octets(input.as_bytes()), 11994318);
    }

    #[test]
    fn crc_octets_for_long_string() {
        let input = "A".repeat(2000);
        println!("{}", input);
        assert_eq!(crc_octets(input.as_bytes()), 11175483);
    }
}
