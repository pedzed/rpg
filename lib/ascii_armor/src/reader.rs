// TODO: Refactor file to be more consistent with ArmorWriter
// TODO: Work on documentation

use std::io::SeekFrom;
use std::io::BufRead;
use std::io::Seek;

use crate::ArmorChecksum;
use crate::ArmorDataHeader;
use crate::ArmorDataHeaderMap;
use crate::ArmorDataType;
use crate::ArmorError;

use crate::LINE_ENDING;

pub trait SeekBufRead: Seek + BufRead {
    // NOTE: Nightly-only experimental
    // https://github.com/rust-lang/rust/issues/59359
    fn stream_len(&mut self) -> std::io::Result<u64> {
        let old_pos = self.stream_position()?;
        let len = self.seek(SeekFrom::End(0))?;

        // Avoid seeking a third time when we were already at the end of the
        // stream. The branch is usually way cheaper than a seek operation.
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }

        Ok(len)
    }

    // NOTE: Nightly-only experimental
    // https://github.com/rust-lang/rust/issues/59359
    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

impl<T: Seek + BufRead> SeekBufRead for T {}

/// ArmorReader for parsing ASCII Armor
///
/// # Links
/// - [RFC 4880, Section 6.2: Forming ASCII Armor](https://tools.ietf.org/html/rfc4880#section-6.2)
#[derive(Debug)]
pub struct ArmorReader {
    pub data_type: Result<ArmorDataType, ArmorError>,
    pub data_headers: Result<ArmorDataHeaderMap, ArmorError>,
    pub data: Result<Vec<u8>, ArmorError>,
    pub checksum: Result<ArmorChecksum, ArmorError>,
}

impl ArmorReader {
    /// Read ASCII Armor by parsing and decoding the data
    ///
    /// # Examples
    /// ```rust
    /// use ascii_armor::ArmorReader;
    /// use ascii_armor::ArmorDataType;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///    let mut buffer = std::io::Cursor::new("\
    ///        -----BEGIN PGP MESSAGE-----\r\n\
    ///        Version: rpg v0.1.0\r\n\
    ///        Comment: An example, just for you.\r\n\
    ///        \r\n\
    ///        SGVsbG8sIGJlYXV0aWZ1bCB3b3JsZCE=\r\n\
    ///        =4oUH\r\n\
    ///        -----END PGP MESSAGE-----\r\n\
    ///    ");
    ///
    ///    let armor = ArmorReader::read(&mut buffer)?;
    ///
    ///    assert_eq!(armor.data_type?, ArmorDataType::PgpMessage);
    ///    assert_eq!(armor.data_headers?.len(), 2);
    ///    assert_eq!(armor.data?, b"Hello, beautiful world!");
    ///    assert!(armor.checksum?.verify(b"Hello, beautiful world!"));
    ///
    ///    Ok(())
    /// }
    /// ```
    pub fn read(buffer: &mut dyn SeekBufRead) -> Result<Self, ArmorError> {
        let data_type = Self::parse_data_type(buffer);
        let data_headers = Self::parse_data_headers(buffer);
        let checksum = Self::parse_checksum(buffer);
        let data = match Self::parse_data(buffer) {
            Err(e) => Err(e),
            Ok(data) => match base64::decode(&data) {
                Err(e) => Err(e.into()),
                Ok(data) => Ok(data),
            },
        };

        Ok(Self {
            data_type,
            data_headers,
            data,
            checksum,
        })
    }

    fn parse_data_type(buffer: &mut dyn SeekBufRead) -> Result<ArmorDataType, ArmorError> {
        buffer.seek(SeekFrom::Start(0))?;

        for line in buffer.lines() {
            let line = Self::normalize_line(&line?);

            if Self::is_header_line(&line) {
                let from = "-----BEGIN ".len();
                let to = line.len() - "-----".len();
                let stripped = line[from..to].trim();
                return Ok(ArmorDataType::from_str(stripped)?);
            }
        }

        Err(ArmorError::ReaderUnknownDataType)
    }

    fn is_header_line(line: &str) -> bool {
        line.starts_with("-----BEGIN ")
    }

    fn normalize_line(line: &str) -> String {
        line
            .trim()
            .replace("\r\n", "⏎")
            .replace("\r", "⏎")
            .replace("\n", "⏎")
            .replace("⏎", LINE_ENDING)
    }

    fn parse_data_headers(buffer: &mut dyn SeekBufRead) -> Result<ArmorDataHeaderMap, ArmorError> {
        buffer.seek(SeekFrom::Start(0))?;

        let mut output = ArmorDataHeaderMap::new();

        for line in buffer.lines() {
            let line = line?;

            if !Self::is_data_header_line(&line) {
                continue;
            }

            let mut header = line.split(":");
            let key = header.next();
            let value = header.next();

            match key {
                None => continue,
                Some(key) => {
                    let key = ArmorDataHeader::from_str(key)?;

                    match value {
                        None => continue,
                        Some(value) => {
                            output
                                .entry(key)
                                .or_insert_with(Vec::new)
                                .push(value.trim().into())
                        },
                    }
                },
            }
        }

        Ok(output)
    }

    fn is_data_header_line(line: &str) -> bool {
        line.contains(":")
    }

