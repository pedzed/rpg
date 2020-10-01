use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    for (i, arg) in args.iter().enumerate() {
        if arg == "--crc" {
            match env::args().nth(i + 1) {
                Some(input) => rpg::run_crc(&input),
                None => panic!("No input argument given for CRC."),
            }
        }
    }
}
