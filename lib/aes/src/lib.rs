pub mod round_key;
pub mod sbox_tables;
pub mod block;
pub mod state;
pub mod word;
pub mod rcon;

use round_key::RoundKey;
use state::State;
use word::Word;

use state::AddRoundKey;
use state::SubBytes;
use state::ShiftRows;
use state::MixColumns;

use word::RotWord;
use word::SubWord;

use rcon::rcon;

macro_rules! define_aes_cipher {
    (
        $mod:ident,
        $cipher:ident,
        $key_size:expr,
        $doc:expr
    ) => {
        pub mod $mod {
            use super::*;

            type CipherKey = [u8; $key_size];
            type ExpandedKeyWords = [[u8; 4]; $cipher::EXPANDED_KEY_WORD_COUNT];

            #[doc=$doc]
            pub struct $cipher {
                pub expanded_key_words: ExpandedKeyWords,
            }

            #[allow(non_upper_case_globals)]
            impl $cipher {
                /// The amount of words in the block
                ///
                /// Called "Nb" in FIPS 197.
                const Nb: usize = 4;

                /// The amount of words in the cipher key
                ///
                /// Called "Nk" in FIPS 197.
                const Nk: usize = $key_size / 4;

                /// The amount of rounds (excl. initialization step)
                ///
                /// Called "Nr" in FIPS 197.
                const Nr: usize = Self::Nk + 6;

                /// The block size in bytes
                const BS: usize = 16;

                pub const EXPANDED_KEY_WORD_COUNT: usize = Self::Nb * (Self::Nr + 1);

                pub fn with_key(cipher_key: CipherKey) -> Self {
                    Self {
                        expanded_key_words: Self::expand_key(cipher_key),
                    }
                }

                fn expand_key(cipher_key: CipherKey) -> ExpandedKeyWords {
                    let mut expanded: ExpandedKeyWords = [[0; 4]; Self::EXPANDED_KEY_WORD_COUNT];

                    // Populating the cipher key
                    for c in 0..Self::Nk {
                        for r in 0..Self::Nb {
                            let key_element_index = r + Self::Nk * c;
                            expanded[c][r] = cipher_key[key_element_index];
                        }
                    }

                    // Populating the rest of the expanded key
                    for c in Self::Nk..Self::EXPANDED_KEY_WORD_COUNT {
                        let mut temp_word: Word = expanded[c-1];

                        if c % Self::Nk == 0 {
                            temp_word.rot_word();
                            temp_word.sub_word();

                            let round_number = c / Self::Nk;
                            temp_word[0] ^= rcon(round_number);
                        } else if Self::Nk > 6 && c % Self::Nk == 4 {
                            temp_word.sub_word();
                        }

                        for (r, _) in temp_word.iter().enumerate() {
                            expanded[c][r] = expanded[c - Self::Nk][r] ^ temp_word[r];
                        }
                    }

                    expanded
                }

                /// Encrypt a single block
                pub fn encrypt_block(self, plaintext: [u8; Self::BS]) -> [u8; Self::BS] {
                    let mut state = State::new([
                        [plaintext[0], plaintext[1], plaintext[2], plaintext[3]],
                        [plaintext[4], plaintext[5], plaintext[6], plaintext[7]],
                        [plaintext[8], plaintext[9], plaintext[10], plaintext[11]],
                        [plaintext[12], plaintext[13], plaintext[14], plaintext[15]],
                    ]);

                    state.add_round_key(self.round_key(0));

                    for round_number in 1..Self::Nr {
                        state.sub_bytes();
                        state.shift_rows();
                        state.mix_columns();
                        state.add_round_key(self.round_key(round_number));
                    }

                    state.sub_bytes();
                    state.shift_rows();
                    state.add_round_key(self.round_key(Self::Nr));

                    state.into()
                }

                fn round_key(&self, round_index: usize) -> RoundKey {
                    let mut output: RoundKey = RoundKey::default();

                    for c in 0..block::COLUMN_COUNT {
                        for r in 0..block::ROW_COUNT {
                            output[c][r] = self.expanded_key_words[round_index * 4 + c][r];
                        }
                    }

                    output
                }

                /// Decrypt a single block
                pub fn decrypt_block(self, ciphertext: [u8; Self::BS]) -> [u8; Self::BS] {
                    let mut state = State::new([
                        [ciphertext[0], ciphertext[1], ciphertext[2], ciphertext[3]],
                        [ciphertext[4], ciphertext[5], ciphertext[6], ciphertext[7]],
                        [ciphertext[8], ciphertext[9], ciphertext[10], ciphertext[11]],
                        [ciphertext[12], ciphertext[13], ciphertext[14], ciphertext[15]],
                    ]);

                    state.add_round_key(self.round_key(Self::Nr));

                    for round_number in (1..=Self::Nr-1).rev() {
                        state.inv_shift_rows();
                        state.inv_sub_bytes();
                        state.add_round_key(self.round_key(round_number));
                        state.inv_mix_columns();
                    }

                    state.inv_shift_rows();
                    state.inv_sub_bytes();
                    state.add_round_key(self.round_key(0));

                    state.into()
                }
            }
        }
    }
}

