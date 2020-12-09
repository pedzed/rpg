extern crate aes;

use aes::aes128 as aes128_backend;

use super::openpgp_cfb::OpenPgpCfbAes128;
use super::symmetric_cipher::{SymmetricEncryption, SymmetricDecryption};
use super::symmetric_cipher::SymmetricKey;
use super::symmetric_cipher::BlockCipher;
use super::symmetric_cipher::{PlainTextOut, CipherTextOut};
use super::symmetric_cipher::Error;
use super::mode_of_operations::Mode;

pub struct Aes128 {
    mode: Option<Mode>,
    key: Option<aes128_backend::CipherKey>,
}

impl Aes128 {
    pub fn new() -> Self {
        Self {
            mode: None,
            key: None,
        }
    }
}

impl SymmetricKey for Aes128 {
    type CipherKey = aes128_backend::CipherKey;

    fn with_key(mut self, key: Self::CipherKey) -> Self {
        self.key = Some(key);
        self
    }
}

impl BlockCipher for Aes128 {
    fn using_mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }
}

impl SymmetricEncryption for Aes128 {
    fn encrypt(self, plaintext: &[u8]) -> Result<CipherTextOut, Error> {
        if self.key.is_none() {
            return Err("Cipher key is not set.".into())
        }

        if self.mode.is_none() {
            return Err("Block mode of operations is not set.".into())
        }

        match self.mode.unwrap() {
            Mode::OpenPgpCfb => OpenPgpCfbAes128::encrypt(plaintext, &self.key.unwrap()),
        }
    }
}

impl SymmetricDecryption for Aes128 {
    fn decrypt(self, ciphertext: &[u8]) -> Result<PlainTextOut, Error> {
        if self.key.is_none() {
            return Err("Cipher key is not set.".into())
        }

        if self.mode.is_none() {
            return Err("Block mode of operations is not set.".into())
        }

        match self.mode.unwrap() {
            Mode::OpenPgpCfb => OpenPgpCfbAes128::decrypt(ciphertext, &self.key.unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes128_openpgp_cfb() {
        let key = (0x112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"Hello world!";

        let ciphertext = Aes128::new()
            .with_key(key)
            .using_mode(Mode::OpenPgpCfb)
            .encrypt(plaintext)
            .unwrap()
        ;

        let decrypted = Aes128::new()
            .with_key(key)
            .using_mode(Mode::OpenPgpCfb)
            .decrypt(&ciphertext)
            .unwrap()
        ;

        assert_eq!(decrypted, plaintext);
    }
}
