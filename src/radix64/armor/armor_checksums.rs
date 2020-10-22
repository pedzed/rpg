use std::str;

use super::super::crc24::Crc24;

#[derive(Debug)]
pub struct ArmorChecksum {
    crc24: Crc24,
}

impl ArmorChecksum {
    pub fn new(checksum: &str) -> Self {
        let crc24 = Crc24::from_encoded(&checksum[1..].as_bytes());

        Self {
            crc24,
        }
    }

    pub fn from_payload(raw_data: &[u8]) -> Self {
        let crc24 = Crc24::from_payload(raw_data);

        Self {
            crc24,
        }
    }

    pub fn get(&self) -> String {
        let encoded_str = str::from_utf8(&self.crc24.encoded).unwrap(); // TODO: Proper error handling
        format!("={}", encoded_str)
    }

    pub fn verify(&self, raw_data: &[u8]) -> bool {
        let verification_crc = Crc24::from_payload(raw_data);

        self.crc24.code == verification_crc.code
    }
}

#[cfg(test)]
mod tests {
    use super::ArmorChecksum;

    #[test]
    fn get_for_hello_world() {
        let checksum = ArmorChecksum::from_payload(b"Hello World");

        assert_eq!(checksum.get(), "=uizE");
    }

    #[test]
    fn get_for_empty_string() {
        let checksum = ArmorChecksum::from_payload(b"");

        assert_eq!(checksum.get(), "=twTO");
    }

    #[test]
    fn verify_valid_for_hello_world() {
        let checksum = ArmorChecksum::new("=uizE");

        assert!(checksum.verify(b"Hello World"));
    }

    #[test]
    fn verify_invalid_for_hello_world() {
        let checksum = ArmorChecksum::new("=uizE");

        assert!(!checksum.verify(b"World, hello."));
    }
}
