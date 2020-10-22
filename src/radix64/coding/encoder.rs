use super::super::coding::BLOCKS_PER_OCTET;
use super::super::coding::BLOCKS_PER_SEXTET;
use super::super::tables;

#[derive(Debug)]
pub struct Radix64Encoder;

impl Radix64Encoder {
    pub fn encode(input: &[u8]) -> Vec<u8> {
        let octets_remaining = input.len() % BLOCKS_PER_OCTET;
        let octets_main_length = input.len() - octets_remaining;

        let mut output: Vec<u8> = vec![];

        for i in (0..octets_main_length).step_by(BLOCKS_PER_OCTET) {
            // Source   Text (ASCII)    M              |a              |n
            //          Octets          77 (0x4d)      |97 (0x61)      |110 (0x6e)
            // Bits                     0 1 0 0 1 1¦0 1|0 1 1 0¦0 0 0 1|0 1¦1 0 1 1 1 0
            // Encoded  Sextets         19         |22         |5          |46
            //          Character       T          |W          |F          |u
            //          Octets          84 (0x54)  |87 (0x57)  |70 (0x46)  |117 (0x75)
            let octets_joined: u32 =
                ((input[i + 0] as u32) << 16) |
                ((input[i + 1] as u32) << 8) |
                ((input[i + 2] as u32) << 0)
            ;

            // Bit masks to extract 6-bit segments from the triplet octet chunk
            let sextets: [u8; BLOCKS_PER_SEXTET] = [
                ((octets_joined & 0b11111100_00000000_00000000) >> 18) as u8,
                ((octets_joined & 0b00000011_11110000_00000000) >> 12) as u8,
                ((octets_joined & 0b00000000_00001111_11000000) >> 6) as u8,
                ((octets_joined & 0b00000000_00000000_00111111) >> 0) as u8,
            ];

            for &sextet in sextets.iter() {
                output.push(tables::STD_ENCODE[sextet as usize]);
            }
        }

        if octets_remaining == 2 {
            let octets_joined =
                (input[octets_main_length + 0] as u32) << 8 |
                (input[octets_main_length + 1] as u32) << 0
            ;

            let sextets: [u8; 3] = [
                ((octets_joined & 0b11111100_00000000) >> 10) as u8,
                ((octets_joined & 0b00000011_11110000) >> 4) as u8,

                // Set the 2 least significant bits to zero
                ((octets_joined & 0b00000000_00001111) << 2) as u8,
            ];

            for sextet in sextets.iter() {
                output.push(tables::STD_ENCODE[*sextet as usize]);
            }

            output.push(b'=');
        } else if octets_remaining == 1 {
            let octets_joined = input[octets_main_length];

            let sextets: [u8; 2] = [
                (octets_joined & 0b11111100) >> 2,

                // Set the 4 least significant bits to zero
                (octets_joined & 0b00000011) << 4,
            ];

            output.push(tables::STD_ENCODE[sextets[0] as usize]);
            output.push(tables::STD_ENCODE[sextets[1] as usize]);
            output.push(b'=');
            output.push(b'=');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

    #[test]
    fn encode_1_char() {
        let encoded = Radix64Encoder::encode(b"H");

        assert_eq!(encoded, b"SA==");
    }

    #[test]
    fn encode_2_chars() {
        let encoded = Radix64Encoder::encode(b"He");

        assert_eq!(encoded, b"SGU=");
    }

    #[test]
    fn encode_3_chars() {
        let encoded = Radix64Encoder::encode(b"Hel");

        assert_eq!(encoded, b"SGVs");
    }

    #[test]
    fn encode_4_chars() {
        let encoded = Radix64Encoder::encode(b"Hell");

        assert_eq!(encoded, b"SGVsbA==");
    }

    #[test]
    fn encode_5_chars() {
        let encoded = Radix64Encoder::encode(b"Hello");

        assert_eq!(encoded, b"SGVsbG8=");
    }

    #[test]
    fn encode_6_chars() {
        let encoded = Radix64Encoder::encode(b"Hello!");

        assert_eq!(encoded, b"SGVsbG8h");
    }

    #[test]
    fn encode_12_chars() {
        let encoded = Radix64Encoder::encode(b"Hello World!");

        assert_eq!(encoded, b"SGVsbG8gV29ybGQh");
    }

    #[test]
    fn encode_longer_text() {
        let unencoded = b"\
            Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
            eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut \
            enim ad minim veniam, quis nostrud exercitation ullamco laboris \
            nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in \
            reprehenderit in voluptate velit esse cillum dolore eu fugiat \
            nulla pariatur. Excepteur sint occaecat cupidatat non proident, \
            sunt in culpa qui officia deserunt mollit anim id est laborum.\
        ";

        let expected = b"\
            TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2Np\
            bmcgZWxpdCwgc2VkIGRvIGVpdXNtb2QgdGVtcG9yIGluY2lkaWR1bnQgdXQgbGFi\
            b3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdWEuIFV0IGVuaW0gYWQgbWluaW0gdmVu\
            aWFtLCBxdWlzIG5vc3RydWQgZXhlcmNpdGF0aW9uIHVsbGFtY28gbGFib3JpcyBu\
            aXNpIHV0IGFsaXF1aXAgZXggZWEgY29tbW9kbyBjb25zZXF1YXQuIER1aXMgYXV0\
            ZSBpcnVyZSBkb2xvciBpbiByZXByZWhlbmRlcml0IGluIHZvbHVwdGF0ZSB2ZWxp\
            dCBlc3NlIGNpbGx1bSBkb2xvcmUgZXUgZnVnaWF0IG51bGxhIHBhcmlhdHVyLiBF\
            eGNlcHRldXIgc2ludCBvY2NhZWNhdCBjdXBpZGF0YXQgbm9uIHByb2lkZW50LCBz\
            dW50IGluIGN1bHBhIHF1aSBvZmZpY2lhIGRlc2VydW50IG1vbGxpdCBhbmltIGlk\
            IGVzdCBsYWJvcnVtLg=="
        ;

        let encoded = Radix64Encoder::encode(unencoded);

        assert_eq!(encoded.to_vec(), expected.to_vec());
    }

    #[test]
    fn encode_binary_file() {
        let file_contents = fs::read("tests/resources/gnupg-icon.png").unwrap();

        let expected = b"\
            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\
            sU0Ob/4BrGxXIweWt2UAAAAASUVORK5CYII=\
        ";

        let encoded = Radix64Encoder::encode(&file_contents);

        assert_eq!(encoded.to_vec(), expected.to_vec());
    }
}
