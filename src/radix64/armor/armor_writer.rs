// Example:
//
// -----BEGIN PGP MESSAGE-----
// Version: OpenPrivacy 0.99
//
// yDgBO22WxBHv7O8X7O/jygAEzol56iUKiXmV+XmpCtmpqQUKiQrFqclFqUDBovzS
// vBSFjNSiVHsuAA==
// =njUN
// -----END PGP MESSAGE-----

use std::collections::HashMap;

use super::armor_checksums::ArmorChecksum;
use super::armor_data_types::ArmorDataType;
use super::armor_headers::ArmorHeader;
use super::super::coding::Radix64;
use super::super::coding::LINE_ENDING;

// 6.2.  Forming ASCII Armor
// https://tools.ietf.org/html/rfc4880#section-6.2
pub struct ArmorWriter {
    pub data_type: Option<ArmorDataType>,
    headers: HashMap<ArmorHeader, Vec<String>>,
    data: Option<Radix64>,
    checksum: Option<ArmorChecksum>,
}

impl ArmorWriter {
    pub fn new() -> Self {
        Self {
            data_type: None,
            headers: HashMap::new(),
            data: None,
            checksum: None,
        }
    }

    pub fn add_header(&mut self, key: ArmorHeader, value: &str) {
        self.headers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(value.to_string())
        ;
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.checksum = Some(ArmorChecksum::new(&data.as_slice()));
        self.data = Some(Radix64::encode(data));
    }

    /// Does not err if provided data is incomplete
    pub fn write_unsafe(&self) -> String {
        let mut output = String::new();

        let mut header_line = String::new();
        let mut tail_line = String::new();

        if let Some(data_type) = &self.data_type {
            let header_and_tail = self.construct_header_and_tail_line(&data_type);
            header_line = header_and_tail.0;
            tail_line = header_and_tail.1;
        }

        output.push_str(&header_line);

        for (key, values) in &self.headers {
            for value in values {
                let header = &format!("{}: {}{}", key.to_str(), value, LINE_ENDING);
                output.push_str(header);
            }
        }

        output.push_str(LINE_ENDING);

        if let Some(data) = &self.data {
            output.push_str(&data.encoded);
            output.push_str(LINE_ENDING);
        }

        if let Some(checksum) = &self.checksum {
            output.push_str(&checksum.get());
            output.push_str(LINE_ENDING);
        }

        output.push_str(&tail_line);

        output
    }

