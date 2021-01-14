pub mod armor;
pub mod crc24;
pub mod crypto;

use std::fs;

use crypto::openpgp_cfb::OpenPgpCfbAes128;
use armor::{armor_reader::ArmorReader, armor_writer::ArmorWriter};
use armor::armor_data_types::ArmorDataType;
use armor::armor_data_headers::ArmorDataHeader;

pub type Error = Box<dyn std::error::Error>;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum SymmetricCipher {
    Aes128,
    Aes192,
    Aes256,
}

impl SymmetricCipher {
    pub fn from_str(input: &str) -> Result<Self, Error> {
        let input = &input.to_uppercase()[..];

        match input {
            "AES128" => Ok(Self::Aes128),
            "AES192" => Ok(Self::Aes192),
            "AES256" => Ok(Self::Aes256),
            x => Err(format!("Unknown cipher `{}`.", x).into()),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Aes128 => "AES-128",
            Self::Aes192 => "AES-192",
            Self::Aes256 => "AES-256",
        }
    }
}

pub struct EncryptionCommand {
    pub algo: SymmetricCipher,
    pub input_file: String,
    pub output_file: String,
    pub cipher_key: Vec<u8>,
    pub with_armor: bool,
}

impl EncryptionCommand {
    pub fn run(self) {
        match &self.algo {
            SymmetricCipher::Aes128 => {},
            SymmetricCipher::Aes192 => unimplemented!(),
            SymmetricCipher::Aes256 => unimplemented!(),
        }

        println!("Encrypting `{}` using {}...", self.input_file, self.algo.to_str());

        let plaintext = fs::read(&self.input_file)
            .expect(&format!("Could not read `{}`.", self.input_file))
        ;

        let ciphertext = OpenPgpCfbAes128::encrypt(&plaintext, &self.cipher_key)
            .expect("Failed to encrypt.")
        ;

        let output = match self.with_armor {
            true => {
                let mut armor = ArmorWriter::new();
                armor.data_type = Some(ArmorDataType::PgpMessage);
                armor.add_data_header(
                    ArmorDataHeader::Version,
                    &format!("{} v{}", APP_NAME, APP_VERSION)
                );

                armor.set_data(&ciphertext);
                armor.write_unsafe().as_bytes().to_vec()
            },
            false => ciphertext,
        };

        fs::write(&self.output_file, &output)
            .expect(&format!("Could not write to `{}`.", &self.output_file))
        ;

        println!(
            "Successfully encrypted `{}`. {} bytes of ciphertext written to {}.",
            &self.input_file,
            output.len(),
            &self.output_file,
        );
    }
}

pub struct DecryptionCommand {
    pub algo: SymmetricCipher,
    pub input_file: String,
    pub output_file: String,
    pub cipher_key: Vec<u8>,
    pub ignore_crc_error: bool,
}

impl DecryptionCommand {
    pub fn run(self) {
        match &self.algo {
            SymmetricCipher::Aes128 => {},
            SymmetricCipher::Aes192 => unimplemented!(),
            SymmetricCipher::Aes256 => unimplemented!(),
        }

        println!("Decrypting `{}` using {}...", self.input_file, self.algo.to_str());

        let input = fs::read(&self.input_file)
            .expect(&format!("Could not read `{}`.", &self.input_file))
        ;

        let armor_reader = ArmorReader::read_file(&self.input_file);

        let ciphertext = match armor_reader {
            Ok(ref armor) => {
                match &armor.decoded_data {
                    Ok(decoded_data) => decoded_data,
                    Err(_) => &input,
                }
            },
            Err(_) => &input,
        };

        let plaintext = OpenPgpCfbAes128::decrypt(&ciphertext, &self.cipher_key)
            .expect("Failed to encrypt.")
        ;

        match armor_reader {
            Ok(ref armor) => {
                let checksum = armor.checksum.as_ref().expect("Could not read checksum.");

                match checksum.verify(&ciphertext) {
                    true => {
                        println!("✓ Checksum verification passed.");
                    },
                    false => {
                        match self.ignore_crc_error {
                            true => {
                                println!("✗ Checksum verification of `{}` failed. Ignoring...", checksum.get());
                            },
                            false => {
                                panic!("✗ Checksum verification of `{}` failed. Aborting.", checksum.get());
                            },
                        }
                    },
                }
            },
            Err(_) => {},
        }

        fs::write(&self.output_file, &plaintext)
            .expect(&format!("Could not write to `{}`.", &self.output_file))
        ;

        println!(
            "Decrypted `{}`. {} bytes of plaintext written to {}.",
            &self.input_file,
            plaintext.len(),
            &self.output_file,
        );
    }
}
