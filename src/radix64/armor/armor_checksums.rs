use super::super::crc24::Crc24;

pub struct ArmorChecksum {
    crc24: Crc24,
}

impl ArmorChecksum {
    pub fn new(raw_data: &[u8]) -> Self {
        let crc24 = Crc24::new(raw_data);

        Self {
            crc24,
        }
    }

    pub fn get(&self) -> String {
        format!("={}", self.crc24.encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::ArmorChecksum;

    #[test]
    fn get_for_hello_world() {
        let checksum = ArmorChecksum::new(b"Hello World");

        assert_eq!(checksum.get(), "=uizE");
    }

    #[test]
    fn get_for_empty_string() {
        let checksum = ArmorChecksum::new(b"");

        assert_eq!(checksum.get(), "=twTO");
    }
}