define_aes_cipher!(aes128, Aes128, 16, "AES-128 block cipher");
// define_aes_cipher!(aes192, Aes192, 24, "AES-192 block cipher");
// define_aes_cipher!(aes256, Aes256, 32, "AES-256 block cipher");

#[cfg(test)]
mod tests {
    use super::aes128::Aes128;
    // use super::aes192::Aes192;
    // use super::aes256::Aes256;

    #[test]
    fn aes_128_encrypt_one_full_block() {
        let plaintext = (0x00112233_44556677_8899AABB_CCDDEEFF as u128).to_be_bytes();
        let cipher_key = (0x00010203_04050607_08090A0B_0C0D0E0F as u128).to_be_bytes();

        let actual_ciphertext = Aes128::with_key(cipher_key).encrypt_block(plaintext);
        let expected_ciphertext = (0x69C4E0D8_6A7B0430_D8CDB780_70B4C55A as u128).to_be_bytes();

        assert_eq!(actual_ciphertext, expected_ciphertext);
    }

    #[test]
    fn aes_128_encrypt_one_full_block_2() {
        let plaintext = (0x3243F6A8_885A308D_313198A2_E0370734 as u128).to_be_bytes();
        let cipher_key = (0x2B7E1516_28AED2A6_ABF71588_09CF4F3C as u128).to_be_bytes();

        let actual_ciphertext = Aes128::with_key(cipher_key).encrypt_block(plaintext);
        let expected_ciphertext = (0x3925841D_02DC09FB_DC118597_196A0B32 as u128).to_be_bytes();

        assert_eq!(actual_ciphertext, expected_ciphertext);
    }

    #[test]
    fn aes_128_decrypt_one_full_block() {
        let ciphertext = (0x69C4E0D8_6A7B0430_D8CDB780_70B4C55A as u128).to_be_bytes();
        let cipher_key = (0x00010203_04050607_08090A0B_0C0D0E0F as u128).to_be_bytes();

        let actual_plaintext = Aes128::with_key(cipher_key).decrypt_block(ciphertext);
        let expected_plaintext = (0x00112233_44556677_8899AABB_CCDDEEFF as u128).to_be_bytes();

        assert_eq!(actual_plaintext, expected_plaintext);
    }

    #[test]
    fn aes_128_expand_keys() {
        assert_eq!(Aes128::EXPANDED_KEY_WORD_COUNT, 44);

        let aes = Aes128::with_key([
            0x2B, 0x7E, 0x15, 0x16,
            0x28, 0xAE, 0xD2, 0xA6,
            0xAB, 0xF7, 0x15, 0x88,
            0x09, 0xCF, 0x4F, 0x3C,
        ]);

        let expected_words = [
            0x2B7E1516, 0x28AED2A6, 0xABF71588, 0x09CF4F3C, // Round Key 0
            0xA0FAFE17, 0x88542CB1, 0x23A33939, 0x2A6C7605, // Round Key 1
            0xF2C295F2, 0x7A96B943, 0x5935807A, 0x7359F67F, // Round Key 2
            0x3D80477D, 0x4716FE3E, 0x1E237E44, 0x6D7A883B, // Round Key 3
            0xEF44A541, 0xA8525B7F, 0xB671253B, 0xDB0BAD00, // Round Key 4
            0xD4D1C6F8, 0x7C839D87, 0xCAF2B8BC, 0x11F915BC, // Round Key 5
            0x6D88A37A, 0x110B3EFD, 0xDBF98641, 0xCA0093FD, // Round Key 6
            0x4E54F70E, 0x5F5FC9F3, 0x84A64FB2, 0x4EA6DC4F, // Round Key 7
            0xEAD27321, 0xB58DBAD2, 0x312BF560, 0x7F8D292F, // Round Key 8
            0xAC7766F3, 0x19FADC21, 0x28D12941, 0x575C006E, // Round Key 9
            0xD014F9A8, 0xC9EE2589, 0xE13F0CC8, 0xB6630CA6, // Round Key 10
        ];

        for (i, _) in expected_words.iter().enumerate() {
            assert_eq!(
                u32::from_be_bytes(aes.expanded_key_words[i]),
                expected_words[i],
                "Word {}", i
            );
        }
    }

    // #[test]
    // fn aes_192_expand_keys() {
    //     assert_eq!(Aes192::EXPANDED_KEY_WORD_COUNT, 52);

    //     let aes = Aes192::with_key([
    //         0x8E, 0x73, 0xB0, 0xF7,
    //         0xDA, 0x0E, 0x64, 0x52,
    //         0xC8, 0x10, 0xF3, 0x2B,
    //         0x80, 0x90, 0x79, 0xE5,
    //         0x62, 0xF8, 0xEA, 0xD2,
    //         0x52, 0x2C, 0x6B, 0x7B,
    //     ]);

