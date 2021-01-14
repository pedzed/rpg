type Crc24Code = u32;

const CRC24_INIT: Crc24Code = 0xB704CE;
const CRC24_POLY: Crc24Code = 0x864CFB;

#[derive(Debug)]
pub struct Crc24 {
    pub code: Crc24Code,
    pub encoded: Vec<u8>,
}

impl Crc24 {
    pub fn from_encoded(encoded: &[u8]) -> Self { // TODO: Unit test
        let decoded = base64::decode(encoded).unwrap(); // TODO: Proper error handling

        let code: Crc24Code =
            (decoded[0] as Crc24Code) << 16 |
            (decoded[1] as Crc24Code) << 8 |
            (decoded[2] as Crc24Code) << 0
        ;

        Self {
            code,
            encoded: encoded.to_vec(),
        }
    }

    pub fn from_payload(input: &[u8]) -> Self {
        let code = Self::calculate_code(input);

        let encoded = base64::encode(&vec![
            (code >> 16) as u8,
            (code >> 8) as u8,
            (code >> 0) as u8,
        ]);

        Self {
            code,
            encoded,
        }
    }

    fn calculate_code(input: &[u8]) -> Crc24Code {
        let mut crc = CRC24_INIT;

        for octet in input.iter() {
            crc ^= (*octet as Crc24Code) << 16;

            for _ in 0..8 {
                crc <<= 1;

                if crc & 0x1000000 != 0 {
                    crc ^= CRC24_POLY;
                }
            }
        }

        crc & 0xFFFFFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world() {
        let crc24 = Crc24::from_payload(b"Hello World");

        assert_eq!(crc24.code, 12201156);
    }

    #[test]
    fn empty_string() {
        let crc24 = Crc24::from_payload(b"");

        assert_eq!(crc24.code, 11994318);
    }

    #[test]
    fn long_string() {
        let input = b"A".repeat(2000);
        let crc24 = Crc24::from_payload(&input);

        assert_eq!(crc24.code, 11175483);
    }
}
