use super::block;
use super::State;
use super::RoundKey;

pub trait AddRoundKey {
    fn add_round_key(&mut self, round_key: RoundKey);
}

impl AddRoundKey for State {
    fn add_round_key(&mut self, round_key: RoundKey) {
        for c in 0..block::COLUMN_COUNT {
            for r in 0..block::ROW_COUNT {
                self.elements[c][r] ^= round_key[c][r];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use super::AddRoundKey;

    #[test]
    fn add_round_key() {
        let mut state = State::new([
            [0x04, 0x66, 0x81, 0xE5],
            [0xE0, 0xCB, 0x19, 0x9A],
            [0x48, 0xF8, 0xD3, 0x7A],
            [0x28, 0x06, 0x26, 0x4C],
        ]);

        state.add_round_key([
            [0xA0, 0xFA, 0xFE, 0x17],
            [0x88, 0x54, 0x2C, 0xB1],
            [0x23, 0xA3, 0x39, 0x39],
            [0x2A, 0x6C, 0x76, 0x05],
        ]);

        assert_eq!(state.elements, [
            [0xA4, 0x9C, 0x7F, 0xF2],
            [0x68, 0x9F, 0x35, 0x2B],
            [0x6B, 0x5B, 0xEA, 0x43],
            [0x02, 0x6A, 0x50, 0x49],
        ]);
    }
}