    //     let expected_words = [
    //         0x8E73B0F7, 0xDA0E6452, 0xC810F32B, 0x809079E5, // Round Key 0
    //         0x62F8EAD2, 0x522C6B7B, 0xFE0C91F7, 0x2402F5A5, // Round Key 1
    //         0xEC12068E, 0x6C827F6B, 0x0E7A95B9, 0x5C56FEC2, // Round Key 2
    //         0x4DB7B4BD, 0x69B54118, 0x85A74796, 0xE92538FD, // Round Key 3
    //         0xE75FAD44, 0xBB095386, 0x485AF057, 0x21EFB14F, // Round Key 4
    //         0xA448F6D9, 0x4D6DCE24, 0xAA326360, 0x113B30E6, // Round Key 5
    //         0xA25E7ED5, 0x83B1CF9A, 0x27F93943, 0x6A94F767, // Round Key 6
    //         0xC0A69407, 0xD19DA4E1, 0xEC1786EB, 0x6FA64971, // Round Key 7
    //         0x485F7032, 0x22CB8755, 0xE26D1352, 0x33F0B7B3, // Round Key 8
    //         0x40BEEB28, 0x2F18A259, 0x6747D26B, 0x458C553E, // Round Key 9
    //         0xA7E1466C, 0x9411F1DF, 0x821F750A, 0xAD07D753, // Round Key 10
    //         0xCA400538, 0x8FCC5006, 0x282D166A, 0xBC3CE7B5, // Round Key 11
    //         0xE98BA06F, 0x448C773C, 0x8ECC7204, 0x01002202, // Round Key 12
    //     ];

    //     for (i, _) in expected_words.iter().enumerate() {
    //         assert_eq!(
    //             u32::from_be_bytes(aes.expanded_key_words[i]),
    //             expected_words[i],
    //             "Word {}", i
    //         );
    //     }
    // }

    // #[test]
    // fn aes_256_expand_keys() {
    //     assert_eq!(Aes256::EXPANDED_KEY_WORD_COUNT, 60);

    //     let aes = Aes256::with_key([
    //         0x60, 0x3D, 0xEB, 0x10,
    //         0x15, 0xCA, 0x71, 0xBE,
    //         0x2B, 0x73, 0xAE, 0xF0,
    //         0x85, 0x7D, 0x77, 0x81,
    //         0x1F, 0x35, 0x2C, 0x07,
    //         0x3B, 0x61, 0x08, 0xD7,
    //         0x2D, 0x98, 0x10, 0xA3,
    //         0x09, 0x14, 0xDF, 0xF4,
    //     ]);

    //     let expected_words = [
    //         0x603DEB10, 0x15CA71BE, 0x2B73AEF0, 0x857D7781, // Round Key 0
    //         0x1F352C07, 0x3B6108D7, 0x2D9810A3, 0x0914DFF4, // Round Key 1
    //         0x9BA35411, 0x8E6925AF, 0xA51A8B5F, 0x2067FCDE, // Round Key 2
    //         0xA8B09C1A, 0x93D194CD, 0xBE49846E, 0xB75D5B9A, // Round Key 3
    //         0xD59AECB8, 0x5BF3C917, 0xFEE94248, 0xDE8EBE96, // Round Key 4
    //         0xB5A9328A, 0x2678A647, 0x98312229, 0x2F6C79B3, // Round Key 5
    //         0x812C81AD, 0xDADF48BA, 0x24360AF2, 0xFAB8B464, // Round Key 6
    //         0x98C5BFC9, 0xBEBD198E, 0x268C3BA7, 0x09E04214, // Round Key 7
    //         0x68007BAC, 0xB2DF3316, 0x96E939E4, 0x6C518D80, // Round Key 8
    //         0xC814E204, 0x76A9FB8A, 0x5025C02D, 0x59C58239, // Round Key 9
    //         0xDE136967, 0x6CCC5A71, 0xFA256395, 0x9674EE15, // Round Key 10
    //         0x5886CA5D, 0x2E2F31D7, 0x7E0AF1FA, 0x27CF73C3, // Round Key 11
    //         0x749C47AB, 0x18501DDA, 0xE2757E4F, 0x7401905A, // Round Key 12
    //         0xCAFAAAE3, 0xE4D59B34, 0x9ADF6ACE, 0xBD10190D, // Round Key 13
    //         0xFE4890D1, 0xE6188D0B, 0x046DF344, 0x706C631E, // Round Key 14
    //     ];

    //     for (i, _) in expected_words.iter().enumerate() {
    //         assert_eq!(
    //             u32::from_be_bytes(aes.expanded_key_words[i]),
    //             expected_words[i],
    //             "Word {}", i
    //         );
    //     }
    // }
}
