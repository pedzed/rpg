use std::path::Path;

fn main() {
    constant_time::measure_xor(1_000_000, Path::new("measurements/xor-1_000_000.png"));
    constant_time::measure_aes_soft(1_000_000, Path::new("measurements/aes_soft-1_000_000.png"));
    constant_time::measure_aes_add_round_key(1_000_000, Path::new("measurements/aes_add_round_key-1_000_000.png"));
}
