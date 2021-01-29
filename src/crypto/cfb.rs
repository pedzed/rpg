use std::convert::TryInto;

use rand::random;
use aes::aes128 as aes128_backend;

use super::symmetric_cipher::CipherTextOut;
use super::symmetric_cipher::PlainTextOut;

use crate::Error;

type Block = [u8; CfbAes128::BS];

pub struct CfbAes128;

impl CfbAes128 {
    const BS: usize = 16;

    #[allow(non_snake_case)]
    pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<CipherTextOut, Error> {
        if plaintext.len() == 0 {
            return Err("Failed reading plaintext data. No plaintext data provided?".into())
        }

        let IV: [u8; Self::BS] = random();
        let mut C: Vec<u8> = Vec::with_capacity(IV.len() + plaintext.len());

        let mut FR = IV;
        let mut FRE = Self::encrypt_block(&FR, key)?;
        C.append(&mut xor_block(&FRE, &plaintext[0..Self::BS]));

        let mut plaintext_blocks = plaintext.chunks(Self::BS);

        while let Some(plaintext_block) = plaintext_blocks.next() {
            {
                let current = C.len();
                let total = C.capacity();
                let percentage = current as f32 / total as f32 * 100.0;

                println!("{:.1}% encrypted ({}/{} KB).", percentage, current / 1000, total / 1000);
            }

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
        if ciphertext.len() == 0 {
            return Err("Failed reading ciphertext data. No ciphertext data provided?".into())
        }

        let mut ciphertext_blocks = ciphertext.chunks(Self::BS).peekable();

        let IV = ciphertext_blocks.next().unwrap();

        let mut P: Vec<u8> = Vec::with_capacity(ciphertext.len() - IV.len());

        let FR = IV;
        let FRE = Self::encrypt_block(&FR, key)?;

        if let Some(ciphertext_block) = ciphertext_blocks.peek() {
            P.append(&mut xor_block(&FRE, ciphertext_block));
        }

        while let Some(FR) = ciphertext_blocks.next() {
            {
                let current = P.len();
                let total = P.capacity();
                let percentage = current as f32 / total as f32 * 100.0;

                println!("{:.1}% decrypted ({}/{} KB).", percentage, current / 1000, total / 1000);
            }

            if let Some(ciphertext_block) = ciphertext_blocks.peek() {
                let FRE = Self::encrypt_block(&FR, key)?;

                let plaintext_block = &mut xor_block(&FRE, ciphertext_block);
                P.append(plaintext_block);
            }
        }

        Ok(P)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_without_plaintext_data() {
        let key = (0x00112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"";
        assert_eq!(plaintext.len(), 0);

        let ciphertext = CfbAes128::encrypt(plaintext, &key);

        assert_eq!(
            ciphertext.unwrap_err().to_string(),
            "Failed reading plaintext data. No plaintext data provided?".to_string()
        );
    }

    #[test]
    fn encrypt_three_blocks_exact() {
        let key = (0x00112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"This secret message uses exactly three blocks...";
        assert_eq!(plaintext.len(), 3 * 16);

        let ciphertext = CfbAes128::encrypt(plaintext, &key).expect("Failed to encrypt.");
        let decrypted_text = CfbAes128::decrypt(&ciphertext, &key).expect("Failed to decrypt.");

        assert_eq!(decrypted_text.to_vec(), plaintext.to_vec());
    }

    #[test]
    fn encrypt_three_blocks_nonfull() {
        let key = (0x00112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"This secret message uses less than 3 blocks.";
        assert_ne!(plaintext.len(), 3 * 16);

        let ciphertext = CfbAes128::encrypt(plaintext, &key).expect("Failed to encrypt.");
        let decrypted_text = CfbAes128::decrypt(&ciphertext, &key).expect("Failed to decrypt.");

        assert_eq!(decrypted_text.to_vec(), plaintext.to_vec());
    }
}
