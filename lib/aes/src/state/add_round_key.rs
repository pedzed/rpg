use super::State;
use super::RoundKey;

pub trait AddRoundKey {
    fn add_round_key(&mut self, round_key: RoundKey);
}

impl AddRoundKey for State {
    fn add_round_key(&mut self, round_key: RoundKey) {
        self.elements[0][0] ^= round_key[0][0];
        self.elements[0][1] ^= round_key[0][1];
        self.elements[0][2] ^= round_key[0][2];
        self.elements[0][3] ^= round_key[0][3];

        self.elements[1][0] ^= round_key[1][0];
        self.elements[1][1] ^= round_key[1][1];
        self.elements[1][2] ^= round_key[1][2];
        self.elements[1][3] ^= round_key[1][3];

        self.elements[2][0] ^= round_key[2][0];
        self.elements[2][1] ^= round_key[2][1];
        self.elements[2][2] ^= round_key[2][2];
        self.elements[2][3] ^= round_key[2][3];

        self.elements[3][0] ^= round_key[3][0];
        self.elements[3][1] ^= round_key[3][1];
        self.elements[3][2] ^= round_key[3][2];
        self.elements[3][3] ^= round_key[3][3];
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
