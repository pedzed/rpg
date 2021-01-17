use crate::ArmorError;

/// ASCII Armor data types
///
/// Useful for reading/writing ASCII Armor headers and footers.
#[derive(Debug, PartialEq)]
pub enum ArmorDataType {
    /// Used for signed, encrypted, or compressed files.
    PgpMessage,

    /// Used for armoring public keys.
    PgpPublicKeyBlock,

    /// Used for armoring private keys.
    PgpPrivateKeyBlock,

    /// Used for multi-part messages, where the armor is split amongst Y
    /// parts, and this is the Xth part out of Y.
    PgpMessagePartXy(usize, usize),

    /// Used for multi-part messages, where this is the Xth part of an
    /// unspecified number of parts.  Requires the MESSAGE-ID Armor
    /// Header to be used.
    PgpMessagePartX(usize),

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

impl ArmorDataType {
    /// Get the enum variant based on a given string slice.
    ///
    /// Useful when reading ASCII Armor.
    ///
    /// # Examples
    /// ```rust
    /// use ascii_armor::ArmorDataType;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// assert_eq!(ArmorDataType::from_str("PGP MESSAGE")?, ArmorDataType::PgpMessage);
    /// assert_eq!(ArmorDataType::from_str("PGP MESSAGE, PART 1/3")?, ArmorDataType::PgpMessagePartXy(1, 3));
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_str(input: &str) -> Result<Self, ArmorError> {
        match input {
            "PGP MESSAGE" => Ok(ArmorDataType::PgpMessage),
            "PGP PUBLIC KEY BLOCK" => Ok(ArmorDataType::PgpPublicKeyBlock),
            "PGP PRIVATE KEY BLOCK" => Ok(ArmorDataType::PgpPrivateKeyBlock),
            "PGP SIGNATURE" => Ok(ArmorDataType::PgpSignature),
            "PGP SIGNED MESSAGE" => Ok(ArmorDataType::PgpSignedMessage),
            "PGP ARMORED FILE" => Ok(ArmorDataType::PgpArmoredFile),
            "PGP SECRET KEY BLOCK" => Ok(ArmorDataType::PgpSecretKeyBlock),
            _ => {
                if input.starts_with("PGP MESSAGE, PART ") {
                    return Self::parse_pgp_message_part(input);
                }

                Err(ArmorError::UnknownDataType(input.into()))
            },
        }
    }

    fn parse_pgp_message_part(input: &str) -> Result<Self, ArmorError> {
        let parts: Vec<&str> = input["PGP MESSAGE, PART ".len()..]
            .trim()
            .split("/")
            .collect()
        ;

        match parts.len() {
            1 => {
                let x = parts[0].parse::<usize>();

                if x.is_ok() {
                    return Ok(ArmorDataType::PgpMessagePartX(x.unwrap()))
                }
            },
            2 => {
                let x = parts[0].parse::<usize>();
                let y = parts[1].parse::<usize>();

                if x.is_ok() && y.is_ok() {
                    return Ok(ArmorDataType::PgpMessagePartXy(x.unwrap(), y.unwrap()))
                }
            },
            _ => {
                return Err(ArmorError::UnknownDataType(input.into()));
            }
        }

        Err(ArmorError::UnknownDataType(input.into()))
    }

    /// Generate a string based on the enum variant.
    ///
    /// Useful when writing ASCII Armor.
    ///
    /// # Examples
    /// ```rust
    /// use ascii_armor::ArmorDataType;
    ///
    /// assert_eq!(ArmorDataType::PgpMessagePartX(2).to_string(), String::from("PGP MESSAGE, PART 2"));
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            Self::PgpMessage => String::from("PGP MESSAGE"),
            Self::PgpPublicKeyBlock => String::from("PGP PUBLIC KEY BLOCK"),
            Self::PgpPrivateKeyBlock => String::from("PGP PRIVATE KEY BLOCK"),
            Self::PgpMessagePartX(x) => format!("PGP MESSAGE, PART {}", x),
            Self::PgpMessagePartXy(x, y) => format!("PGP MESSAGE, PART {}/{}", x, y),
            Self::PgpSignature => String::from("PGP SIGNATURE"),
            Self::PgpSignedMessage => String::from("PGP SIGNED MESSAGE"),
            Self::PgpArmoredFile => String::from("PGP ARMORED FILE"),
            Self::PgpSecretKeyBlock => String::from("PGP SECRET KEY BLOCK"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_to_enum_fails_for_invalid_input() {
        assert_eq!(
            ArmorDataType::from_str("InvalidInput"),
            Err(ArmorError::UnknownDataType("InvalidInput".into()))
        );
    }

    #[test]
    fn str_to_enum_for_pgpmessage() {
        assert_eq!(
            ArmorDataType::from_str("PGP MESSAGE").unwrap(),
            ArmorDataType::PgpMessage
        );
    }

    #[test]
    fn str_to_enum_for_pgpmessagepartx() {
        assert_eq!(
            ArmorDataType::from_str("PGP MESSAGE, PART 2").unwrap(),
            ArmorDataType::PgpMessagePartX(2)
        );
    }

    #[test]
    fn str_to_enum_for_pgpmessagepartxy() {
        assert_eq!(
            ArmorDataType::from_str("PGP MESSAGE, PART 2/3").unwrap(),
            ArmorDataType::PgpMessagePartXy(2, 3)
        );
    }

    #[test]
    fn enum_to_string_for_pgpmessage() {
        assert_eq!(ArmorDataType::PgpMessage.to_string(), "PGP MESSAGE");
    }

    #[test]
    fn enum_to_string_for_pgpmessagepartx() {
        assert_eq!(ArmorDataType::PgpMessagePartX(2).to_string(), "PGP MESSAGE, PART 2");
    }

    #[test]
    fn enum_to_string_for_pgpmessagepartxy() {
        assert_eq!(ArmorDataType::PgpMessagePartXy(2, 3).to_string(), "PGP MESSAGE, PART 2/3");
    }
}
