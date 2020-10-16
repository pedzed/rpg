use std::io;
use std::fs;
use std::collections::HashMap;

use super::armor_checksums::ArmorChecksum;
use super::armor_data_types::ArmorDataTypeError;
use super::armor_data_types::ArmorDataType;
use super::armor_headers::ArmorHeader;
use super::super::armor::ArmorHeaderMap;
use super::super::armor::ArmorData;
use super::super::armor::LINE_ENDING;
use super::super::coding::Radix64;

#[derive(Debug, PartialEq)]
pub struct ArmorReaderError(String);

// 6.2.  Forming ASCII Armor
// https://tools.ietf.org/html/rfc4880#section-6.2
#[derive(Debug)]
pub struct ArmorReader {
    data_type: Result<ArmorDataType, ArmorDataTypeError>,
    headers: ArmorHeaderMap,
    data: Result<ArmorData, ArmorReaderError>,
    checksum: Result<ArmorChecksum, ArmorReaderError>,
}

impl ArmorReader {
    pub fn read_file(file: &str) -> Result<Self, io::Error> {
        let file_contents = fs::read_to_string(file)?;

        Ok(Self::read_str(&file_contents))
    }

    pub fn read_str(input: &str) -> Self {
        let input = Self::normalize(input);

        let data_type = Self::parse_data_type(&input);
        let headers = Self::parse_headers(&input);
        let data = Self::parse_data(&input);
        let checksum = Self::parse_checksum(&input);

        Self {
            data_type,
            headers,
            data,
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

    fn parse_data_type(input: &str) -> Result<ArmorDataType, ArmorDataTypeError> {
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

    fn parse_headers(input: &str) -> ArmorHeaderMap {
        let mut output = HashMap::new();

        input.lines()
            .for_each(|line| {
                if !line.contains(":") {
                    return
                }

                let split: Vec<&str> = line.split(":").collect();

                if split.len() != 2 {
                    return
                }

                let key = ArmorHeader::from_str(split[0]);

                if key.is_err() {
                    // TODO: Log failure
                    return
                }

                let value = split[1].trim();

                output
                    .entry(key.unwrap())
                    .or_insert_with(Vec::new)
                    .push(String::from(value))
            })
        ;

        output
    }

    fn parse_data(input: &str) -> Result<ArmorData, ArmorReaderError> {
        let data = input
            .rsplitn(2, "\r\n\r\n").next().unwrap()
            .trim()
            .lines()
            .filter(|line| {
                !Self::is_tail_line(line) &&
                !Self::is_checksum_line(line)
            })
            .collect::<Vec<&str>>()
            .join("\r\n") // ..
        ;

        if data.len() > 0 {
            Ok(Radix64::decode(&data))
        } else {
            Err(ArmorReaderError(String::from("Not yet implemented.")))
        }
    }

    fn is_tail_line(line: &str) -> bool {
        line.replace("-", "").trim().contains("END")
    }

    fn is_checksum_line(line: &str) -> bool {
        let line = line.trim();

        line.chars().nth(0).unwrap_or('.') == '=' &&
            line.len() == "=EHJM".len()
    }

    fn parse_checksum(input: &str) -> Result<ArmorChecksum, ArmorReaderError> {
        for line in input.lines() {
            let line = line.trim();

            if Self::is_checksum_line(line) {
                let checksum = ArmorChecksum::new(line);
                return Ok(checksum)
            }
        }

        Err(ArmorReaderError(String::from("Failed to find checksum.")))
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
    fn single_header() {
        let armor = ArmorReader::read_str("\
            Version: OpenPrivacy 0.99\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(
            armor.headers.get(&ArmorHeader::Version),
            Some(&vec![String::from("OpenPrivacy 0.99")])
        );
    }

    #[test]
    fn multiple_headers_with_same_key() {
        let armor = ArmorReader::read_str("\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(
            armor.headers.get(&ArmorHeader::Comment),
            Some(&vec![
                String::from("Comment on first line"),
                String::from("And also on second line"),
            ])
        );
    }

    #[test]
    fn multiple_headers_with_different_keys() {
        let armor = ArmorReader::read_str("\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            Charset: UTF-8\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        assert_eq!(
            armor.headers.get(&ArmorHeader::Comment),
            Some(&vec![
                String::from("Comment on first line"),
                String::from("And also on second line"),
            ])
        );
        assert_eq!(
            armor.headers.get(&ArmorHeader::Charset),
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

        let data = armor.data.unwrap();

        assert_eq!(data.encoded, "SGVsbG8=");
        assert_eq!(data.unencoded, b"Hello".to_vec());

        let checksum = armor.checksum.unwrap();

        assert!(checksum.verify(&data.unencoded));
    }

    #[test]
    fn everything_with_text_data() {
        let armor = ArmorReader::read_str("
            -----BEGIN PGP MESSAGE-----\r\n\
            Version: OpenPrivacy 0.99\r\n\
            Comment: Note that some\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
            -----END PGP MESSAGE-----\r\n\
        ");

        assert_eq!(armor.data_type.unwrap(), ArmorDataType::PgpMessage);

        assert_eq!(armor.headers.len(), 2);

        let data = armor.data.unwrap();
        assert_eq!(data.encoded, "SGVsbG8=");
        assert_eq!(data.unencoded, b"Hello".to_vec());

        assert!(armor.checksum.unwrap().verify(&data.unencoded));
    }

    // #[test]
    // fn everything_with_binary_data_from_str() {
    //     let armor = ArmorReader::read_str("
    //         -----BEGIN PGP MESSAGE-----\r\n\
    //         Version: Test #123\r\n\
    //         Comment: Note that some transport methods are sensitive to line length.  While\r\n\
    //         Comment: there is a limit of 76 characters for the Radix-64 data (Section\r\n\
    //         Comment: 6.3), there is no limit to the length of Armor Headers.  Care should\r\n\
    //         Comment: be taken that the Armor Headers are short enough to survive\r\n\
    //         Comment: transport.  One way to do this is to repeat an Armor Header key\r\n\
    //         Comment: multiple times with different values for each so that no one line is\r\n\
    //         Comment: overly long.\r\n\

    //         XDg5UE5HChoKXDAwXDAwXDAwCklIRFJcMDBcMDBcMDBcODBcMDBcMDBcMDBcODAI\r\n\
    //         A1wwMFwwMFwwMFxGNFxFMFw5MVxGOVwwMFwwMFwwMDNQTFRFXDAwXDkzXEREClw5\r\n\
    //         OFxERTxcQURcRTZcOTVcRDNcRjF2XEM1XEVEXDAwXDlEXEVEXEJDXEUzXEY3XEZG\r\n\
    //         XEZGXEZGXDAwXDhEXERCXDAwXDk4XEU2XEY2XEZCXEZFXEQ2XEVFXEZBXEU4XEY1\r\n\
    //         XEZDJ1xBM1xFMlxBRVxERFxGNVJcQjVcRThcXEJB6aGsXDhFDlwwMFwwMAVySURB\r\n\
    //         VHhcREFcRURcOULZklxEQyAMRQFhGlxCM1xGRlxGRteGXEM1XEVFBlxBRlxFMG1c\r\n\
    //         QUFSXEQ2QzJ5XEM43IMQSAZcODE+J0woKVkwXEE5XDg0OFxGODtcRDBcQzFcRkZc\r\n\
    //         QTcYOFxBMjFCXEE4XEY3XDg2ENaWXDgyFA8BSCBcQkFcRUZcQkIuKA8WXEZGXDg5\r\n\
    //         NDVcRTJ2XDAwBQQHXEIxJVxFQlxCQl5TeStcODBcQTReHW1YXERGIVxDN24DUF5+\r\n\
    //         HDsPXEY2XEZCXDg5XEZGGDpkXEU1LVwwMAJcRjQgXEVGXEY1MHEUTFwDBlw4MFxG\r\n\
    //         QVw4MFxDQyA6DFxFMnpcMDBGXDkyXEJDXEQ3IdSoXDk5dwwlaGRcRTg7Ii8GEFxE\r\n\
    //         RVxGQkkCzNcRTlHT1xBOXM0elxDM28KPkBcQkZOMFw5NwJcQTg0fC8PalxcOEJcQ\r\n\
    //         0VcRUYDPDNcOENcODlcQTUWfxcmXAhcQzB0GlxGRThcQjd+JwjaiFw5N0tcQTBcO\r\n\
    //         DjFvlxBM1w5NwEYXDlDXDg2XEVGXEQyXEU4XDk5w5xcQTN9XEFCIVxBOAJcODBcQ\r\n\
    //         jhcRTVcRjlcRTFcQTdcRDFbVFxBM15JXDgwXEVBXEY1XEI1TFxDMVgtX0VQAWBcO\r\n\
    //         TI+USkYGlxFNEMgXEMwaVwwMFw4NjN9QG1cRkFcQThcQzdcRUMkXDgwXEQyXEM5X\r\n\
    //         EZGQV9cQjhGXEY5QFxBMFxENSlcMDBBXEJBGH8yXEVFBVxFRFxGQT4MXERDKVwwM\r\n\
    //         Fw5Qda/WVxEM1xFRlM9cjwMXEQwXlwwMEZ9GlxFN1w4Ms+Sf1w4RlxCNTccflxEQ\r\n\
    //         3ABVlw4NwFUCkAtXEMyXFw5NFxGQV1cOEYdMFxFNVw4QkFfGVxGQTxcODRcRDcZN\r\n\
    //         lxEN1xFMjZcODBcRkJOXDgwXEIwXEJDXDk0XEI3XDkzfChcQTleR1w5MAcBWFw5Q\r\n\
    //         VwwMBtiXDgxFxNPXEQ4UlxBRFw4NldqXEE1XDhFHANIKyBcRjIDLyZ1JVxBQ1w5N\r\n\
    //         FsJyJ4dAlw4MG50XDgwXENDXEY3XDlGXEFEWgNcOTYJNlxcQjABIHRcRkFlXEFDD\r\n\
    //         FxDMM6KXEREfVxCM1xDMQVoXEQ3AVxBNAxcODBtXEZENVw4Ms62AwwOXEUwXEYwY\r\n\
    //         Vw4NTNFVVxFRVw5QRpcOTZcQ0RcMDBpD0JYXEU1E1xCMFxCN1xCM1xGRjxXXEJCF\r\n\
    //         1xBQwNcRDgxBFxGMxVcRDBcRDdUXDlBXEE0W1xDQUlcQTIRIG1cODJ+E1JcQjhmK\r\n\
    //         lw4Qj0JLUxcQzIaOlxEQXEkVlkEXEY2XEI4XEFF2KddfVJEOzNAWFxCNgVcRUNk1\r\n\
    //         p8LFlZcQzJaXEY0XEEwXDlENUBcOEIaXEE0XEY2XEFCXEQzdVxENXNcODB2XEU2X\r\n\
    //         DkxZBHQk1xEQVxDRihcRDNXXEFGXDgzNVwwMFw4NnBcOTTGjWV+EVxDMVxBNQdcO\r\n\
    //         TZ3EFxCNFw5MQgxXEM2XEU1GFxBMBZcRTBDFlwwMFw5NiMYbWZiXEY0AlxCQ1wwM\r\n\
    //         C9cQzALXEYwAlxCQ1wwMC9cQzALXEYwAlxCQ1wwMC9cQzBcRkYFXEEwTgNcOTgzX\r\n\
    //         DAwHFV6XDAwXEE4N3BcRTFcQkJoalw5NtaaXDlFXEZFV1xGRlw5NWkqP1xDREgXb\r\n\
    //         FxGOVxBQ1xBQlxBQlxCNVxEOVxGMFxCNVxBOT5cQTIhO1w4N1xERghcRUZcRkYxX\r\n\
    //         DkzX1xCRkg9XDAwXEQwahxcODNqOVxBQVxCRBhcODBjXEFBXERBXEVFCy4FXEYwX\r\n\
    //         EYyXEIyXEY1XENFXEU4OlwwMFxCRS9/I1wwMA9cQjdcRENcRkJcQTdcOUFcRjdcM\r\n\
    //         DBcODQrZG1XXERBSVxFNFxCRFwwMFxCMVw5N0ITXENBXEQ2XEQ2HdC7XDAwXEY4X\r\n\
    //         EE4bdiGXEU3aX85XEMwXEQwQRJ6B1xDMFxDOFxFRFkXXDhFXwVcQzBcQzdcQ0UVX\r\n\
    //         EFDB+WKphFJXEY47KyvFWAcXEFGXEQ3dRQqXDg1XEM3XEUzS1xDQ1xEMQlcODAkX\r\n\
    //         EFDU1xGM8yBXEE2LUFjXEQzXEMzQVw4MCFcQkNaXEM2Ox1cRkVwXEYxegRcQzBcR\r\n\
    //         UZoFlJ4HdWXbjx0bwdcRTBcODhcODBcRkFcOUMyAVw5OFxBRh04XEVGAXBuXEQ5X\r\n\
    //         EU3XEE0eVxFRmdcOTcebVwwMFxFQlw5NUxcQjUxXEYyXDkzXEY3XDkzXEQ5XEU0A\r\n\
    //         Vw4RVxBODgtXDhGMnkKXDkyNFwwMHB9XEQ2XEZCZiIvPlxBNmEKXEI4PRd7CgpcR\r\n\
    //         TfHulBcRThqXDAwXENF4ZS8XEE0eCYfLlw4MWtjXDgwXEEzM1xGQVxDMlxEOEJ+X\r\n\
    //         EFDC1lcRjUy5JidGXxcQkVcRUVcQjIGXEJD0IZSBzBcQjRtHVJcODdfW2NqflxGQ\r\n\
    //         30TXEI4XERBZHRYX0FcRUVcRkFcRThcRkLMk1BcOUJcOENcODZcQjZcQURcRDZ5X\r\n\
    //         EY3Yy8mPmRcOTBcRkMXXEE1S1xGMBpcODADXEZBXEExXEExXEY0XEFCHlxCQitnb\r\n\
    //         VxBNwZVAlxFOEZ9IVM3XEVCaFxCMVxDQlxEQlxDQ1xFNnDQr1wwMFxBMFw4RFxFM\r\n\
    //         xcsfFxEMDtnXDlEN1grVlxBMD4dXDlGXEREXEZEXDk3XDhCXEUxXDg2dHxcODNcQ\r\n\
    //         lcQUMKXEVCL1wwMFw4QS5cQ0NcRTcBVNmEXEZBOFw4MFw5OXQBPgxcOTBcOTdDf1\r\n\
    //         wwMFxDMBY6XDkwHwRgXDhCClxEME8BKCBcQ0IdXEMwXDhGXDAwCFZUBA8DeFxGNV\r\n\
    //         xCQyJ4GEAZXEE3XEE3XEVBXEJFJFxDMk9cMDDErFw4OFxGOUzdl0RwXEI3B0JfXE\r\n\
    //         FGXEQ1M1xGMXRSXEM2fC1+XEQzVhxcOUVcOUEZXEEwXDk2XEE0XDk3HgsnZemsil\r\n\
    //         xcQkMVKwNcQ0USbxpjXEM0XEU3XEQyXEMzSVw5OR1cOUZcOUVcOTUdXEYxV3hASA\r\n\
    //         ZcQzlcRTNcODVcRkNcQUYiIlxGOVxCQjd6Vy5QXEMxXEZELlxGQSFcOTkmXEM0Ug\r\n\
    //         plRTRcRTnIvlxBMyISQgUTYlxFOcmhXDlDdVxFND9bXDkyZScjf1w5MQolWVxEOB\r\n\
    //         BcOUbzgKKYH9qKxZ1cRjJPATBcOEFcOEZnXEMzXEQzXDAwXEFCXEE1XEMwI1wwMF\r\n\
    //         w5M1xFRlxFM2cBFFxBM2RcRjcVXERDLlxDMFxCMRdcQ0MxF2tUXEYxBlxFRmoPXD\r\n\
    //         g0JzZcRjEJXEY0QlJcQkFcRDYDXEVBXEJCXEUxXEZBXEJGXEE0XDk0XEYxXEE5a3\r\n\
    //         p8XVxBQn1cREMDXEExXENDXEYzOUcPFi5cQzM5XEUyXDlDNykfXEY3XDgwXDAwXE\r\n\
    //         NESztzXEFEXEQ2DhDOulxGQlxFQlxBQzkZKXZcQjFNDm9cRkUBXEFDbFcjB1w5Nl\r\n\
    //         xCN2VcMDBcMDBcMDBcMDBJRU5EXEFFQmBcODI=\r\n\
    //         =/u+x\r\n\
    //         -----END PGP MESSAGE----\r\n\
    //     ");

    //     assert_eq!(armor.data_type.unwrap(), ArmorDataType::PgpMessage);

    //     assert_eq!(armor.headers.len(), 8);

    //     let data = armor.data.unwrap();
    //     // assert_eq!(data.encoded, "SGVsbG8=");
    //     // assert_eq!(data.unencoded, b"Hello".to_vec());

    //     assert!(armor.checksum.unwrap().verify(&data.unencoded));
    // }

    // #[test]
    // fn everything_with_binary_data_from_file() {
    //     let file = "tests/resources/radix64/armor/gnupg-icon.png.asc";
    //     let armor = ArmorReader::read_file(file).unwrap();

    //     let bytes_decoded = armor.data.unwrap().unencoded;
    //     let bytes_from_file = fs::read(file).unwrap();

    //     assert_eq!(bytes_decoded, bytes_from_file);
    //     assert!(armor.checksum.unwrap().verify(&bytes_from_file));
    // }
}
