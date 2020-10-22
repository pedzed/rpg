use super::decoder_errors::DecoderError;
use super::super::coding::BLOCKS_PER_SEXTET;
use super::super::tables;

const INVALID_VALUE: u8 = 255;

#[derive(Debug)]
pub struct Radix64Decoder;

impl Radix64Decoder {
    pub fn decode(input: &[u8]) -> Result<Vec<u8>, DecoderError> {
        let input = Self::remove_padding(input);
        let input = Self::remove_whitespaces(input);

        let all_chunks: Vec<&[u8]> = input.chunks(BLOCKS_PER_SEXTET).collect();
        let (last_chunk, main_chunks) = all_chunks.split_last().unwrap();

        let mut output;

        if last_chunk.len() == BLOCKS_PER_SEXTET {
            output = Self::decode_full_chunks(&all_chunks)?;
        } else {
            output = Self::decode_full_chunks(main_chunks)?;

            let mut unencoded_partial = Self::decode_partial_chunk(last_chunk)?;
            output.append(&mut unencoded_partial);
        }

        Ok(output)
    }

    fn remove_padding(input: &[u8]) -> &[u8] {
        let padding_count = input
            .iter()
            .rev()
            .take_while(|&b| *b == b'=')
            .count()
        ;

        match padding_count {
            0 => &input[0..input.len()-0],
            1 => &input[0..input.len()-1],
            2 => &input[0..input.len()-2],
            x => unreachable!("Padding count of {} not possible.", x),
        }
    }

    fn remove_whitespaces(input: &[u8]) -> Vec<u8> {
        input
            .iter()
            .copied()
            .filter(|c| !c.is_ascii_whitespace())
            .collect()
    }

    fn decode_full_chunks(chunks: &[&[u8]]) -> Result<Vec<u8>, DecoderError> {
        let mut unencoded: Vec<u8> = vec![];

        for chunk in chunks {
            unencoded.append(&mut Self::decode_full_chunk(chunk)?);
        }

        Ok(unencoded)
    }

    fn decode_full_chunk(chunk: &[u8]) -> Result<Vec<u8>, DecoderError> {
        let decoded_sextets = Self::decode_sextets(chunk)?;

        let sextets_joined: u32 =
            ((decoded_sextets[0] as u32) << 18) |
            ((decoded_sextets[1] as u32) << 12) |
            ((decoded_sextets[2] as u32) << 6) |
            ((decoded_sextets[3] as u32) << 0)
        ;

        Ok(vec![
            ((sextets_joined & 0b11111111_00000000_00000000) >> 16) as u8,
            ((sextets_joined & 0b00000000_11111111_00000000) >> 8) as u8,
            ((sextets_joined & 0b00000000_00000000_11111111) >> 0) as u8,
        ])
    }

    fn decode_sextets(chunk: &[u8]) -> Result<Vec<u8>, DecoderError>  {
        let mut decoded_sextets = vec![];

        for (i, &encoded_sextet) in chunk.iter().enumerate() {
            decoded_sextets.push(tables::STD_DECODE[encoded_sextet as usize]);

            if decoded_sextets[i] == INVALID_VALUE {
                return Err(DecoderError::UnexpectedChar(encoded_sextet))
            }
        }

        Ok(decoded_sextets)
    }

    fn decode_partial_chunk(chunk: &[u8]) -> Result<Vec<u8>, DecoderError> {
        let decoded_sextets = Self::decode_sextets(chunk)?;

        let output: Vec<u8>;

        match chunk.len() {
            3 => {
                output = vec![
                    ((decoded_sextets[0] << 2) | (decoded_sextets[1] >> 4)) as u8,
                    ((decoded_sextets[1] << 4) | (decoded_sextets[2] >> 2)) as u8,
                ];
            },
            2 => {
                output = vec![
                    ((decoded_sextets[0] << 2) | (decoded_sextets[1] >> 4)) as u8,
                ];
            },
            x => unreachable!("Chunk size of {} is not possible.", x),
        }

        Ok(output)
    }
}


