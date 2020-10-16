use std::str;

use super::tables;
use super::armor::LINE_ENDING;

/// The encoded output stream must be represented in lines of no more
/// than 76 characters each according to RFC 4880. GnuPG uses 64.
const LINE_LENGTH: usize = 64;

const BLOCKS_PER_OCTET: usize = 3;
const BLOCKS_PER_SEXTET: usize = 4;

pub const INVALID_VALUE: u8 = 255;

#[derive(Debug)]
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
            // Source   Text (ASCII)    M              |a              |n
            //          Octets          77 (0x4d)      |97 (0x61)      |110 (0x6e)
            // Bits                     0 1 0 0 1 1¦0 1|0 1 1 0¦0 0 0 1|0 1¦1 0 1 1 1 0
            // Encoded  Sextets         19         |22         |5          |46
            //          Character       T          |W          |F          |u
            //          Octets          84 (0x54)  |87 (0x57)  |70 (0x46)  |117 (0x75)
            let unencoded_octet_chunk: u32 =
                ((octets[i + 0] as u32) << 16) |
                ((octets[i + 1] as u32) << 8) |
                ((octets[i + 2] as u32) << 0)
            ;

            // Bit masks to extract 6-bit segments from the triplet octet chunk
            let encoded_sextet_chunk: [u8; 4] = [
                ((unencoded_octet_chunk & 0b11111100_00000000_00000000) >> 18) as u8,
                ((unencoded_octet_chunk & 0b00000011_11110000_00000000) >> 12) as u8,
                ((unencoded_octet_chunk & 0b00000000_00001111_11000000) >> 6) as u8,
                ((unencoded_octet_chunk & 0b00000000_00000000_00111111) >> 0) as u8,
            ];

            for sextet in encoded_sextet_chunk.iter() {
                encoded_octets.push(tables::STD_ENCODE[*sextet as usize]);
            }
        }

        if octets_remaining == 2 {
            let unencoded_octet_chunk = (octets[octets_main_length] as u32) << 8 |
                (octets[octets_main_length + 1] as u32) << 0
            ;

            let encoded_sextet_chunk: [u8; 3] = [
                ((unencoded_octet_chunk & 0b11111100_00000000) >> 10) as u8,
                ((unencoded_octet_chunk & 0b00000011_11110000) >> 4) as u8,

                // Set the 2 least significant bits to zero
                ((unencoded_octet_chunk & 0b00000000_00001111) << 2) as u8,
            ];

            for sextet in encoded_sextet_chunk.iter() {
                encoded_octets.push(tables::STD_ENCODE[*sextet as usize]);
            }

            encoded_octets.push(b'=');
        } else if octets_remaining == 1 {
            let unencoded_octet_chunk = octets[octets_main_length];

            let encoded_sextet_chunk: [u8; 2] = [
                (unencoded_octet_chunk & 0b11111100) >> 2,

                // Set the 4 least significant bits to zero
                (unencoded_octet_chunk & 0b00000011) << 4,
            ];

            encoded_octets.push(tables::STD_ENCODE[encoded_sextet_chunk[0] as usize]);
            encoded_octets.push(tables::STD_ENCODE[encoded_sextet_chunk[1] as usize]);
            encoded_octets.push(b'=');
            encoded_octets.push(b'=');
        }

        let encoded_octet_lines = encoded_octets.chunks(LINE_LENGTH);

        let encoded_length = encoded_octets.len() +
            encoded_octet_lines.len() * LINE_ENDING.len()
        ;

        let mut encoded_string = String::with_capacity(encoded_length);

        for line in encoded_octets.chunks(LINE_LENGTH) {
            encoded_string.push_str(&format!(
                "{}{}",
                str::from_utf8(line).expect("Found invalid UTF-8."),
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
        let encoded_without_padding = encoded_without_padding(encoded);
        let encoded_normalized = encoded_without_padding.replace("\r\n", "");

        let encoded_chunks = encoded_normalized.as_bytes().chunks(BLOCKS_PER_SEXTET).collect::<Vec<_>>();
        let (last_chunk, main_chunks) = encoded_chunks.split_last().unwrap();

        let mut unencoded;

        if last_chunk.len() == BLOCKS_PER_SEXTET {
            unencoded = decode_full_chunks(&encoded_chunks);
        } else {
            let mut unencoded_partial = decode_partial_chunk(last_chunk);

            unencoded = decode_full_chunks(main_chunks);
            unencoded.append(&mut unencoded_partial);
        }

        Self {
            unencoded,
            encoded: String::from(encoded),
        }
    }
}

fn encoded_without_padding(encoded: &str) -> &str {
    let padding_count = encoded.as_bytes()
        .iter()
        .rev()
        .take_while(|&b| *b == b'=')
        .count()
    ;

    match padding_count {
        0 => &encoded[0..encoded.len()-0],
        1 => &encoded[0..encoded.len()-1],
        2 => &encoded[0..encoded.len()-2],
        _ => unreachable!("Padding count not possible."),
    }
}

fn decode_full_chunks(full_chunks: &[&[u8]]) -> Vec<u8> {
    let mut unencoded: Vec<u8> = vec![];

    for encoded_chunk in full_chunks {
        let mut decoded_sextets: [u8; BLOCKS_PER_SEXTET] = [INVALID_VALUE; BLOCKS_PER_SEXTET];

        for (i, encoded_char) in encoded_chunk.iter().enumerate() {
            decoded_sextets[i] = tables::STD_DECODE[*encoded_char as usize];

            if decoded_sextets[i] == INVALID_VALUE {
                // TODO: Replace with catchable error
                panic!("Unexpected value to decode.");
            }
        }

        let decoded_chunk: u32 =
            ((decoded_sextets[0] as u32) << 18) |
            ((decoded_sextets[1] as u32) << 12) |
            ((decoded_sextets[2] as u32) << 6) |
            ((decoded_sextets[3] as u32) << 0)
        ;

        let decoded_octet_chunk: [u8; BLOCKS_PER_OCTET] = [
            ((decoded_chunk & 0b11111111_00000000_00000000) >> 16) as u8,
            ((decoded_chunk & 0b00000000_11111111_00000000) >> 8) as u8,
            ((decoded_chunk & 0b00000000_00000000_11111111) >> 0) as u8,
        ];

        for &octet in decoded_octet_chunk.iter() {
            if octet == INVALID_VALUE {
                // TODO: Replace with catchable error
                panic!("Unexpected value `{}` to decode.", octet);
            }

            unencoded.push(octet);
        }
    }

    unencoded
}

fn decode_partial_chunk(partial_chunk: &[u8]) -> Vec<u8> {
    let mut unencoded: Vec<u8> = vec![];

    match partial_chunk.len() {
        2 => {
            let mut decoded_sextets: [u8; 2] = [INVALID_VALUE; 2];

            for (i, encoded_char) in partial_chunk.iter().enumerate() {
                decoded_sextets[i] = tables::STD_DECODE[*encoded_char as usize];

                if decoded_sextets[i] == INVALID_VALUE {
                    // TODO: Replace with catchable error
                    panic!("Unexpected value to decode.");
                }
            }

            let decoded_octet: u8 = (decoded_sextets[0] << 2) | (decoded_sextets[1] >> 4);

            unencoded.push(decoded_octet);
        },
        3 => {
            let mut decoded_sextets: [u8; 3] = [INVALID_VALUE; 3];

            for (i, encoded_char) in partial_chunk.iter().enumerate() {
                decoded_sextets[i] = tables::STD_DECODE[*encoded_char as usize];

                if decoded_sextets[i] == INVALID_VALUE {
                    // TODO: Replace with catchable error
                    panic!("Unexpected value to decode.");
                }
            }

            let decoded_octet_chunk: [u8; 2] = [
                ((decoded_sextets[0] << 2) | (decoded_sextets[1] >> 4)) as u8,
                ((decoded_sextets[1] << 4) | (decoded_sextets[2] >> 2)) as u8,
            ];

            for octet in decoded_octet_chunk.iter() {
                unencoded.push(*octet);
            }
        },
        x => unreachable!("Partial chunk size of {} is not possible.", x),
    }

    unencoded
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
        let unencoded = b"\
            Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
            eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut \
            enim ad minim veniam, quis nostrud exercitation ullamco laboris \
            nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in \
            reprehenderit in voluptate velit esse cillum dolore eu fugiat \
            nulla pariatur. Excepteur sint occaecat cupidatat non proident, \
            sunt in culpa qui officia deserunt mollit anim id est laborum.\
        ".to_vec();

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

    #[test]
    fn can_get_encoding_without_padding_0() {
        assert_eq!(encoded_without_padding("SGVs"), "SGVs");
    }

    #[test]
    fn can_get_encoding_without_padding_1() {
        assert_eq!(encoded_without_padding("SGU="), "SGU");
    }

    #[test]
    fn can_get_encoding_without_padding_2() {
        assert_eq!(encoded_without_padding("SA=="), "SA");
    }

    #[test]
    fn can_decode_2_chars() {
        let radix64 = Radix64::decode("SA==");
        assert_eq!(radix64.encoded, String::from("SA=="));
        assert_eq!(radix64.unencoded, b"H".to_vec());
    }

    #[test]
    fn can_decode_3_chars() {
        let radix64 = Radix64::decode("SGU=");
        assert_eq!(radix64.encoded, String::from("SGU="));
        assert_eq!(radix64.unencoded, b"He".to_vec());
    }

    #[test]
    fn can_decode_4_chars() {
        let radix64 = Radix64::decode("SGVs");
        assert_eq!(radix64.encoded, String::from("SGVs"));
        assert_eq!(radix64.unencoded, b"Hel".to_vec());
    }

    #[test]
    fn can_decode_6_chars() {
        let radix64 = Radix64::decode("SGVsbA==");
        assert_eq!(radix64.encoded, String::from("SGVsbA=="));
        assert_eq!(radix64.unencoded, b"Hell".to_vec());
    }

    #[test]
    fn can_decode_7_chars() {
        let radix64 = Radix64::decode("SGVsbG8=");
        assert_eq!(radix64.encoded, String::from("SGVsbG8="));
        assert_eq!(radix64.unencoded, b"Hello".to_vec());
    }

    #[test]
    fn can_decode_8_chars() {
        let radix64 = Radix64::decode("SGVsbG8h");
        assert_eq!(radix64.encoded, String::from("SGVsbG8h"));
        assert_eq!(radix64.unencoded, b"Hello!".to_vec());
    }

    #[test]
    fn can_decode_16_chars() {
        let radix64 = Radix64::decode("SGVsbG8gV29ybGQh");
        assert_eq!(radix64.encoded, String::from("SGVsbG8gV29ybGQh"));
        assert_eq!(radix64.unencoded, b"Hello World!".to_vec());
    }

    #[test]
    fn can_decode_longer_text() {
        let encoded = "\
            TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2Np\r\n\
            bmcgZWxpdCwgc2VkIGRvIGVpdXNtb2QgdGVtcG9yIGluY2lkaWR1bnQgdXQgbGFi\r\n\
            b3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdWEuIFV0IGVuaW0gYWQgbWluaW0gdmVu\r\n\
            aWFtLCBxdWlzIG5vc3RydWQgZXhlcmNpdGF0aW9uIHVsbGFtY28gbGFib3JpcyBu\r\n\
            aXNpIHV0IGFsaXF1aXAgZXggZWEgY29tbW9kbyBjb25zZXF1YXQuIER1aXMgYXV0\r\n\
            ZSBpcnVyZSBkb2xvciBpbiByZXByZWhlbmRlcml0IGluIHZvbHVwdGF0ZSB2ZWxp\r\n\
            dCBlc3NlIGNpbGx1bSBkb2xvcmUgZXUgZnVnaWF0IG51bGxhIHBhcmlhdHVyLiBF\r\n\
            eGNlcHRldXIgc2ludCBvY2NhZWNhdCBjdXBpZGF0YXQgbm9uIHByb2lkZW50LCBz\r\n\
            dW50IGluIGN1bHBhIHF1aSBvZmZpY2lhIGRlc2VydW50IG1vbGxpdCBhbmltIGlk\r\n\
            IGVzdCBsYWJvcnVtLg==\
        ";
        let expected = b"\
            Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
            eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut \
            enim ad minim veniam, quis nostrud exercitation ullamco laboris \
            nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in \
            reprehenderit in voluptate velit esse cillum dolore eu fugiat \
            nulla pariatur. Excepteur sint occaecat cupidatat non proident, \
            sunt in culpa qui officia deserunt mollit anim id est laborum.\
        ";

        let radix64 = Radix64::decode(encoded);

        assert_eq!(radix64.unencoded.len(), expected.len());
        assert_eq!(radix64.unencoded[0..32], expected[0..32]);

        let len = expected.len();
        assert_eq!(
            radix64.unencoded[len-32..len],
            expected[len-32..len]
        );
    }
}
