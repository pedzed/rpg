pub mod crypto;

use std::fs;

use ascii_armor::ArmorWriterBuilder;
use ascii_armor::ArmorDataHeader;
use ascii_armor::ArmorDataType;
use ascii_armor::ArmorReader;

use crypto::openpgp_cfb::OpenPgpCfbAes128;

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


        if self.with_armor {
            let mut buffer = fs::File::create(&self.output_file).unwrap();

            let armor = ArmorWriterBuilder::new()
                .data_type(ArmorDataType::PgpMessage)
                .add_data_header(
                    ArmorDataHeader::Version,
                    &format!("{} v{}", APP_NAME, APP_VERSION)
                )
                .data(&ciphertext)
                .build()
            ;

            armor.write_unchecked(&mut buffer).unwrap();
        } else {
            fs::write(&self.output_file, &ciphertext)
                .expect(&format!("Could not write to `{}`.", &self.output_file))
            ;
        }

        println!(
            "Successfully encrypted `{}`. {} bytes of ciphertext written to {}.",
            &self.input_file,
            ciphertext.len(),
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
                                std::thread::sleep(std::time::Duration::from_millis(2500));
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

        let plaintext = OpenPgpCfbAes128::decrypt(&ciphertext, &self.cipher_key)
            .expect("Failed to encrypt.")
        ;

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
