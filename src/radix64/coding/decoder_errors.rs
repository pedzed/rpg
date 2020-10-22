use std::error;
use std::fmt;

#[derive(Debug)]
pub enum DecoderError {
    UnexpectedChar(u8),
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecoderError::UnexpectedChar(ref x) => write!(f, "Unexpected character `{}` found.", x),
        }
    }
}

impl error::Error for DecoderError {
    // TODO?
}
