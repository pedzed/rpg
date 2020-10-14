#[derive(Debug, PartialEq)]
pub struct ArmorHeaderError(String);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ArmorHeader {
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

impl ArmorHeader {
    pub fn from_str(input: &str) -> Result<Self, ArmorHeaderError> {
        match input {
            "Version" => Ok(Self::Version),
            "Comment" => Ok(Self::Comment),
            "MessageID" => Ok(Self::MessageId),
            "Hash" => Ok(Self::Hash),
            "Charset" => Ok(Self::Charset),
            x => Err(ArmorHeaderError(format!("Input `{}` not valid.", x))),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Version => "Version",
            Self::Comment => "Comment",
            Self::MessageId => "MessageID",
            Self::Hash => "Hash",
            Self::Charset => "Charset",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_to_enum_for_messageid() {
        assert_eq!(ArmorHeader::from_str("MessageID").unwrap(), ArmorHeader::MessageId);
    }

    #[test]
    fn str_to_enum_fails_for_invalid_input() {
        assert_eq!(
            ArmorHeader::from_str("InvalidInput"),
            Err(ArmorHeaderError(String::from("Input `InvalidInput` not valid.")))
        );
    }

    #[test]
    fn enum_to_str_for_messageid() {
        assert_eq!(ArmorHeader::MessageId.to_str(), "MessageID");
    }
}