#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

    #[test]
    fn remove_padding_0() {
        assert_eq!(Radix64Decoder::remove_padding(b"SGVs"), b"SGVs");
    }

    #[test]
    fn remove_padding_1() {
        assert_eq!(Radix64Decoder::remove_padding(b"SGU="), b"SGU");
    }

    #[test]
    fn remove_padding_2() {
        assert_eq!(Radix64Decoder::remove_padding(b"SA=="), b"SA");
    }

    #[test]
    fn decode_2_chars() {
        let decoded = Radix64Decoder::decode(b"SA==").unwrap();

        assert_eq!(decoded, b"H");
    }

    #[test]
    fn decode_3_chars() {
        let decoded = Radix64Decoder::decode(b"SGU=").unwrap();

        assert_eq!(decoded, b"He");
    }

    #[test]
    fn decode_4_chars() {
        let decoded = Radix64Decoder::decode(b"SGVs").unwrap();

        assert_eq!(decoded, b"Hel");
    }

    #[test]
    fn decode_6_chars() {
        let decoded = Radix64Decoder::decode(b"SGVsbA==").unwrap();

        assert_eq!(decoded, b"Hell");
    }

    #[test]
    fn decode_7_chars() {
        let decoded = Radix64Decoder::decode(b"SGVsbG8=").unwrap();

        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn decode_8_chars() {
        let decoded = Radix64Decoder::decode(b"SGVsbG8h").unwrap();

        assert_eq!(decoded, b"Hello!");
    }

    #[test]
    fn decode_16_chars() {
        let decoded = Radix64Decoder::decode(b"SGVsbG8gV29ybGQh").unwrap();

        assert_eq!(decoded, b"Hello World!");
    }

    #[test]
    fn decode_longer_text_without_line_breaks() {
        let encoded = b"\
            TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2Np\
            bmcgZWxpdCwgc2VkIGRvIGVpdXNtb2QgdGVtcG9yIGluY2lkaWR1bnQgdXQgbGFi\
            b3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdWEuIFV0IGVuaW0gYWQgbWluaW0gdmVu\
            aWFtLCBxdWlzIG5vc3RydWQgZXhlcmNpdGF0aW9uIHVsbGFtY28gbGFib3JpcyBu\
            aXNpIHV0IGFsaXF1aXAgZXggZWEgY29tbW9kbyBjb25zZXF1YXQuIER1aXMgYXV0\
            ZSBpcnVyZSBkb2xvciBpbiByZXByZWhlbmRlcml0IGluIHZvbHVwdGF0ZSB2ZWxp\
            dCBlc3NlIGNpbGx1bSBkb2xvcmUgZXUgZnVnaWF0IG51bGxhIHBhcmlhdHVyLiBF\
            eGNlcHRldXIgc2ludCBvY2NhZWNhdCBjdXBpZGF0YXQgbm9uIHByb2lkZW50LCBz\
            dW50IGluIGN1bHBhIHF1aSBvZmZpY2lhIGRlc2VydW50IG1vbGxpdCBhbmltIGlk\
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

        let decoded = Radix64Decoder::decode(encoded).unwrap();

        assert_eq!(decoded.to_vec(), expected.to_vec());
    }

    #[test]
    fn decode_longer_text_with_line_breaks() {
        let encoded = b"\
            TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2Np\r\
            bmcgZWxpdCwgc2VkIGRvIGVpdXNtb2QgdGVtcG9yIGluY2lkaWR1bnQgdXQgbGFi\n\
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

        let decoded = Radix64Decoder::decode(encoded).unwrap();

        assert_eq!(decoded.to_vec(), expected.to_vec());
    }

    #[test]
    fn decode_binary() {
        let encoded = b"\
            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\
            sU0Ob/4BrGxXIweWt2UAAAAASUVORK5CYII=\
        ";

        let decoded = Radix64Decoder::decode(encoded).unwrap();
        let file_contents = fs::read("tests/resources/gnupg-icon.png").unwrap();

        assert_eq!(decoded, file_contents);
    }
}
