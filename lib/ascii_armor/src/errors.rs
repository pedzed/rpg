/// ASCII Armor errors
#[derive(Debug, PartialEq)]
pub enum ArmorError {
    UnknownDataHeader(String),
    UnknownDataType(String),
    InvalidChecksum(String),
    FailedIo(String),

    // NOTE: Consider moving to own ArmorReaderError enum
    ReaderUnknownDataType,
    ReaderUnknownChecksum,
    ReaderFailedDecoding(String),
}

impl std::fmt::Display for ArmorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArmorError::UnknownDataHeader(ref header) => write!(f, "Unknown armor data header `{}`.", header),
            ArmorError::UnknownDataType(ref data_type) => write!(f, "Unknown armor data type `{}`.", data_type),
            ArmorError::InvalidChecksum(ref checksum) => write!(f, "Invalid checksum `{}`.", checksum),
            ArmorError::FailedIo(ref error) => write!(f, "{}", error.to_owned()),
            ArmorError::ReaderUnknownDataType => write!(f, "Cannot find valid data type in ASCII Armor."),
            ArmorError::ReaderUnknownChecksum => write!(f, "Cannot find valid checksum in ASCII Armor."),
            ArmorError::ReaderFailedDecoding(ref error) => write!(f, "{}", error.to_owned()),
        }
    }
}

impl std::error::Error for ArmorError {}

impl From<std::io::Error> for ArmorError {
    fn from(e: std::io::Error) -> Self {
        Self::FailedIo(e.to_string())
    }
}

impl From<base64::DecoderError> for ArmorError {
    fn from(e: base64::DecoderError) -> Self {
        Self::ReaderFailedDecoding(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::ArmorError;

    #[test]
    fn unknown_data_header_error_formats_correctly() {
        assert_eq!(
            ArmorError::UnknownDataHeader("InvalidInput".into()).to_string(),
            "Unknown armor data header `InvalidInput`."
        );
    }

    #[test]
    fn unknown_data_type_error_formats_correctly() {
        assert_eq!(
            ArmorError::UnknownDataType("InvalidInput".into()).to_string(),
            "Unknown armor data type `InvalidInput`."
        );
    }

    #[test]
    fn invalid_checksum_error_formats_correctly() {
        assert_eq!(
            ArmorError::InvalidChecksum("InvalidInput".into()).to_string(),
            "Invalid checksum `InvalidInput`."
        );
    }

    #[test]
    fn io_error_to_armor_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "Fail.");
        assert_eq!(ArmorError::from(io_error), ArmorError::FailedIo(String::from("Fail.")));
    }

    #[test]
    fn decoder_error_to_armor_error() {
        let decoder_error = base64::DecoderError::UnexpectedChar(0x01);
        assert_eq!(
            ArmorError::from(decoder_error),
            ArmorError::ReaderFailedDecoding(format!("Unexpected character `{}` found.", 0x01))
        );
    }
}
