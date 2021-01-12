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

            pub type CipherKey = [u8; $key_size];
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
                    for word_index in 0..Self::Nk {
                        for word_element_index in 0..4 {
                            let key_element_index = word_element_index + Self::Nb * word_index;
                            expanded[word_index][word_element_index] = cipher_key[key_element_index];
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
                    let mut state: State = plaintext.into();

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
                    let mut state: State = ciphertext.into();

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
define_aes_cipher!(aes192, Aes192, 24, "AES-192 block cipher");
define_aes_cipher!(aes256, Aes256, 32, "AES-256 block cipher");

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::aes128::Aes128;
    use super::aes192::Aes192;
    use super::aes256::Aes256;

    #[test]
    fn aes_128_encrypt_one_full_block() {
        let plaintext = hex!("00112233 44556677 8899AABB CCDDEEFF");
        let cipher_key: [u8; 16] = hex!("00010203 04050607 08090A0B 0C0D0E0F");

        let actual_ciphertext = Aes128::with_key(cipher_key).encrypt_block(plaintext);
        let expected_ciphertext = hex!("69C4E0D8 6A7B0430 D8CDB780 70B4C55A");

        assert_eq!(actual_ciphertext, expected_ciphertext);
    }

    #[test]
    fn aes_128_encrypt_one_full_block_2() {
        let plaintext = hex!("3243F6A8 885A308D 313198A2 E0370734");
        let cipher_key = hex!("2B7E1516 28AED2A6 ABF71588 09CF4F3C");

        let actual_ciphertext = Aes128::with_key(cipher_key).encrypt_block(plaintext);
        let expected_ciphertext = hex!("3925841D 02DC09FB DC118597 196A0B32");

        assert_eq!(actual_ciphertext, expected_ciphertext);
    }

    #[test]
    fn aes_128_decrypt_one_full_block() {
        let ciphertext = hex!("69C4E0D8 6A7B0430 D8CDB780 70B4C55A");
        let cipher_key = hex!("00010203 04050607 08090A0B 0C0D0E0F");

        let actual_plaintext = Aes128::with_key(cipher_key).decrypt_block(ciphertext);
        let expected_plaintext = hex!("00112233 44556677 8899AABB CCDDEEFF");

        assert_eq!(actual_plaintext, expected_plaintext);
    }

    #[test]
    fn aes_128_expand_keys() {
        assert_eq!(Aes128::EXPANDED_KEY_WORD_COUNT, 44);

        let aes = Aes128::with_key(hex!("2B7E1516 28AED2A6 ABF71588 09CF4F3C"));

        let expected_words = [
            hex!("2B7E1516"), hex!("28AED2A6"), hex!("ABF71588"), hex!("09CF4F3C"), // Round Key 0
            hex!("A0FAFE17"), hex!("88542CB1"), hex!("23A33939"), hex!("2A6C7605"), // Round Key 1
            hex!("F2C295F2"), hex!("7A96B943"), hex!("5935807A"), hex!("7359F67F"), // Round Key 2
            hex!("3D80477D"), hex!("4716FE3E"), hex!("1E237E44"), hex!("6D7A883B"), // Round Key 3
            hex!("EF44A541"), hex!("A8525B7F"), hex!("B671253B"), hex!("DB0BAD00"), // Round Key 4
            hex!("D4D1C6F8"), hex!("7C839D87"), hex!("CAF2B8BC"), hex!("11F915BC"), // Round Key 5
            hex!("6D88A37A"), hex!("110B3EFD"), hex!("DBF98641"), hex!("CA0093FD"), // Round Key 6
            hex!("4E54F70E"), hex!("5F5FC9F3"), hex!("84A64FB2"), hex!("4EA6DC4F"), // Round Key 7
            hex!("EAD27321"), hex!("B58DBAD2"), hex!("312BF560"), hex!("7F8D292F"), // Round Key 8
            hex!("AC7766F3"), hex!("19FADC21"), hex!("28D12941"), hex!("575C006E"), // Round Key 9
            hex!("D014F9A8"), hex!("C9EE2589"), hex!("E13F0CC8"), hex!("B6630CA6"), // Round Key 10
        ];

        for (i, _) in expected_words.iter().enumerate() {
            assert_eq!(
                aes.expanded_key_words[i],
                expected_words[i],
                "Word {}", i
            );
        }
    }

    #[test]
    fn aes_192_expand_keys() {
        assert_eq!(Aes192::EXPANDED_KEY_WORD_COUNT, 52);

        let aes = Aes192::with_key(hex!("8E73B0F7 DA0E6452 C810F32B 809079E5 62F8EAD2 522C6B7B"));

        let expected_words = [
            hex!("8E73B0F7"), hex!("DA0E6452"), hex!("C810F32B"), hex!("809079E5"), // Round Key 0
            hex!("62F8EAD2"), hex!("522C6B7B"), hex!("FE0C91F7"), hex!("2402F5A5"), // Round Key 1
            hex!("EC12068E"), hex!("6C827F6B"), hex!("0E7A95B9"), hex!("5C56FEC2"), // Round Key 2
            hex!("4DB7B4BD"), hex!("69B54118"), hex!("85A74796"), hex!("E92538FD"), // Round Key 3
            hex!("E75FAD44"), hex!("BB095386"), hex!("485AF057"), hex!("21EFB14F"), // Round Key 4
            hex!("A448F6D9"), hex!("4D6DCE24"), hex!("AA326360"), hex!("113B30E6"), // Round Key 5
            hex!("A25E7ED5"), hex!("83B1CF9A"), hex!("27F93943"), hex!("6A94F767"), // Round Key 6
            hex!("C0A69407"), hex!("D19DA4E1"), hex!("EC1786EB"), hex!("6FA64971"), // Round Key 7
            hex!("485F7032"), hex!("22CB8755"), hex!("E26D1352"), hex!("33F0B7B3"), // Round Key 8
            hex!("40BEEB28"), hex!("2F18A259"), hex!("6747D26B"), hex!("458C553E"), // Round Key 9
            hex!("A7E1466C"), hex!("9411F1DF"), hex!("821F750A"), hex!("AD07D753"), // Round Key 10
            hex!("CA400538"), hex!("8FCC5006"), hex!("282D166A"), hex!("BC3CE7B5"), // Round Key 11
            hex!("E98BA06F"), hex!("448C773C"), hex!("8ECC7204"), hex!("01002202"), // Round Key 12
        ];

        for (i, _) in expected_words.iter().enumerate() {
            assert_eq!(
                aes.expanded_key_words[i],
                expected_words[i],
                "Word {}", i
            );
        }
    }

    #[test]
    fn aes_192_encrypt_one_full_block() {
        let plaintext = hex!("00112233 44556677 8899AABB CCDDEEFF");
        let cipher_key = hex!("00010203 04050607 08090A0B 0C0D0E0F 10111213 14151617");

        let actual_ciphertext = Aes192::with_key(cipher_key).encrypt_block(plaintext);
        let expected_ciphertext = hex!("DDA97CA4 864CDFE0 6EAF70A0 EC0D7191");

        assert_eq!(actual_ciphertext, expected_ciphertext);
    }

    #[test]
    fn aes_192_decrypt_one_full_block() {
        let ciphertext = hex!("DDA97CA4 864CDFE0 6EAF70A0 EC0D7191");
        let cipher_key = hex!("00010203 04050607 08090A0B 0C0D0E0F 10111213 14151617");

        let actual_plaintext = Aes192::with_key(cipher_key).decrypt_block(ciphertext);
        let expected_plaintext = hex!("00112233 44556677 8899AABB CCDDEEFF");

        assert_eq!(actual_plaintext, expected_plaintext);
    }

    #[test]
    fn aes_256_expand_keys() {
        assert_eq!(Aes256::EXPANDED_KEY_WORD_COUNT, 60);

        let aes = Aes256::with_key(hex!("603DEB10 15CA71BE 2B73AEF0 857D7781 1F352C07 3B6108D7 2D9810A3 0914DFF4"));

        let expected_words = [
            hex!("603DEB10"), hex!("15CA71BE"), hex!("2B73AEF0"), hex!("857D7781"), // Round Key 0
            hex!("1F352C07"), hex!("3B6108D7"), hex!("2D9810A3"), hex!("0914DFF4"), // Round Key 1
            hex!("9BA35411"), hex!("8E6925AF"), hex!("A51A8B5F"), hex!("2067FCDE"), // Round Key 2
            hex!("A8B09C1A"), hex!("93D194CD"), hex!("BE49846E"), hex!("B75D5B9A"), // Round Key 3
            hex!("D59AECB8"), hex!("5BF3C917"), hex!("FEE94248"), hex!("DE8EBE96"), // Round Key 4
            hex!("B5A9328A"), hex!("2678A647"), hex!("98312229"), hex!("2F6C79B3"), // Round Key 5
            hex!("812C81AD"), hex!("DADF48BA"), hex!("24360AF2"), hex!("FAB8B464"), // Round Key 6
            hex!("98C5BFC9"), hex!("BEBD198E"), hex!("268C3BA7"), hex!("09E04214"), // Round Key 7
            hex!("68007BAC"), hex!("B2DF3316"), hex!("96E939E4"), hex!("6C518D80"), // Round Key 8
            hex!("C814E204"), hex!("76A9FB8A"), hex!("5025C02D"), hex!("59C58239"), // Round Key 9
            hex!("DE136967"), hex!("6CCC5A71"), hex!("FA256395"), hex!("9674EE15"), // Round Key 10
            hex!("5886CA5D"), hex!("2E2F31D7"), hex!("7E0AF1FA"), hex!("27CF73C3"), // Round Key 11
            hex!("749C47AB"), hex!("18501DDA"), hex!("E2757E4F"), hex!("7401905A"), // Round Key 12
            hex!("CAFAAAE3"), hex!("E4D59B34"), hex!("9ADF6ACE"), hex!("BD10190D"), // Round Key 13
            hex!("FE4890D1"), hex!("E6188D0B"), hex!("046DF344"), hex!("706C631E"), // Round Key 14
        ];

        for (i, _) in expected_words.iter().enumerate() {
            assert_eq!(
                aes.expanded_key_words[i],
                expected_words[i],
                "Word {}", i
            );
        }
    }

    #[test]
    fn aes_256_encrypt_one_full_block() {
        let plaintext = hex!("00112233 44556677 8899AABB CCDDEEFF");
        let cipher_key = hex!("00010203 04050607 08090A0B 0C0D0E0F 10111213 14151617 18191A1B 1C1D1E1F");

        let actual_ciphertext = Aes256::with_key(cipher_key).encrypt_block(plaintext);
        let expected_ciphertext = hex!("8EA2B7CA 516745BF EAFC4990 4B496089");

        assert_eq!(actual_ciphertext, expected_ciphertext);
    }

    #[test]
    fn aes_256_decrypt_one_full_block() {
        let ciphertext = hex!("8EA2B7CA 516745BF EAFC4990 4B496089");
        let cipher_key = hex!("00010203 04050607 08090A0B 0C0D0E0F 10111213 14151617 18191A1B 1C1D1E1F");

        let actual_plaintext = Aes256::with_key(cipher_key).decrypt_block(ciphertext);
        let expected_plaintext = hex!("00112233 44556677 8899AABB CCDDEEFF");

        assert_eq!(actual_plaintext, expected_plaintext);
    }
}
