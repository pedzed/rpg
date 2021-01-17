use std::io::Write;
use std::io::Result;

use crate::{ArmorChecksum, ArmorDataHeader, ArmorDataType};
use crate::ArmorDataHeaderMap;
use crate::LINE_ENDING;

/// The encoded output stream must be represented in lines of no more
/// than 76 characters each according to RFC 4880. GnuPG uses 64.
const LINE_LENGTH: usize = 64;

#[derive(Debug)]
/// ArmorWriter for creating ASCII Armor
///
/// Writes to the buffer of choice, where the buffer implements the Write trait.
pub struct ArmorWriter<'a> {
    data_type: Option<ArmorDataType>,
    data_headers: ArmorDataHeaderMap,
    data: Option<&'a [u8]>,
    checksum: Option<ArmorChecksum>,
}

impl<'a> ArmorWriter<'_> {
    /// Write to the buffer without failing on missing properties
    pub fn write_unchecked(&self, buffer: &mut dyn Write) -> Result<()> {
        if let Some(data_type) = &self.data_type {
            self.write_header_line(data_type, buffer)?;
        }

        self.write_data_headers(&self.data_headers, buffer)?;
        self.write_newline(buffer)?;

        if let Some(data) = self.data {
            self.write_data(data, buffer)?;
            self.write_checksum(&self.checksum.unwrap().to_owned(), buffer)?;
        }

        if let Some(data_type) = &self.data_type {
            self.write_footer_line(data_type, buffer)?;
        }

        Ok(())
    }

    fn write_header_line(&self, data_type: &ArmorDataType, buffer: &mut dyn Write) -> Result<()> {
        let header_line = format!("-----BEGIN {}-----{}", data_type.to_string(), LINE_ENDING);
        buffer.write_all(header_line.as_bytes())?;

        Ok(())
    }

    fn write_footer_line(&self, data_type: &ArmorDataType, buffer: &mut dyn Write) -> Result<()> {
        let header_line = format!("-----END {}-----{}", data_type.to_string(), LINE_ENDING);
        buffer.write_all(header_line.as_bytes())?;

        Ok(())
    }

    fn write_data_headers(&self, data_headers: &ArmorDataHeaderMap, buffer: &mut dyn Write) -> Result<()> {
        for (key, values) in data_headers {
            for value in values {
                let data_header_line = format!("{}: {}{}", key.to_str(), value, LINE_ENDING);
                buffer.write_all(data_header_line.as_bytes())?;
            }
        }

        Ok(())
    }

    fn write_data(&self, data: &[u8], buffer: &mut dyn Write) -> Result<()> {
        let encoded = base64::encode(data);

        for line in encoded.chunks(LINE_LENGTH) {
            buffer.write_all(&line)?;
            self.write_newline(buffer)?;
        }

        Ok(())
    }

    fn write_checksum(&self, checksum: &ArmorChecksum, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_all(checksum.get().as_bytes())?;
        self.write_newline(buffer)?;

        Ok(())
    }

    fn write_newline(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_all(LINE_ENDING.as_bytes())?;
        Ok(())
    }
}

/// ArmorWriterBuilder for building ArmorWriter data
///
/// Provides a fluent interface to chain methods.
///
/// # Examples
/// ```rust
/// use ascii_armor::ArmorDataType;
/// use ascii_armor::ArmorDataHeader;
/// use ascii_armor::ArmorWriterBuilder;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let armor = ArmorWriterBuilder::new()
///         .data_type(ArmorDataType::PgpMessage)
///         .add_data_header(ArmorDataHeader::Charset, "UTF-8")
///         .add_data_header(ArmorDataHeader::Comment, "This is an example comment, to demonstrate how a long comment")
///         .add_data_header(ArmorDataHeader::Comment, "can be split in multiple data headers.")
///         .data(b"Lorem ipsum dolor sit amet.")
///         .build()
///     ;
///
///     let mut buffer: Vec<u8> = vec![];
///     armor.write_unchecked(&mut buffer)?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ArmorWriterBuilder<'a> {
    data_type: Option<ArmorDataType>,
    data_headers: ArmorDataHeaderMap,
    data: Option<&'a [u8]>,
    checksum: Option<ArmorChecksum>,
}

