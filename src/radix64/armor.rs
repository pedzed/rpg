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

// https://tools.ietf.org/html/rfc4880#section-6.2
pub struct Armor {
    data_type: ArmorDataType,
    header_line: String,
    headers: HashMap<ArmorHeaderKey, String>,
    data: Vec<String>,
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
            data: vec![],
            checksum: String::new(),
            tail_line,
        }
    }

    pub fn set_header(&mut self, key: ArmorHeaderKey, value: &str) {
        self.headers.insert(key, value.to_string());
    }

    pub fn armorize(&self) -> String {
        let mut armor = String::new();
        armor.push_str(&format!("-----{}-----\n", self.header_line));

        for (key, val) in &self.headers {
            let header = &format!("{}: {}\n", self.header_key_as_str(key), val);
            armor.push_str(header);
        }

        armor.push_str("\n");

        // TODO: Add Radix-64 encoding
        // TODO: Add CRC-24 checksum

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
        armor.set_header(ArmorHeaderKey::Version, "OpenPrivacy 0.99");

        assert_eq!(armor.headers.get(&ArmorHeaderKey::Version).unwrap(), "OpenPrivacy 0.99");
        assert_eq!(armor.headers.len(), 1);
    }

    #[test]
    fn armorize_with_version_header() {
        let mut armor = Armor::new(ArmorDataType::PgpMessage);
        armor.set_header(ArmorHeaderKey::Version, "OpenPrivacy 0.99");

        assert_eq!(
            armor.armorize(),
            "\
-----BEGIN PGP MESSAGE-----
Version: OpenPrivacy 0.99

-----END PGP MESSAGE-----"
        );
    }
}
