pub type Crc24 = u32;

const CRC24_INIT: u32 = 0xB704CE;
const CRC24_POLY: u32 = 0x1864CFB; // NOTE: RFC 4880 also mentions 0x864CFB

/// Calculate the CRC-24 value from a given byte slice.
///
/// # Examples
/// ```rust
/// use ascii_armor::crc24;
/// assert_eq!(crc24::calculate(b"Hello World"), 0xBA2CC4);
/// ```
///
/// # Links
/// - [RFC 4880, Section 6.1](https://tools.ietf.org/html/rfc4880#section-6.1)
pub fn calculate(input: &[u8]) -> Crc24 {
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
    #[test]
    fn empty_data() {
        assert_eq!(super::calculate(b""), super::CRC24_INIT);
    }

    #[test]
    fn hello_world() {
        assert_eq!(super::calculate(b"Hello World"), 0xBA2CC4);
    }

    #[test]
    fn long_data() {
        let input = b"A".repeat(10_000);
        assert_eq!(super::calculate(&input), 0x3BE9A0);
    }
}
