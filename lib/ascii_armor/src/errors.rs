/// ASCII Armor errors
#[derive(Debug, PartialEq)]
pub enum ArmorError {
    UnknownDataHeader(String),
    UnknownDataType(String),
    InvalidChecksum(String),

    // NOTE: Consider moving to own ArmorReaderError enum
    ReaderUnknownChecksum,
}

impl std::fmt::Display for ArmorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArmorError::UnknownDataHeader(ref header) => write!(f, "Unknown armor data header `{}`.", header),
            ArmorError::UnknownDataType(ref data_type) => write!(f, "Unknown armor data type `{}`.", data_type),
            ArmorError::InvalidChecksum(ref checksum) => write!(f, "Invalid checksum `{}`.", checksum),
            ArmorError::ReaderUnknownChecksum => write!(f, "Cannot find valid checksum in ASCII Armor."),
        }
    }
}

impl std::error::Error for ArmorError {}

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
}