    fn construct_header_and_tail_line(&self, data_type: &ArmorDataType) -> (String, String) {
        let header_line;
        let tail_line;

        match data_type {
            ArmorDataType::PgpMessage => {
                header_line = String::from("BEGIN PGP MESSAGE");
                tail_line = String::from("END PGP MESSAGE");
            },
            ArmorDataType::PgpPublicKeyBlock => {
                header_line = String::from("BEGIN PGP PUBLIC KEY BLOCK");
                tail_line = String::from("END PGP PUBLIC KEY BLOCK");
            },
            ArmorDataType::PgpPrivateKeyBlock => {
                header_line = String::from("BEGIN PGP PRIVATE KEY BLOCK");
                tail_line = String::from("END PGP PRIVATE KEY BLOCK");
            },
            ArmorDataType::PgpMessagePartXy(x, y) => {
                header_line = format!("BEGIN PGP MESSAGE, PART {}/{}", x, y);
                tail_line = format!("END PGP MESSAGE, PART {}/{}", x, y);
            },
            ArmorDataType::PgpMessagePartX(x) => {
                header_line = format!("BEGIN PGP MESSAGE, PART {}", x);
                tail_line = format!("END PGP MESSAGE, PART {}", x);
            },
            ArmorDataType::PgpSignature => {
                header_line = String::from("BEGIN PGP SIGNATURE");
                tail_line = String::from("END dummy"); // Not used
            },
            ArmorDataType::PgpSignedMessage => {
                header_line = String::from("BEGIN PGP SIGNED MESSAGE");
                tail_line = String::from("END PGP SIGNED MESSAGE");
            },
            ArmorDataType::PgpArmoredFile => {
                header_line = String::from("BEGIN PGP ARMORED FILE");
                tail_line = String::from("END PGP ARMORED FILE");
            },
            ArmorDataType::PgpSecretKeyBlock => {
                header_line = String::from("BEGIN PGP SECRET KEY BLOCK");
                tail_line = String::from("END PGP SECRET KEY BLOCK");
            },
        }

        (
            format!("-----{}-----{}", header_line, LINE_ENDING),
            format!("-----{}-----", tail_line)
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

    #[test]
    fn write_header_footer_lines_for_pgp_message() {
        let mut armor = ArmorWriter::new();
        armor.data_type = Some(ArmorDataType::PgpMessage);

        assert_eq!(armor.write_unsafe(), "\
            -----BEGIN PGP MESSAGE-----\r\n\
            \r\n\
            -----END PGP MESSAGE-----\
        ");
    }

    // #[test]
    // fn write_header_footer_lines_for_pgp_signature() {
    //     let mut armor = ArmorWriter::new();
    //     armor.data_type = Some(ArmorDataType::PgpSignature);
    // }

    #[test]
    fn write_header_footer_lines_for_pgp_message_part_x() {
        let mut armor = ArmorWriter::new();
        armor.data_type = Some(ArmorDataType::PgpMessagePartX(2));

        assert_eq!(armor.write_unsafe(), "\
            -----BEGIN PGP MESSAGE, PART 2-----\r\n\
            \r\n\
            -----END PGP MESSAGE, PART 2-----\
        ");
    }

    #[test]
    fn write_header_footer_lines_for_pgp_message_part_xy() {
        let mut armor = ArmorWriter::new();
        armor.data_type = Some(ArmorDataType::PgpMessagePartXy(2, 3));

        assert_eq!(armor.write_unsafe(), "\
            -----BEGIN PGP MESSAGE, PART 2/3-----\r\n\
            \r\n\
            -----END PGP MESSAGE, PART 2/3-----\
        ");
    }

    #[test]
    fn write_single_header() {
        let mut armor = ArmorWriter::new();
        armor.add_header(ArmorHeader::Version, "OpenPrivacy 0.99");

        assert_eq!(armor.write_unsafe(), "\
            Version: OpenPrivacy 0.99\r\n\
            \r\n\
        ");
    }

    #[test]
    fn write_multiple_headers_with_single_key() {
        let mut armor = ArmorWriter::new();
        armor.add_header(ArmorHeader::Comment, "Comment on first line");
        armor.add_header(ArmorHeader::Comment, "And also on second line");

        assert_eq!(armor.write_unsafe(), "\
            Comment: Comment on first line\r\n\
            Comment: And also on second line\r\n\
            \r\n\
        ");
    }

    #[test]
    fn write_multiple_headers_with_multiple_keys() {
        let mut armor = ArmorWriter::new();
        armor.add_header(ArmorHeader::Comment, "Comment on first line");
        armor.add_header(ArmorHeader::Comment, "And also on second line");
        armor.add_header(ArmorHeader::Charset, "UTF-8");

        let armor = armor.write_unsafe();

        let mut lines: Vec<&str> = armor.lines().map(From::from).collect();

        let mut expected = vec![
            "Comment: Comment on first line",
            "Comment: And also on second line",
            "Charset: UTF-8",
            "",
        ];

        assert_eq!(lines.len(), expected.len());

        // Ordering of HashMap elements is arbitrary, hence the sorting:
        lines.sort();
        expected.sort();
        assert_eq!(lines, expected);
    }

    #[test]
    fn set_data() {
        let mut armor = ArmorWriter::new();
        armor.set_data(b"Hello".to_vec());

        // Encoded data and calculated checksum
        assert_eq!(armor.write_unsafe(), "\
            \r\n\
            SGVsbG8=\r\n\
            =EHJM\r\n\
        ");
    }

    #[test]
    fn everything_with_binary_data() {
        let mut armor = ArmorWriter::new();
            armor.data_type = Some(ArmorDataType::PgpMessage);
            armor.add_header(ArmorHeader::Version, "OpenPrivacy 0.99");
            armor.add_header(ArmorHeader::Comment, "Note that some transport methods are sensitive to line length.  While");
            armor.add_header(ArmorHeader::Comment, "there is a limit of 76 characters for the Radix-64 data (Section");
            armor.add_header(ArmorHeader::Comment, "6.3), there is no limit to the length of Armor Headers.  Care should");
            armor.add_header(ArmorHeader::Comment, "be taken that the Armor Headers are short enough to survive");
            armor.add_header(ArmorHeader::Comment, "transport.  One way to do this is to repeat an Armor Header key");
            armor.add_header(ArmorHeader::Comment, "multiple times with different values for each so that no one line is");
            armor.add_header(ArmorHeader::Comment, "overly long.");

            let data_bytes = fs::read("tests/resources/gnupg-icon.png").unwrap();
            armor.set_data(data_bytes);
        let armor = armor.write_unsafe();

        let mut lines: Vec<&str> = armor.lines().map(From::from).collect();

        let mut expected: Vec<&str> = vec![
            "-----BEGIN PGP MESSAGE-----",
            "Version: OpenPrivacy 0.99",
            "Comment: Note that some transport methods are sensitive to line length.  While",
            "Comment: there is a limit of 76 characters for the Radix-64 data (Section",
            "Comment: 6.3), there is no limit to the length of Armor Headers.  Care should",
            "Comment: be taken that the Armor Headers are short enough to survive",
            "Comment: transport.  One way to do this is to repeat an Armor Header key",
            "Comment: multiple times with different values for each so that no one line is",
            "Comment: overly long.",
            "",
            "iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAMAAAD04JH5AAAAM1BMVEUAk90NmN48",
            "reaV0/F2xe0Ane284/f///8AjdsAmOb2+/7W7vro9fwno+Ku3fVStehcuumhrI4O",
            "AAAFcklEQVR42u2b2ZLcIAxFAWEas///14bF7gav4G2qUtZDMnnI3IMQSAaBPidM",
            "KClZMKmEOPg70MH/pxg4ojFCqPeGENaWghQPAUgguu+7LigPFv+JNDXidgAFBAex",
            "Jeu7XlN5K4CkXh1tWN8hx24DUF5+HDsP9vuJ/xg6ZOUtAAL0IO/1MHEUTFwDBoD6",
            "gMwgOgziegBGkrzXIdSomXcMJWhk6DsiLwYQ3vtJPgsz6UdPqXM0esNvCj5Av04w",
            "lwKoNHwvD2pci87vAzwzjImlFn8XJlwIwHQa/ji3ficI2oiXS6CIxb6jlwEYnIbv",
            "0uiZw5yjfashqAKAuOX54afRW1SjXkmA6vW1TMFYLV9FUAFgkj5RKRga5EMgwGkA",
            "hjN9QG36qMfsJIDSyf9BX7hG+UCg1SkAQboYfzLuBe36PgzcKQCa1r9Z0+9TPXI8",
            "DNBeAEZ9GueCz5J/j7U3HH7ccAFWhwFUCkAtwlyU+l2PHTDli0FfGfo8hNcZNtfi",
            "NoD7ToCwvJS3k3woqV5HkAcBWJoAG2KBFxNP2FKthldqpY4cA0grIPIDLyZ1JayU",
            "WwnInh0CgG50gMz3n61aA5YJNlywASB0+mWsDMDOit19s8EFaNcBpAyAbf01gs62",
            "AwwO4PBhhTNFVe6aGpbNAGkPQljlE7C3s/88V7sXrAPYMQTzFdDXVJqkW8pJohEg",
            "bYJ+E1K4ZiqLPQktTMIaOtpxJFZZBPa4rtinXX1SRDszQFi2Bexk1p8LFlbCWvSg",
            "nTVAixqk9qvTddVzgHbmkWQR0JPazyjTV6+DNQCGcJTGjWV+EcGlB5Z3ELSRCDHG",
            "5RigFuBDFgCWIxhtZmL0ArwAL8AL8AK8AC/AC/ACvAAvwP8FoE4DmDMAHFV6AKg3",
            "cOG7aGqW1pqe/lf/lWkqP81IF2z5rKurtdnwtak+oiE7h98I7/8xk1+/SD0A0Goc",
            "g2o5qr0YgGOq2u4LLgXw8rL1zug6AL4vfyMAD7fc+6ea9wCEK2RtV9pJ5L0AsZdC",
            "E8rW1h3QuwD4qG3YhudpfznA0EESegfAyO1ZF45fBcDHzhWsB+WKphFJ+OysrxVg",
            "HK/XdRQqhcfjS8zRCYAkrFPzzIGmLUFj08NBgCG8WsY7Hf5w8XoEwO9oFlJ4HdWX",
            "bjx0bwfgiID6nDIBmK8dOO8BcG7Z56R572eXHm0A65VMtTHyk/eT2eQBjqg4LY8y",
            "eQ2SNABwfdb7ZiIvPqZhCrg9F3sKCufHulDoagDO4ZS8pHgmHy6Ba2OAozP6wthC",
            "fqwLWfUy5JidGXy+7rIGvNCGUgcwtG0dUodfW2Nqfvx9E7jaZHRYX0Hu+uj7zJNQ",
            "m4yGtq3WefdjLyY+ZJD8F6VL8BqAA/qhofSrHrsrZ22nBlUC6EZ9IVM362ixy9vM",
            "5nDQrwCgjeMXLHzQO2edN1grVqA+HZ/d/ZeL4YZ0fIOprA3rLwCKLsznAVTZhPo4",
            "gJl0AT4MkJdDfwDAFjqQHwRgiw3QTwEoIMsdwI8ACFZUBA8DePW8IngYQBmnp+q+",
            "JMJPAMSsiPlM3ZdEcLcHQl+v1TPxdFLGfC1+01YcnpoZoJaklx4LJ2XprIpcvBUr",
            "A84SbxpjxOfSw0mZHZ+elR3xV3hASAbJ44X8ryIi+bs3elcuUMH9LvohmSbEUgpl",
            "RTTpyL6jIhJCBRNi6cmhnHXkP1uSZScjf5ENJVnYEJ/zgKKYH9qKxZ3yTwEwio9n",
            "w9MAq6XAIwCT7+NnARSjZPcV3C7AsRfMMRdrVPEG72oPhCc28Qn0QlK61gPqu+H6",
            "v6SU8alrenxdq33cA6HM8zlHDxYuwzninDcpH/eAAM1LO3Ot1g4Qzrr766w5GSl2",
            "sU0Ob/4BrGxXIweWt2UAAAAASUVORK5CYII=",
            "=/u+x",
            "-----END PGP MESSAGE-----",
        ];

        assert_eq!(lines.len(), expected.len());

        // Ordering of HashMap elements is arbitrary, hence the sorting:
        lines.sort();
        expected.sort();
        assert_eq!(lines, expected);
    }
}
