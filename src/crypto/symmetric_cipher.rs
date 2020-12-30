use crate::Error;

use super::mode_of_operations::Mode;

pub type PlainTextOut = Vec<u8>;
pub type CipherTextOut = Vec<u8>;

pub trait SymmetricKey {
    type CipherKey;

    fn with_key(self, key: Self::CipherKey) -> Self;
}

pub trait BlockCipher {
    fn using_mode(self, mode: Mode) -> Self;
}

pub trait SymmetricEncryption {
    fn encrypt(self, plaintext: &[u8]) -> Result<CipherTextOut, Error>;
}

pub trait SymmetricDecryption {
    fn decrypt(self, ciphertext: &[u8]) -> Result<PlainTextOut, Error>;
}
