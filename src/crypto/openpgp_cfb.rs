use std::convert::TryInto;

use aes::aes128 as aes128_backend;

use super::symmetric_cipher::CipherTextOut;
use super::symmetric_cipher::PlainTextOut;
use super::symmetric_cipher::Error;

type Block = [u8; OpenPgpCfbAes128::BS];

pub struct OpenPgpCfbAes128;

impl OpenPgpCfbAes128 {
    const BS: usize = 16;

    #[allow(non_snake_case)]
    pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<CipherTextOut, Error> {
        // NOTE: The following steps are taken from RFC 4880, where start index
        //       is assumed to be 1 instead of 0.

        // 1.  The feedback register (FR) is set to the IV, which is all zeros.
        let IV: [u8; Self::BS] = [0; Self::BS];
        let mut FR = IV;

        // 2.  FR is encrypted to produce FRE (FR Encrypted).  This is the
        //     encryption of an all-zero value.
        let mut FRE = Self::encrypt_block(&FR, key)?;

        // 3.  FRE is xored with the first BS octets of random data prefixed to
        //     the plaintext to produce C[1] through C[BS], the first BS octets
        //     of ciphertext.
        let prefix = b"\x00\x11\x22\x33\x44\x55\x66\x77\x88\x99\xAA\xBB\xCC\xDD\xEE\xFF\xEE\xFF"; // TODO: Randomize
        // let prefix = b"\x00\x11\x22\x33\x44\x55\x66\x77\x88\x99\xAA\xBB\xCC\xDD\xEE\xFF"; // TODO: Randomize
        let unencrypted: &[u8] = &[prefix, plaintext].concat();

        let mut C: Vec<u8> = Vec::with_capacity(unencrypted.len());

        let range = 0..Self::BS;

        C.append(&mut xor_block(&FRE, &unencrypted[range.clone()]));

        // 4.  FR is loaded with C[1] through C[BS].
        FR = C[range].try_into()?;

        // 5.  FR is encrypted to produce FRE, the encryption of the first BS
        //     octets of ciphertext.
        FRE = Self::encrypt_block(&FR, key)?;

        // 6.  The left two octets of FRE get xored with the next two octets of
        //     data that were prefixed to the plaintext.  This produces C[BS+1]
        //     and C[BS+2], the next two octets of ciphertext.
        C.push(FRE[0] ^ unencrypted[Self::BS + 0]);
        C.push(FRE[1] ^ unencrypted[Self::BS + 1]);

        // 7.  (The resync step) FR is loaded with C[3] through C[BS+2].
        let range = 2..C.len();
        FR = C[range].try_into()?;

        // 8.  FR is encrypted to produce FRE.
        FRE = Self::encrypt_block(&FR, key)?;

        // 9.  FRE is xored with the first BS octets of the given plaintext, now
        //     that we have finished encrypting the BS+2 octets of prefixed
        //     data.  This produces C[BS+3] through C[BS+(BS+2)], the next BS
        //     octets of ciphertext.
        let range = C.len()..(C.len() + Self::BS);
        C.append(&mut xor_block(&FRE, &unencrypted[range]));

        for _ in (C.len()..C.capacity()).step_by(Self::BS) {
            // 10. FR is loaded with C[BS+3] to C[BS + (BS+2)]
            let range = (C.len() - Self::BS)..C.len();
            FR = C[range].try_into()?;

            // 11. FR is encrypted to produce FRE.
            FRE = Self::encrypt_block(&FR, key)?;

            // 12. FRE is xored with the next BS octets of plaintext, to produce
            //     the next BS octets of ciphertext.  These are loaded into FR, and
            //     the process is repeated until the plaintext is used up.
            let mut range = C.len()..(C.len() + Self::BS);

            if range.end > unencrypted.len() {
                range.end = unencrypted.len();
            }

            C.append(&mut xor_block(&FRE, &unencrypted[range]));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_three_blocks_exact() {
        let key = (0x112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"This secret message uses exactly three blocks...";
        assert_eq!(plaintext.len(), 3 * 16);

        let ciphertext = OpenPgpCfbAes128::encrypt(plaintext, &key)
            .expect("Failed to encrypt.")
        ;

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

        let decrypted_text = OpenPgpCfbAes128::decrypt(&ciphertext, &key)
            .expect("Failed to decrypt.")
        ;

        assert_eq!(decrypted_text.to_vec(), plaintext.to_vec());
    }

    #[test]
    fn encrypt_three_blocks_nonfull() {
        let key = (0x112233445566778899AABBCCDDEEFF as u128).to_be_bytes();
        let plaintext = b"This secret message uses less than 3 blocks.";
        assert_ne!(plaintext.len(), 3 * 16);

        let ciphertext = OpenPgpCfbAes128::encrypt(plaintext, &key)
            .expect("Failed to encrypt.")
        ;

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

        let decrypted_text = OpenPgpCfbAes128::decrypt(&ciphertext, &key)
            .expect("Failed to decrypt.")
        ;

        assert_eq!(decrypted_text.to_vec(), plaintext.to_vec());
    }
}
