// Example:
//
// -----BEGIN PGP MESSAGE-----
// Version: OpenPrivacy 0.99
//
// yDgBO22WxBHv7O8X7O/jygAEzol56iUKiXmV+XmpCtmpqQUKiQrFqclFqUDBovzS
// vBSFjNSiVHsuAA==
// =njUN
// -----END PGP MESSAGE-----

use std::fs;
use std::collections::HashMap;

use super::crc24::Crc24;
use super::coding::Radix64;
use super::coding::LINE_ENDING;

// https://tools.ietf.org/html/rfc4880#section-6.2
pub struct Armor {
    data_type: ArmorDataType,
    header_line: String,
    headers: HashMap<ArmorHeaderKey, Vec<String>>,
    data: String,
    checksum: String,
    tail_line: String,
}

pub enum ArmorDataType {
    /// Used for signed, encrypted, or compressed files.
    PgpMessage,

    /// Used for armoring public keys.
    PgpPublicKeyBlock,

    /// Used for armoring private keys.
    PgpPrivateKeyBlock,

    /// Used for multi-part messages, where the armor is split amongst Y
    /// parts, and this is the Xth part out of Y.
    PgpMessagePartXY(u8, u8),

    /// Used for multi-part messages, where this is the Xth part of an
    /// unspecified number of parts.  Requires the MESSAGE-ID Armor
    /// Header to be used.
    PgpMessagePartX(u8),

    /// Used for detached signatures, OpenPGP/MIME signatures, and
    /// cleartext signatures.  Note that PGP 2.x uses BEGIN PGP MESSAGE
    /// for detached signatures.
    PgpSignature,

    // https://tools.ietf.org/html/rfc4880#section-7
    PgpSignedMessage,

    // NOTE: GnuPG specific
    // https://github.com/gpg/gnupg/blob/master/g10/armor.c#L84-L93
    PgpArmoredFile,

    /// When exporting a private key, PGP 2.x generates the header "BEGIN
    /// PGP SECRET KEY BLOCK" instead of "BEGIN PGP PRIVATE KEY BLOCK".
    /// All previous versions ignore the implied data type, and look
    /// directly at the packet data type.
    PgpSecretKeyBlock,
}

#[derive(PartialEq, Eq, Hash)]
pub enum ArmorHeaderKey {
    // "Version", which states the OpenPGP implementation and version
    // used to encode the message.
    Version,

    // "Comment", a user-defined comment.  OpenPGP defines all text to
    // be in UTF-8.  A comment may be any UTF-8 string.  However, the
    // whole point of armoring is to provide seven-bit-clean data.
    // Consequently, if a comment has characters that are outside the
    // US-ASCII range of UTF, they may very well not survive transport.
    Comment,

    // "MessageID", a 32-character string of printable characters.  The
    // string must be the same for all parts of a multi-part message
    // that uses the "PART X" Armor Header.  MessageID strings should be
    // unique enough that the recipient of the mail can associate all
    // the parts of a message with each other.  A good checksum or
    // cryptographic hash function is sufficient.
    //
    // The MessageID SHOULD NOT appear unless it is in a multi-part
    // message.  If it appears at all, it MUST be computed from the
    // finished (encrypted, signed, etc.) message in a deterministic
    // fashion, rather than contain a purely random value.  This is to
    // allow the legitimate recipient to determine that the MessageID
    // cannot serve as a covert means of leaking cryptographic key
    // information.
    MessageId,

    // "Hash", a comma-separated list of hash algorithms used in this
    // message.  This is used only in cleartext signed messages.
    Hash,

    // "Charset", a description of the character set that the plaintext
    // is in.  Please note that OpenPGP defines text to be in UTF-8.  An
    // implementation will get best results by translating into and out
    // of UTF-8.  However, there are many instances where this is easier
    // said than done.  Also, there are communities of users who have no
    // need for UTF-8 because they are all happy with a character set
    // like ISO Latin-5 or a Japanese character set.  In such instances,
    // an implementation MAY override the UTF-8 default by using this
    // header key.  An implementation MAY implement this key and any
    // translations it cares to; an implementation MAY ignore it and
    // assume all text is UTF-8.
    Charset,
}

