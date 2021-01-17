// TODO: Refactor file to be more consistent with ArmorWriter
// TODO: Work on documentation

use std::io;
use std::fs;

use base64::DecoderError;

use crate::ArmorChecksum;
use crate::ArmorDataHeader;
use crate::ArmorDataHeaderMap;
use crate::ArmorDataType;
use crate::ArmorError;

use crate::LINE_ENDING;

type ArmorData = Vec<u8>;

// 6.2.  Forming ASCII Armor
// https://tools.ietf.org/html/rfc4880#section-6.2
#[derive(Debug)]
pub struct ArmorReader {
    pub data_type: Result<ArmorDataType, ArmorError>,
    pub data_headers: ArmorDataHeaderMap,
    pub encoded_data: Result<ArmorData, ArmorError>,
    pub decoded_data: Result<ArmorData, DecoderError>,
    pub checksum: Result<ArmorChecksum, ArmorError>,
}

impl ArmorReader {
    pub fn read_file(file: &str) -> Result<Self, io::Error> {
        let file_contents = fs::read_to_string(file)?;

        Ok(Self::read_str(&file_contents))
    }

    pub fn read_str(input: &str) -> Self {
        let input = Self::normalize(input);

        let data_type = Self::parse_data_type(&input);
        let data_headers = Self::parse_data_headers(&input);
        let encoded_data = Self::parse_data(&input);
        let decoded_data = base64::decode(&encoded_data);
        let checksum = Self::parse_checksum(&input);

        Self {
            data_type,
            data_headers,
            encoded_data: Ok(encoded_data),
            decoded_data,
            checksum,
        }
    }

    fn normalize(armor: &str) -> String {
        armor.trim()
            .replace("\r\n", "⏎")
            .replace("\r", "⏎")
            .replace("\n", "⏎")
            .replace("⏎", LINE_ENDING)
    }

    fn parse_data_type(input: &str) -> Result<ArmorDataType, ArmorError> {
        let mut stripped_header_line = "";

        for line in input.lines() {
            if line.starts_with("-----BEGIN") {
                let from = "-----BEGIN".len();
                let to = line.len() - "-----".len();
                stripped_header_line = line[from..to].trim();
                break
            }
        }

        ArmorDataType::from_str(stripped_header_line)
    }

    fn parse_data_headers(input: &str) -> ArmorDataHeaderMap {
        let mut output = ArmorDataHeaderMap::new();

        input.lines()
            .filter(|line| Self::is_data_header_line(line))
            .map(|line| line.split(":").collect::<Vec<&str>>())
            .for_each(|data_header| {
                let key = ArmorDataHeader::from_str(data_header[0]);

                if key.is_err() {
                    // TODO: Log failure
                    return
                }

                let value = data_header[1].trim();

                output
                    .entry(key.unwrap())
                    .or_insert_with(Vec::new)
                    .push(String::from(value))
            })
        ;

        output
    }

    fn is_data_header_line(line: &str) -> bool {
        line.contains(":")
    }

    fn parse_data(input: &str) -> ArmorData {
        input
            .rsplitn(2, &LINE_ENDING.repeat(2)).next().unwrap()
            .trim()
            .lines()
            .filter(|line| {
                !Self::is_tail_line(line) &&
                !Self::is_checksum_line(line)
            })
            .collect::<Vec<&str>>()
            .join("")
            .as_bytes()
            .to_vec()
    }

    fn is_tail_line(line: &str) -> bool {
        line.replace("-", "").trim().contains("END")
    }

    fn is_checksum_line(line: &str) -> bool {
        let line = line.trim();

        line.chars().nth(0).unwrap_or('.') == '=' &&
            line.len() == "=EHJM".len()
    }

    fn parse_checksum(input: &str) -> Result<ArmorChecksum, ArmorError> {
        for line in input.lines() {
            let line = line.trim();

            if Self::is_checksum_line(line) {
                return ArmorChecksum::new(line);
            }
        }

        Err(ArmorError::ReaderUnknownChecksum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_line_for_pgp_message() {
        let armor = ArmorReader::read_str("\
            -----BEGIN PGP MESSAGE-----\r\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(armor.data_type, Ok(ArmorDataType::PgpMessage));
    }

    #[test]
    fn header_line_for_pgp_message_part_x() {
        let armor = ArmorReader::read_str("\
            -----BEGIN PGP MESSAGE, PART 2-----\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(armor.data_type, Ok(ArmorDataType::PgpMessagePartX(2)));
    }

    #[test]
    fn header_line_for_pgp_message_part_xy() {
        let armor = ArmorReader::read_str("\
            -----BEGIN PGP MESSAGE, PART 2/3-----\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(armor.data_type, Ok(ArmorDataType::PgpMessagePartXy(2, 3)));
    }

    #[test]
    fn header_without_value() {
        let armor = ArmorReader::read_str("\
            Version:\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(
            armor.data_headers.get(&ArmorDataHeader::Version),
            Some(&vec![String::new()])
        );
    }

    #[test]
    fn single_data_header() {
        let armor = ArmorReader::read_str("\
            Version: OpenPrivacy 0.99\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(
            armor.data_headers.get(&ArmorDataHeader::Version),
            Some(&vec![String::from("OpenPrivacy 0.99")])
        );
    }

    #[test]
    fn multiple_data_headers_with_same_key() {
        let armor = ArmorReader::read_str("\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(
            armor.data_headers.get(&ArmorDataHeader::Comment),
            Some(&vec![
                String::from("Comment on first line"),
                String::from("And also on second line"),
            ])
        );
    }

    #[test]
    fn multiple_data_headers_with_different_keys() {
        let armor = ArmorReader::read_str("\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            Charset: UTF-8\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(
            armor.data_headers.get(&ArmorDataHeader::Comment),
            Some(&vec![
                String::from("Comment on first line"),
                String::from("And also on second line"),
            ])
        );
        assert_eq!(
            armor.data_headers.get(&ArmorDataHeader::Charset),
            Some(&vec![String::from("UTF-8")])
        );
    }

    #[test]
    fn data_and_checksum() {
        let armor = ArmorReader::read_str("\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(armor.decoded_data.unwrap(), b"Hello");
        assert!(armor.checksum.unwrap().verify(b"Hello"));
    }

    #[test]
    fn everything_with_text_data() {
        let armor = ArmorReader::read_str("
            -----BEGIN PGP MESSAGE-----\r\n\
            Version: Test\r\n\
            Comment: Aren't tests great?\r\n\
            \r\n\
            SGVsbG8sIGJlYXV0aWZ1bCB3b3JsZCE=\r\n\
            =4oUH\r\n\
            -----END PGP MESSAGE-----\r\n\
        ");

        assert_eq!(armor.data_type.unwrap(), ArmorDataType::PgpMessage);
        assert_eq!(armor.data_headers.len(), 2);
        assert_eq!(armor.decoded_data.unwrap(), b"Hello, beautiful world!");
        assert!(armor.checksum.unwrap().verify(b"Hello, beautiful world!"));
    }

    #[test]
    fn everything_with_binary_data_from_file() {
        let binary_file = "tests/resources/gnupg-icon.png";
        let expected_data = fs::read(binary_file).unwrap();

        let armor_file = "tests/resources/gnupg-icon.png.asc";
        let armor = ArmorReader::read_file(armor_file).unwrap();
        let armor_data = armor.decoded_data.unwrap();

        assert_eq!(armor_data, expected_data);
        assert!(armor.checksum.unwrap().verify(&expected_data));
    }
}
