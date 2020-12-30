use std::convert::TryInto;

use aes::aes128 as aes128_backend;

use super::symmetric_cipher::CipherTextOut;
use super::symmetric_cipher::PlainTextOut;

use crate::Error;

type Block = [u8; OpenPgpCfbAes128::BS];

pub struct OpenPgpCfbAes128;

impl OpenPgpCfbAes128 {
    const BS: usize = 16;
    const PREFIX_LENGTH: usize = Self::BS + 2;

    #[allow(non_snake_case)]
    pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<CipherTextOut, Error> {
        let mut C: Vec<u8> = Vec::with_capacity(Self::PREFIX_LENGTH + plaintext.len());

        let prefix = generate_random_prefix(Self::PREFIX_LENGTH);

        let IV: [u8; Self::BS] = [0; Self::BS];
        let mut FR = IV;
        let mut FRE = Self::encrypt_block(&FR, key)?;
        C.append(&mut xor_block(&FRE, &prefix[0..Self::BS]));

        FR = C[0..Self::BS].try_into()?;
        FRE = Self::encrypt_block(&FR, key)?;
        C.push(FRE[0] ^ prefix[Self::BS + 0]);
        C.push(FRE[1] ^ prefix[Self::BS + 1]);

        if plaintext.len() == 0 {
            // NOTE: Could also return error. In that case, return early at fn start
            // return Err("Failed reading plaintext data. No plaintext data provided?".into())
            return Ok(C)
        }

        let mut plaintext_blocks = plaintext.chunks(Self::BS);

        // The resync step
        FR = C[2..prefix.len()].try_into()?;
        FRE = Self::encrypt_block(&FR, key)?;
        C.append(&mut xor_block(&FRE, &plaintext_blocks.next().unwrap()));

        while let Some(plaintext_block) = plaintext_blocks.next() {
            let range = (C.len() - Self::BS)..C.len();
            FR = C[range].try_into()?;
            FRE = Self::encrypt_block(&FR, key)?;
            C.append(&mut xor_block(&FRE, &plaintext_block));
        }

        Ok(C)
    }

    fn encrypt_block(plaintext_block: &[u8], key: &[u8]) -> Result<Block, Error> {
        let key = key.try_into()?;
        let plaintext = plaintext_block.try_into()?;

        Ok(aes128_backend::Aes128::with_key(key).encrypt_block(plaintext))
    }

    #[allow(non_snake_case)]
    pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<PlainTextOut, Error> {
        let offset = 2;

        let mut ciphertext_blocks = ciphertext[offset..]
            .chunks(Self::BS)
            .peekable()
        ;

        let mut decrypted: Vec<u8> = Vec::with_capacity(ciphertext.len());

        let IV: [u8; Self::BS] = [0; Self::BS];
        let FRE = Self::encrypt_block(&IV, key)?;

        if let Some(ciphertext_block) = ciphertext_blocks.peek() {
            decrypted.append(&mut xor_block(&FRE, ciphertext_block));
        }

        while let Some(FR) = ciphertext_blocks.next() {
            if let Some(ciphertext_block) = ciphertext_blocks.peek() {
                let FRE = Self::encrypt_block(&FR, key)?;

                let plaintext_block = &mut xor_block(&FRE, ciphertext_block);
                decrypted.append(plaintext_block);
            }
        }

        let plaintext = decrypted[Self::BS..].to_vec();

        Ok(plaintext)
    }
}

fn xor_block(input1: &[u8], input2: &[u8]) -> Vec<u8> {
    input1
        .iter()
        .zip(input2)
        .map(|(i, j)| i ^ j)
        .collect()
}

// More efficient in-place XOR
// fn xor_block(input1: &[u8], input2: &mut [u8]) {
//     input1
//         .iter()
//         .zip(input2)
//         .for_each(|(left, right)| *right ^= left);
// }

fn generate_random_prefix(length: usize) -> Vec<u8> {
    let prefix = random_data(length - 2);
    [prefix.as_ref(), &prefix[prefix.len()-2..]].concat()
}

