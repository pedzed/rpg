use std::env;

use rpg::{APP_NAME, APP_VERSION, DecryptionCommand};
use rpg::{EncryptionCommand, SymmetricCipher};

fn main() {
    let mut args: Vec<String> = env::args().collect();

    let _program = args.remove(0);

    if args.len() == 0 {
        println!("NOTE: No command supplied. Showing help instead...\n");
        println!("{}", get_help_text());
        return;
    }

    if let Some(arg) = args.get(0) {
        if arg == "--help" || arg == "-h" {
            println!("{}", get_help_text());
        } else if arg == "--encrypt" || arg == "--decrypt" {
            let algo = args.get(1)
                .expect(&format!("Expected algorithm. None provided."))
            ;
            let algo = SymmetricCipher::from_str(algo)
                .expect(&format!("Unknown algorithm `{}`.", algo))
            ;

            let input_file = args.get(2)
                .expect(&format!("Expected input file. None provided."))
            ;

            let with_armor = args.contains(&String::from("--armor"));

            if arg == "--encrypt" {
                let output_file = format!("{}.rpg", input_file);

                EncryptionCommand {
                    algo,
                    input_file: String::from(input_file),
                    output_file,
                    cipher_key: (0x112233445566778899AABBCCDDEEFF as u128).to_be_bytes().to_vec(),
                    with_armor,
                }.run();
            } else {
                let output_file = format!("{}.decrypted", input_file);

                DecryptionCommand {
                    algo,
                    input_file: String::from(input_file),
                    output_file,
                    cipher_key: (0x112233445566778899AABBCCDDEEFF as u128).to_be_bytes().to_vec(),
                }.run();
            }
        } else {
            panic!("Unknown argument `{}` provided. Run --help for available commands.", arg);
        }
    }
}

fn get_help_text() -> String {
    format!(
        "\
{app_name} v{app_version}

    Commands:
    -h, --help              This help.
    --encrypt               Encrypt a file. Available ciphers:
                            aes128, aes192, aes256

                            Example usage:
                            {app_bin} --encrypt aes128 input.txt [--armor]

    --decrypt               Decrypt a file. Available ciphers:
                            aes128, aes192, aes256

                            Example usage:
                            {app_bin} --decrypt aes128 input.txt.rpg

    Options:
    --armor                 Apply Radix-64 encoding in ASCII Armor. Useful for
                            transferring data in binary-safe format.

                            To be used with the --encrypt command.
\
        ",
        app_bin=APP_NAME,
        app_name=APP_NAME,
        app_version=APP_VERSION,
    )
}
