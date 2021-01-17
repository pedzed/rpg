use crate::ArmorError;
use crate::crc24;
use crate::crc24::Crc24;

/// ASCII Armor checksum
///
/// Useful for generating and verifying checksums
#[derive(Debug, Clone, Copy)]
pub struct ArmorChecksum {
    crc24: Crc24,
}

impl ArmorChecksum {
    /// Create an instance of ArmorChecksum based on a pre-calculated checksum
    ///
    /// # Examples
    /// ```rust
    /// use ascii_armor::ArmorChecksum;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let checksum = ArmorChecksum::new("=uizE")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(checksum: &str) -> Result<Self, ArmorError> {
        let encoded = &checksum[1..];
        let decoded = match base64::decode(encoded.as_bytes()) {
            Ok(x) => x,
            Err(_) => return Err(ArmorError::InvalidChecksum(encoded.into())),
        };

        let crc24: Crc24 =
            (decoded[0] as Crc24) << 16 |
            (decoded[1] as Crc24) << 8 |
            (decoded[2] as Crc24) << 0
        ;

        Ok(Self {
            crc24,
        })
    }

    /// Create an instance of ArmorChecksum based on raw data
    ///
    /// # Examples
    /// ```rust
    /// use ascii_armor::ArmorChecksum;
    ///
    /// let checksum = ArmorChecksum::from_data(b"Hello World");
    /// ```
    pub fn from_data(raw_data: &[u8]) -> Self {
        let crc24 = crc24::calculate(&raw_data);

        Self {
            crc24,
        }
    }

    /// Get the checksum
    ///
    /// # Examples
    /// ```rust
    /// use ascii_armor::ArmorChecksum;
    ///
    /// let checksum = ArmorChecksum::from_data(b"Hello World");
    ///
    /// assert_eq!(checksum.get(), "=uizE");
    /// ```
    pub fn get(&self) -> String {
        let encoded = base64::encode(&[
            (self.crc24 >> 16) as u8,
            (self.crc24 >> 8) as u8,
            (self.crc24 >> 0) as u8,
        ]);

        let encoded = std::str::from_utf8(&encoded).unwrap();
        format!("={}", encoded)
    }

    /// Verify the checksum against raw data
    ///
    /// # Examples
    /// ```rust
    /// use ascii_armor::ArmorChecksum;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let checksum = ArmorChecksum::new("=uizE")?;
    ///
    /// assert!(checksum.verify(b"Hello World"));
    /// assert!(!checksum.verify(b"Hello, World"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify(&self, raw_data: &[u8]) -> bool {
        let verification_checksum = ArmorChecksum::from_data(raw_data);

        self.crc24 == verification_checksum.crc24
    }
}

#[cfg(test)]
mod tests {
    use super::ArmorChecksum;

    #[test]
    fn encoded_data_hello_world() {
        let checksum = ArmorChecksum::new("=uizE").unwrap();

        assert!(checksum.verify(b"Hello World"));
        assert!(!checksum.verify(b"Hello, World"));
    }

    #[test]
    fn encoded_data_binary_data() {
        let data = std::fs::read("tests/resources/gnupg-icon.png").unwrap();
        assert!(ArmorChecksum::new("=/u+x").unwrap().verify(&data));
        assert!(!ArmorChecksum::new("=ABCD").unwrap().verify(&data));
    }

    #[test]
    fn raw_data_empty() {
        let checksum = ArmorChecksum::from_data(b"");

        assert!(checksum.verify(b""));
        assert_eq!(checksum.get(), "=twTO");
    }

    #[test]
    fn raw_data_hello_world() {
        let checksum = ArmorChecksum::from_data(b"Hello World");

        assert!(checksum.verify(b"Hello World"));
        assert_eq!(checksum.get(), "=uizE");
    }

    #[test]
    fn raw_data_binary_data() {
        let data = std::fs::read("tests/resources/gnupg-icon.png").unwrap();
        let checksum = ArmorChecksum::from_data(&data);

        assert!(checksum.verify(&data));
        assert_eq!(checksum.get(), "=/u+x");
    }

    #[test]
    fn clone() {
        let checksum = ArmorChecksum::from_data(b"");
        let checksum2 = checksum.clone();

        assert_eq!(checksum.get(), checksum2.get());
    }
}