impl Armor {
    pub fn new(data_type: ArmorDataType) -> Armor {
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
            ArmorDataType::PgpMessagePartXY(x, y) => {
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

        Armor {
            data_type,
            header_line,
            headers: HashMap::new(),
            data: String::new(),
            checksum: String::new(),
            tail_line,
        }
    }

    pub fn add_header(&mut self, key: ArmorHeaderKey, value: &str) {
        self.headers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(value.to_string())
        ;
    }

    pub fn armorize(&self) -> String {
        let mut armor = String::new();
        armor.push_str(&format!("-----{}-----{}", self.header_line, LINE_ENDING));

        for (key, values) in &self.headers {
            for value in values {
                let header = &format!("{}: {}{}", self.header_key_as_str(key), value, LINE_ENDING);
                armor.push_str(header);
            }
        }

        armor.push_str(LINE_ENDING);

        armor.push_str(&self.data);
        armor.push_str(LINE_ENDING);

        armor.push_str(&self.checksum);
        armor.push_str(LINE_ENDING);

        armor.push_str(&format!("-----{}-----", self.tail_line));

        armor
    }

    fn header_key_as_str(&self, key: &ArmorHeaderKey) -> &str {
        match key {
            ArmorHeaderKey::Version => "Version",
            ArmorHeaderKey::Comment => "Comment",
            ArmorHeaderKey::MessageId => "MessageID",
            ArmorHeaderKey::Hash => "Hash",
            ArmorHeaderKey::Charset => "Charset",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_footer_lines_for_pgp_message() {
        let armor = Armor::new(ArmorDataType::PgpMessage);
        assert_eq!(armor.header_line, "BEGIN PGP MESSAGE");
        assert_eq!(armor.tail_line, "END PGP MESSAGE");
    }

    #[test]
    fn header_footer_lines_for_pgp_signature() {
        let armor = Armor::new(ArmorDataType::PgpSignature);
        assert_eq!(armor.header_line, "BEGIN PGP SIGNATURE");
        assert_eq!(armor.tail_line, "END dummy");
    }

    #[test]
    fn header_footer_lines_for_pgp_message_part_xy() {
        let armor = Armor::new(ArmorDataType::PgpMessagePartXY(2, 3));
        assert_eq!(armor.header_line, "BEGIN PGP MESSAGE, PART 2/3");
        assert_eq!(armor.tail_line, "END PGP MESSAGE, PART 2/3");
    }

    #[test]
    fn header_footer_lines_for_pgp_message_part_x() {
        let armor = Armor::new(ArmorDataType::PgpMessagePartX(2));
        assert_eq!(armor.header_line, "BEGIN PGP MESSAGE, PART 2");
        assert_eq!(armor.tail_line, "END PGP MESSAGE, PART 2");
    }

    #[test]
    fn headerless() {
        let armor = Armor::new(ArmorDataType::PgpMessage);
        assert_eq!(armor.headers.len(), 0);
    }

    #[test]
    fn header_version() {
        let mut armor = Armor::new(ArmorDataType::PgpMessage);
        armor.add_header(ArmorHeaderKey::Version, "OpenPrivacy 0.99");

        let header_values = &armor.headers.get(&ArmorHeaderKey::Version).unwrap();

        assert_eq!(header_values[0], "OpenPrivacy 0.99");
        assert_eq!(armor.headers.len(), 1);
    }

    #[test]
    fn can_set_data() {
        let mut armor = Armor::new(ArmorDataType::PgpMessage);

        let radix64 = Radix64::encode(b"Hello".to_vec());
        armor.data = radix64.encoded;

        let expected = "SGVsbG8=";
        assert_eq!(armor.data, expected);
    }

    #[test]
    fn can_set_checksum() {
        let mut armor = Armor::new(ArmorDataType::PgpMessage);

        let crc = Crc24::new(b"Hello");
        armor.checksum = crc.radix64_checksum();

        let expected = "=EHJM";
        assert_eq!(armor.checksum, expected);
    }

    #[test]
    fn armorize_with_real_binary_data() {
        let mut armor = Armor::new(ArmorDataType::PgpMessage);
        armor.add_header(ArmorHeaderKey::Version, "OpenPrivacy 0.99");
        armor.add_header(ArmorHeaderKey::Comment, "Note that some transport methods are sensitive to line length.  While");
        armor.add_header(ArmorHeaderKey::Comment, "there is a limit of 76 characters for the Radix-64 data (Section");
        armor.add_header(ArmorHeaderKey::Comment, "6.3), there is no limit to the length of Armor Headers.  Care should");
        armor.add_header(ArmorHeaderKey::Comment, "be taken that the Armor Headers are short enough to survive");
        armor.add_header(ArmorHeaderKey::Comment, "transport.  One way to do this is to repeat an Armor Header key");
        armor.add_header(ArmorHeaderKey::Comment, "multiple times with different values for each so that no one line is");
        armor.add_header(ArmorHeaderKey::Comment, "overly long.");

        let data_bytes = fs::read("tests/resources/gnupg-icon.png").unwrap();

        let crc = Crc24::new(data_bytes.as_slice());
        armor.checksum = crc.radix64_checksum();

        armor.data = Radix64::encode(data_bytes).encoded;

        let armorized = armor.armorize();
        let mut lines: Vec<&str> = armorized.lines().map(From::from).collect();

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
            "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",
            "sU0Ob/4BrGxXIweWt2UAAAAASUVORK5CYII=",
            "=/u+x",
            "-----END PGP MESSAGE-----",
        ];

        assert_eq!(lines.sort(), expected.sort());
    }
}