// WARN: Not random at the moment
fn random_data(length: usize) -> Vec<u8> {
    let data = b"\x00\x11\x22\x33\x44\x55\x66\x77\x88\x99\xAA\xBB\xCC\xDD\xEE\xFF"; // TODO: Randomize

    data.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_without_plaintext_data() {
        let key = (0x112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"";
        assert_eq!(plaintext.len(), 0);

        let ciphertext = OpenPgpCfbAes128::encrypt(plaintext, &key).expect("Failed to encrypt.");
        let decrypted_text = OpenPgpCfbAes128::decrypt(&ciphertext, &key).expect("Failed to decrypt.");

        assert_eq!(decrypted_text.to_vec(), plaintext.to_vec());
    }

    #[test]
    fn encrypt_three_blocks_exact() {
        let key = (0x112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"This secret message uses exactly three blocks...";
        assert_eq!(plaintext.len(), 3 * 16);

        let ciphertext = OpenPgpCfbAes128::encrypt(plaintext, &key).expect("Failed to encrypt.");

        assert_eq!(ciphertext, vec![
            // Prefix
            0xFD, 0xF5, 0xD9, 0x9D, 0x0E, 0x5C, 0x86, 0x57,
            0x67, 0x6E, 0x88, 0x2D, 0x53, 0x5E, 0x6D, 0xD4,
            0xFE, 0xE4,

            // Encrypted plaintext
            0xA9, 0x4E, 0x69, 0x05, 0x69, 0x68, 0x17, 0x06,
            0xCD, 0xFE, 0x30, 0x0B, 0xAC, 0xDF, 0xC2, 0x99,

            0xC5, 0x98, 0xDC, 0x37, 0x6F, 0x88, 0x94, 0x2A,
            0xA4, 0xFC, 0xC5, 0xC5, 0xA8, 0x15, 0x6E, 0x3F,

            0xA1, 0xA6, 0x30, 0x51, 0x8B, 0x4B, 0xE8, 0x4D,
            0xA7, 0x2F, 0xF0, 0x79, 0x9B, 0xDE, 0x17, 0x14,
        ]);

        let decrypted_text = OpenPgpCfbAes128::decrypt(&ciphertext, &key).expect("Failed to decrypt.");

        assert_eq!(decrypted_text.to_vec(), plaintext.to_vec());
    }

    #[test]
    fn encrypt_three_blocks_nonfull() {
        let key = (0x112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"This secret message uses less than 3 blocks.";
        assert_ne!(plaintext.len(), 3 * 16);

        let ciphertext = OpenPgpCfbAes128::encrypt(plaintext, &key).expect("Failed to encrypt.");

        assert_eq!(ciphertext, vec![
            // Prefix
            0xFD, 0xF5, 0xD9, 0x9D, 0x0E, 0x5C, 0x86, 0x57,
            0x67, 0x6E, 0x88, 0x2D, 0x53, 0x5E, 0x6D, 0xD4,
            0xFE, 0xE4,

            // Encrypted plaintext
            0xA9, 0x4E, 0x69, 0x05, 0x69, 0x68, 0x17, 0x06,
            0xCD, 0xFE, 0x30, 0x0B, 0xAC, 0xDF, 0xC2, 0x99,

            0xC5, 0x98, 0xDC, 0x37, 0x6F, 0x88, 0x94, 0x2A,
            0xA4, 0xF5, 0xD8, 0xD7, 0xB8, 0x41, 0x76, 0x2E,

            0x1A, 0x96, 0x3F, 0xA9, 0xB1, 0x80, 0xEE, 0x73,
            0xD7, 0x82, 0x58, 0xAB,
        ]);

        let decrypted_text = OpenPgpCfbAes128::decrypt(&ciphertext, &key).expect("Failed to decrypt.");

        assert_eq!(decrypted_text.to_vec(), plaintext.to_vec());
    }
}