impl<'a> ArmorWriterBuilder<'a> {
    /// Prepare the builder
    pub fn new() -> Self {
        Self {
            data_type: None,
            data_headers: ArmorDataHeaderMap::new(),
            data: None,
            checksum: None,
        }
    }

    /// Set the armor data type
    ///
    /// This determines the header and footer lines.
    pub fn data_type(mut self, data_type: ArmorDataType) -> Self {
        self.data_type = Some(data_type);
        self
    }

    /// Add a data header
    ///
    /// Multiple data headers can be added, even with the same key
    pub fn add_data_header(mut self, key: ArmorDataHeader, value: &str) -> Self {
        self.data_headers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(value.to_string())
        ;
        self
    }

    /// Set the data
    ///
    /// A checksum of the provided data is automatically generated. Then the
    /// data gets encoded using Radix-64.
    pub fn data(mut self, data: &'a [u8]) -> Self {
        self.data = Some(data);
        self.checksum = Some(ArmorChecksum::from_data(data));
        self
    }

    /// Build the ArmorWriter
    pub fn build(self) -> ArmorWriter<'a> {
        ArmorWriter {
            data_type: self.data_type,
            data_headers: self.data_headers,
            data: self.data,
            checksum: self.checksum,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ArmorDataType;
    use crate::ArmorDataHeader;
    use super::ArmorWriterBuilder;

    #[test]
    fn header_footer_lines_for_pgp_message() {
        let armor = ArmorWriterBuilder::new()
            .data_type(ArmorDataType::PgpMessage)
            .build()
        ;

        let mut buffer: Vec<u8> = vec![];
        armor.write_unchecked(&mut buffer).unwrap();

        assert_eq!(buffer, b"\
            -----BEGIN PGP MESSAGE-----\r\n\
            \r\n\
            -----END PGP MESSAGE-----\r\n\
        ");
    }

    #[test]
    fn single_data_header() {
        let armor = ArmorWriterBuilder::new()
            .add_data_header(ArmorDataHeader::Version, "OpenPrivacy 0.99")
            .build()
        ;

        let mut buffer: Vec<u8> = vec![];
        armor.write_unchecked(&mut buffer).unwrap();

        assert_eq!(buffer, b"\
            Version: OpenPrivacy 0.99\r\n\
            \r\n\
        ");
    }

    #[test]
    fn multiple_data_headers_with_single_key() {
        let armor = ArmorWriterBuilder::new()
            .add_data_header(ArmorDataHeader::Comment, "Comment on first line")
            .add_data_header(ArmorDataHeader::Comment, "And also on second line")
            .build()
        ;

        let mut buffer: Vec<u8> = vec![];
        armor.write_unchecked(&mut buffer).unwrap();

        assert_eq!(buffer, b"\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            \r\n\
        ");
    }

    #[test]
    fn multiple_data_headers_with_multiple_keys() {
        let armor = ArmorWriterBuilder::new()
            .add_data_header(ArmorDataHeader::Comment, "Comment on first line")
            .add_data_header(ArmorDataHeader::Comment, "And also on second line")
            .add_data_header(ArmorDataHeader::Charset, "UTF-8")
            .build()
        ;

        let mut buffer: Vec<u8> = vec![];
        armor.write_unchecked(&mut buffer).unwrap();

        let mut expected = *b"\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            Charset: UTF-8\r\n\
            \r\n\
        ";

        assert_eq!(buffer.len(), expected.len());

        // HashMap elements have an arbitrary order, hence sort:
        buffer.sort();
        expected.sort();

        assert_eq!(buffer, expected);
    }

    #[test]
    fn data_and_checksum() {
        let armor = ArmorWriterBuilder::new()
            .data(b"Hello")
            .build()
        ;

        let mut buffer: Vec<u8> = vec![];
        armor.write_unchecked(&mut buffer).unwrap();

        assert_eq!(buffer, b"\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");
    }

    #[test]
    fn everything_with_binary_data() {
        let data = std::fs::read("tests/resources/gnupg-icon.png").unwrap();

        let armor = ArmorWriterBuilder::new()
            .data_type(ArmorDataType::PgpMessage)
            .add_data_header(ArmorDataHeader::Version, "OpenPrivacy 0.99")
            .add_data_header(ArmorDataHeader::Comment, "Note that some transport methods are sensitive to line length.  While")
            .add_data_header(ArmorDataHeader::Comment, "there is a limit of 76 characters for the Radix-64 data (Section")
            .add_data_header(ArmorDataHeader::Comment, "6.3), there is no limit to the length of Armor Headers.  Care should")
            .add_data_header(ArmorDataHeader::Comment, "be taken that the Armor Headers are short enough to survive")
            .add_data_header(ArmorDataHeader::Comment, "transport.  One way to do this is to repeat an Armor Header key")
            .add_data_header(ArmorDataHeader::Comment, "multiple times with different values for each so that no one line is")
            .add_data_header(ArmorDataHeader::Comment, "overly long.")
            .data(&data)
            .build()
        ;

        let mut buffer: Vec<u8> = vec![];
        armor.write_unchecked(&mut buffer).unwrap();

        let mut expected = *b"\
            -----BEGIN PGP MESSAGE-----\r\n\
            Version: OpenPrivacy 0.99\r\n\
            Comment: Note that some transport methods are sensitive to line length.  While\r\n\
            Comment: there is a limit of 76 characters for the Radix-64 data (Section\r\n\
            Comment: 6.3), there is no limit to the length of Armor Headers.  Care should\r\n\
            Comment: be taken that the Armor Headers are short enough to survive\r\n\
            Comment: transport.  One way to do this is to repeat an Armor Header key\r\n\
            Comment: multiple times with different values for each so that no one line is\r\n\
            Comment: overly long.\r\n\
            \r\n\
            iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAMAAAD04JH5AAAAM1BMVEUAk90NmN48\r\n\
            reaV0/F2xe0Ane284/f///8AjdsAmOb2+/7W7vro9fwno+Ku3fVStehcuumhrI4O\r\n\
            AAAFcklEQVR42u2b2ZLcIAxFAWEas///14bF7gav4G2qUtZDMnnI3IMQSAaBPidM\r\n\
            KClZMKmEOPg70MH/pxg4ojFCqPeGENaWghQPAUgguu+7LigPFv+JNDXidgAFBAex\r\n\
            Jeu7XlN5K4CkXh1tWN8hx24DUF5+HDsP9vuJ/xg6ZOUtAAL0IO/1MHEUTFwDBoD6\r\n\
            gMwgOgziegBGkrzXIdSomXcMJWhk6DsiLwYQ3vtJPgsz6UdPqXM0esNvCj5Av04w\r\n\
            lwKoNHwvD2pci87vAzwzjImlFn8XJlwIwHQa/ji3ficI2oiXS6CIxb6jlwEYnIbv\r\n\
            0uiZw5yjfashqAKAuOX54afRW1SjXkmA6vW1TMFYLV9FUAFgkj5RKRga5EMgwGkA\r\n\
            hjN9QG36qMfsJIDSyf9BX7hG+UCg1SkAQboYfzLuBe36PgzcKQCa1r9Z0+9TPXI8\r\n\
            DNBeAEZ9GueCz5J/j7U3HH7ccAFWhwFUCkAtwlyU+l2PHTDli0FfGfo8hNcZNtfi\r\n\
            NoD7ToCwvJS3k3woqV5HkAcBWJoAG2KBFxNP2FKthldqpY4cA0grIPIDLyZ1JayU\r\n\
            WwnInh0CgG50gMz3n61aA5YJNlywASB0+mWsDMDOit19s8EFaNcBpAyAbf01gs62\r\n\
            AwwO4PBhhTNFVe6aGpbNAGkPQljlE7C3s/88V7sXrAPYMQTzFdDXVJqkW8pJohEg\r\n\
            bYJ+E1K4ZiqLPQktTMIaOtpxJFZZBPa4rtinXX1SRDszQFi2Bexk1p8LFlbCWvSg\r\n\
            nTVAixqk9qvTddVzgHbmkWQR0JPazyjTV6+DNQCGcJTGjWV+EcGlB5Z3ELSRCDHG\r\n\
            5RigFuBDFgCWIxhtZmL0ArwAL8AL8AK8AC/AC/ACvAAvwP8FoE4DmDMAHFV6AKg3\r\n\
            cOG7aGqW1pqe/lf/lWkqP81IF2z5rKurtdnwtak+oiE7h98I7/8xk1+/SD0A0Goc\r\n\
            g2o5qr0YgGOq2u4LLgXw8rL1zug6AL4vfyMAD7fc+6ea9wCEK2RtV9pJ5L0AsZdC\r\n\
            E8rW1h3QuwD4qG3YhudpfznA0EESegfAyO1ZF45fBcDHzhWsB+WKphFJ+OysrxVg\r\n\
            HK/XdRQqhcfjS8zRCYAkrFPzzIGmLUFj08NBgCG8WsY7Hf5w8XoEwO9oFlJ4HdWX\r\n\
            bjx0bwfgiID6nDIBmK8dOO8BcG7Z56R572eXHm0A65VMtTHyk/eT2eQBjqg4LY8y\r\n\
            eQ2SNABwfdb7ZiIvPqZhCrg9F3sKCufHulDoagDO4ZS8pHgmHy6Ba2OAozP6wthC\r\n\
            fqwLWfUy5JidGXy+7rIGvNCGUgcwtG0dUodfW2Nqfvx9E7jaZHRYX0Hu+uj7zJNQ\r\n\
            m4yGtq3WefdjLyY+ZJD8F6VL8BqAA/qhofSrHrsrZ22nBlUC6EZ9IVM362ixy9vM\r\n\
            5nDQrwCgjeMXLHzQO2edN1grVqA+HZ/d/ZeL4YZ0fIOprA3rLwCKLsznAVTZhPo4\r\n\
            gJl0AT4MkJdDfwDAFjqQHwRgiw3QTwEoIMsdwI8ACFZUBA8DePW8IngYQBmnp+q+\r\n\
            JMJPAMSsiPlM3ZdEcLcHQl+v1TPxdFLGfC1+01YcnpoZoJaklx4LJ2XprIpcvBUr\r\n\
            A84SbxpjxOfSw0mZHZ+elR3xV3hASAbJ44X8ryIi+bs3elcuUMH9LvohmSbEUgpl\r\n\
            RTTpyL6jIhJCBRNi6cmhnHXkP1uSZScjf5ENJVnYEJ/zgKKYH9qKxZ3yTwEwio9n\r\n\
            w9MAq6XAIwCT7+NnARSjZPcV3C7AsRfMMRdrVPEG72oPhCc28Qn0QlK61gPqu+H6\r\n\
            v6SU8alrenxdq33cA6HM8zlHDxYuwzninDcpH/eAAM1LO3Ot1g4Qzrr766w5GSl2\r\n\
            sU0Ob/4BrGxXIweWt2UAAAAASUVORK5CYII=\r\n\
            =/u+x\r\n\
            -----END PGP MESSAGE-----\r\n\
        ";

        assert_eq!(buffer.len(), expected.len());

        // HashMap elements have an arbitrary order, hence sort:
        buffer.sort();
        expected.sort();

        assert_eq!(buffer, expected);
    }
}