    fn parse_data(buffer: &mut dyn SeekBufRead) -> Result<Vec<u8>, ArmorError> {
        buffer.seek(SeekFrom::Start(0))?;

        let buffer_length = buffer.stream_len()? as usize;

        let mut output = Vec::with_capacity(buffer_length);

        for line in buffer.lines() {
            let line = line?;
            let line = line.trim();

            if Self::is_header_line(line) ||
                Self::is_data_header_line(line) ||
                Self::is_tail_line(line) ||
                Self::is_checksum_line(line)
            {
                continue;
            }

            output.extend(line.as_bytes());
        }

        Ok(output)
    }

    fn is_tail_line(line: &str) -> bool {
        line.replace("-", "").trim().contains("END")
    }

    fn is_checksum_line(line: &str) -> bool {
        line.chars().nth(0).unwrap_or('.') == '=' &&
            line.len() == "=ABCD".len()
    }

    fn parse_checksum(buffer: &mut dyn SeekBufRead) -> Result<ArmorChecksum, ArmorError> {
        buffer.seek(SeekFrom::Start(0))?;

        for line in buffer.lines() {
            let line = line?;
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
        let mut buffer = std::io::Cursor::new("\
            -----BEGIN PGP MESSAGE-----\r\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();

        assert_eq!(armor.data_type.unwrap(), ArmorDataType::PgpMessage);
    }
    #[test]
    fn header_line_for_pgp_message_part_x() {
        let mut buffer = std::io::Cursor::new("\
            -----BEGIN PGP MESSAGE, PART 2-----\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();

        assert_eq!(armor.data_type.unwrap(), ArmorDataType::PgpMessagePartX(2));
    }

    #[test]
    fn header_line_for_pgp_message_part_xy() {
        let mut buffer = std::io::Cursor::new("\
            -----BEGIN PGP MESSAGE, PART 2/3-----  \r\n  \
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();

        assert_eq!(armor.data_type.unwrap(), ArmorDataType::PgpMessagePartXy(2, 3));
    }

    #[test]
    fn header_without_value() {
        let mut buffer = std::io::Cursor::new("\
            Version:\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();

        assert_eq!(
            armor.data_headers.unwrap().get(&ArmorDataHeader::Version),
            Some(&vec![String::new()])
        );
    }

    #[test]
    fn single_data_header() {
        let mut buffer = std::io::Cursor::new("\
            Version: OpenPrivacy 0.99\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();

        assert_eq!(
            armor.data_headers.unwrap().get(&ArmorDataHeader::Version),
            Some(&vec![String::from("OpenPrivacy 0.99")])
        );
    }

    #[test]
    fn multiple_data_headers_with_same_key() {
        let mut buffer = std::io::Cursor::new("\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();

        assert_eq!(
            armor.data_headers.unwrap().get(&ArmorDataHeader::Comment), Some(&vec![
                String::from("Comment on first line"),
                String::from("And also on second line"),
            ])
        );
    }

    #[test]
    fn multiple_data_headers_with_different_keys() {
        let mut buffer = std::io::Cursor::new("\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            Charset: UTF-8\r\n\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();
        let headers = armor.data_headers.unwrap();

        assert_eq!(
            headers.get(&ArmorDataHeader::Comment),
            Some(&vec![
                String::from("Comment on first line"),
                String::from("And also on second line"),
            ])
        );
        assert_eq!(
            headers.get(&ArmorDataHeader::Charset),
            Some(&vec![String::from("UTF-8")])
        );
    }

    #[test]
    fn data_and_checksum() {
        let mut buffer = std::io::Cursor::new("\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();

        assert_eq!(armor.data.unwrap(), b"Hello");
        assert!(armor.checksum.unwrap().verify(b"Hello"));
    }

    #[test]
    fn everything_with_text_data() {
        let mut buffer = std::io::Cursor::new("\
            -----BEGIN PGP MESSAGE-----\r\n\
            Version: Test\r\n\
            Comment: Aren't tests great?\r\n\
            \r\n\
            SGVsbG8sIGJlYXV0aWZ1bCB3b3JsZCE=\r\n\
            =4oUH\r\n\
            -----END PGP MESSAGE-----\r\n\
        ");

        let armor = ArmorReader::read(&mut buffer).unwrap();

        assert_eq!(armor.data_type.unwrap(), ArmorDataType::PgpMessage);
        assert_eq!(armor.data_headers.unwrap().len(), 2);
        assert_eq!(armor.data.unwrap(), b"Hello, beautiful world!");
        assert!(armor.checksum.unwrap().verify(b"Hello, beautiful world!"));
    }

    #[test]
    fn everything_with_binary_data_from_file() {
        let binary_file = "tests/resources/gnupg-icon.png";
        let expected_data = std::fs::read(binary_file).unwrap();

        let armor_file = "tests/resources/gnupg-icon.png.asc";
        let file = std::fs::File::open(armor_file).unwrap();
        let mut buffer = std::io::BufReader::new(file);
        let armor = ArmorReader::read(&mut buffer).unwrap();
        let armor_data = armor.data.unwrap();

        assert_eq!(armor_data, expected_data);
        assert!(armor.checksum.unwrap().verify(&expected_data));
    }
}
