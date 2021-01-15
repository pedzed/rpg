mod tables;
mod encoder;
mod decoder;

pub use self::encoder::encode;
pub use self::decoder::decode;

type U6 = u8;

const OCTETS_PER_BLOCK: usize = 3;
const SEXTETS_PER_BLOCK: usize = 4;
const PAD_BYTE: u8 = b'=';

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
