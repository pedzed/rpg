pub mod crypto;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::BufReader;

use ascii_armor::ArmorError;
use ascii_armor::ArmorWriterBuilder;
use ascii_armor::ArmorDataHeader;
use ascii_armor::ArmorDataType;
use ascii_armor::ArmorReader;

use crypto::aes::Aes128;
use crypto::mode_of_operations::Mode;
use crypto::cfb::CfbAes128;
use crypto::openpgp_cfb::OpenPgpCfbAes128;
use crypto::symmetric_cipher::BlockCipher;
use crypto::symmetric_cipher::SymmetricDecryption;
use crypto::symmetric_cipher::SymmetricEncryption;
use crypto::symmetric_cipher::SymmetricKey;

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
    pub mode: Mode,
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

        let ciphertext = Aes128::new()
            .with_key(&self.cipher_key)
            .using_mode(&self.mode)
            .encrypt(&plaintext)
            .unwrap()
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
    pub mode: Mode,
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

        let mut file = File::open(&self.input_file).unwrap();

        let armor = self.read_armor(&file);

        let plaintext = match armor {
            Ok(armor) => match self.decrypt_armor(armor) {
                Ok(plaintext) => plaintext,
                Err(_) => self.decrypt_file_without_armor(&mut file).unwrap(),
            },
            Err(_) => self.decrypt_file_without_armor(&mut file).unwrap(),
        };

        println!("✓ Decrypted `{}` ({} bytes).", &self.input_file, plaintext.len());

        println!("Writing to `{}`...", &self.output_file);
        fs::write(&self.output_file, &plaintext).unwrap();

        println!("✓ Finished.");
    }

    fn read_armor(&self, file: &File) -> Result<ArmorReader, ArmorError> {
        let mut reader = BufReader::new(file);
        ArmorReader::read(&mut reader)
    }

    fn decrypt_armor(&self, armor: ArmorReader) -> Result<Vec<u8>, Error> {
        let ciphertext = armor.data?;
        let plaintext = Aes128::new()
            .with_key(&self.cipher_key)
            .using_mode(&self.mode)
            .decrypt(&ciphertext)
            .unwrap()
        ;

        let checksum = armor.checksum?;

        match checksum.verify(&ciphertext) {
            true => println!("✓ Checksum verification passed."),
            false => match self.ignore_crc_error {
                true => {
                    println!("✗ Checksum verification of `{}` failed. Ignoring...", checksum.get());
                    std::thread::sleep(std::time::Duration::from_millis(2500));
                },
                false => {
                    panic!("✗ Checksum verification of `{}` failed. Aborting.", checksum.get());
                },
            },
        }

        Ok(plaintext)
    }

    fn decrypt_file_without_armor(&self, file: &mut File) -> Result<Vec<u8>, Error> {
        // let mut ciphertext = Vec::with_capacity(file.metadata()?.len() as usize);
        // file.read_to_end(&mut ciphertext)?;

        let ciphertext = std::fs::read(&self.input_file)?;

        let plaintext = Aes128::new()
            .with_key(&self.cipher_key)
            .using_mode(&self.mode)
            .decrypt(&ciphertext)
            .unwrap()
        ;

        Ok(plaintext)
    }
}
