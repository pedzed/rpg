use std::str;
use crate::radix64::tables;

/// The encoded output stream must be represented in lines of no more
/// than 76 characters each according to RFC 4880. GnuPG uses 64.
const LINE_LENGTH: usize = 64;
pub const LINE_ENDING: &str = "\r\n";

const BLOCKS_PER_OCTET: usize = 3;
const BLOCKS_PER_SEXTET: usize = 4;

pub const INVALID_VALUE: u8 = 255;

pub struct Radix64 {
    pub unencoded: Vec<u8>,
    pub encoded: String,
}

impl Radix64 {
    pub fn encode(unencoded: Vec<u8>) -> Self {
        let octets = &unencoded;
        let octets_remaining = octets.len() % BLOCKS_PER_OCTET;
        let octets_main_length = octets.len() - octets_remaining;

        let mut encoded_octets: Vec<u8> = vec![];

        for i in (0..octets_main_length).step_by(BLOCKS_PER_OCTET) {
            let chunk: u32 =
                ((octets[i] as u32) << 16) |
                ((octets[i + 1] as u32) << 8) |
                ((octets[i + 2] as u32) << 0)
            ;

            // Bit masks to extract 6-bit segments from the octet triplet chunk
            let a = (chunk & 16515072) >> 18;   // 16515072 = (2^6 - 1) << 18
            let b = (chunk & 258048) >> 12;     // 258048   = (2^6 - 1) << 12
            let c = (chunk & 4032) >> 6;        // 4032     = (2^6 - 1) << 6
            let d = (chunk & 63) >> 0;          // 63       = (2^6 - 1) << 0

            encoded_octets.push(tables::STD_ENCODE[a as usize]);
            encoded_octets.push(tables::STD_ENCODE[b as usize]);
            encoded_octets.push(tables::STD_ENCODE[c as usize]);
            encoded_octets.push(tables::STD_ENCODE[d as usize]);
        }

        if octets_remaining == 1 {
            let chunk = octets[octets_main_length];
            let a = (chunk & 252) >> 2;         // 252 = (2^6 - 1) << 2

            // Set the 4 least significant bits to zero
            let b = (chunk & 3) << 4;           // 3   = 2^2 - 1

            encoded_octets.push(tables::STD_ENCODE[a as usize]);
            encoded_octets.push(tables::STD_ENCODE[b as usize]);
            encoded_octets.push(b'=');
            encoded_octets.push(b'=');
        } else if octets_remaining == 2 {
            let chunk = (octets[octets_main_length] as u32) << 8 |
                (octets[octets_main_length + 1] as u32) << 0
            ;

            let a = (chunk & 64512) >> 10;      // 64512 = (2^6 - 1) << 10
            let b = (chunk & 1008) >> 4;        // 1008  = (2^6 - 1) << 4

            // Set the 2 least significant bits to zero
            let c = (chunk & 15) << 2; // 15    = 2^4 - 1

            encoded_octets.push(tables::STD_ENCODE[a as usize]);
            encoded_octets.push(tables::STD_ENCODE[b as usize]);
            encoded_octets.push(tables::STD_ENCODE[c as usize]);
            encoded_octets.push(b'=');
        }

        let mut encoded_lines = vec![];

        for sextet_chunk in encoded_octets.chunks(LINE_LENGTH) {
            encoded_lines.push(sextet_chunk);
        }

        let mut encoded_string: String;

        encoded_string = String::with_capacity(encoded_octets.len()); // TODO: Add new line count

        for l in encoded_lines {
            encoded_string.push_str(&format!(
                "{}{}",
                str::from_utf8(l).expect("Found invalid UTF-8."),
                LINE_ENDING
            ));
        }

        if encoded_string.ends_with(LINE_ENDING) {
            encoded_string.pop();
            encoded_string.pop();
        }

        Self {
            unencoded,
            encoded: encoded_string,
        }
    }

    pub fn decode(encoded: &str) -> Self {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_encode_1_char() {
        let radix64 = Radix64::encode(b"H".to_vec());
        assert_eq!(radix64.unencoded, b"H");
        assert_eq!(radix64.encoded, String::from("SA=="));
    }

    #[test]
    fn can_encode_2_chars() {
        let radix64 = Radix64::encode(b"He".to_vec());
        assert_eq!(radix64.unencoded, b"He");
        assert_eq!(radix64.encoded, String::from("SGU="));
    }

    #[test]
    fn can_encode_3_chars() {
        let radix64 = Radix64::encode(b"Hel".to_vec());
        assert_eq!(radix64.unencoded, b"Hel".to_vec());
        assert_eq!(radix64.encoded, String::from("SGVs"));
    }

    #[test]
    fn can_encode_4_chars() {
        let radix64 = Radix64::encode(b"Hell".to_vec());
        assert_eq!(radix64.unencoded, b"Hell".to_vec());
        assert_eq!(radix64.encoded, String::from("SGVsbA=="));
    }

    #[test]
    fn can_encode_5_chars() {
        let radix64 = Radix64::encode(b"Hello".to_vec());
        assert_eq!(radix64.unencoded, b"Hello".to_vec());
        assert_eq!(radix64.encoded, String::from("SGVsbG8="));
    }

    #[test]
    fn can_encode_6_chars() {
        let radix64 = Radix64::encode(b"Hello!".to_vec());
        assert_eq!(radix64.unencoded, b"Hello!".to_vec());
        assert_eq!(radix64.encoded, String::from("SGVsbG8h"));
    }

    #[test]
    fn can_encode_12_chars() {
        let radix64 = Radix64::encode(b"Hello World!".to_vec());
        assert_eq!(radix64.unencoded, b"Hello World!".to_vec());
        assert_eq!(radix64.encoded, String::from("SGVsbG8gV29ybGQh"));
    }

    #[test]
    fn can_encode_longer_text() {
        let unencoded = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_vec();

        let expected = "\
            TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2Np\r\n\
            bmcgZWxpdCwgc2VkIGRvIGVpdXNtb2QgdGVtcG9yIGluY2lkaWR1bnQgdXQgbGFi\r\n\
            b3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdWEuIFV0IGVuaW0gYWQgbWluaW0gdmVu\r\n\
            aWFtLCBxdWlzIG5vc3RydWQgZXhlcmNpdGF0aW9uIHVsbGFtY28gbGFib3JpcyBu\r\n\
            aXNpIHV0IGFsaXF1aXAgZXggZWEgY29tbW9kbyBjb25zZXF1YXQuIER1aXMgYXV0\r\n\
            ZSBpcnVyZSBkb2xvciBpbiByZXByZWhlbmRlcml0IGluIHZvbHVwdGF0ZSB2ZWxp\r\n\
            dCBlc3NlIGNpbGx1bSBkb2xvcmUgZXUgZnVnaWF0IG51bGxhIHBhcmlhdHVyLiBF\r\n\
            eGNlcHRldXIgc2ludCBvY2NhZWNhdCBjdXBpZGF0YXQgbm9uIHByb2lkZW50LCBz\r\n\
            dW50IGluIGN1bHBhIHF1aSBvZmZpY2lhIGRlc2VydW50IG1vbGxpdCBhbmltIGlk\r\n\
            IGVzdCBsYWJvcnVtLg=="
        ;

        let radix64 = Radix64::encode(unencoded);

        assert_eq!(radix64.encoded, expected);
    }
}
