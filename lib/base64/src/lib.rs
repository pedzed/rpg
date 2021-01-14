mod tables;
mod encoder;
mod decoder;

// pub(crate) type Error = Box<dyn std::error::Error>;
pub(crate) type U6 = u8;

const OCTETS_PER_BLOCK: usize = 3;
const SEXTETS_PER_BLOCK: usize = 4;

const PAD_BYTE: u8 = b'=';

/// Encode octets to sextets using base64.
///
/// Base64 encoding is useful for safe binary data transmission.
///
/// # Examples
/// ```rust
/// assert_eq!(base64::encode(b"Hello World!"), b"SGVsbG8gV29ybGQh");
/// assert_eq!(base64::encode(&[0x48, 0x65, 0x6C, 0x6C, 0x6F]), b"SGVsbG8=");
/// ```
pub fn encode(input: &[u8]) -> Vec<U6> {
    encoder::encode(input)
}

/// Decode base64 encoded data.
///
/// # Examples
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert_eq!(base64::decode(b"SGVsbG8gV29ybGQh")?, b"Hello World!");
/// assert_eq!(base64::decode(b"SGVsbG8=")?, &[0x48, 0x65, 0x6C, 0x6C, 0x6F]);
/// # Ok(())
/// # }
/// ```
pub fn decode(input: &[u8]) -> Result<Vec<u8>, DecoderError> {
    decoder::decode(input)
}

#[derive(Debug)]
pub enum DecoderError {
    UnexpectedChar(u8),
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DecoderError::UnexpectedChar(ref x) => write!(f, "Unexpected character `{}` found.", x),
        }
    }
}

impl std::error::Error for DecoderError {}
