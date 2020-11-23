pub struct MixColumns;

use super::super::state::State;

impl MixColumns {
    pub fn mix_columns(state: State) -> State {
        let gmul_1 = |e| e;
        let gmul_2 = |e| Self::gmul(e, 0x02);
        let gmul_3 = |e| Self::gmul(e, 0x03);

        State::new([
            gmul_2(state[0]) ^ gmul_3(state[4]) ^ gmul_1(state[8]) ^ gmul_1(state[12]),
            gmul_2(state[1]) ^ gmul_3(state[5]) ^ gmul_1(state[9]) ^ gmul_1(state[13]),
            gmul_2(state[2]) ^ gmul_3(state[6]) ^ gmul_1(state[10]) ^ gmul_1(state[14]),
            gmul_2(state[3]) ^ gmul_3(state[7]) ^ gmul_1(state[11]) ^ gmul_1(state[15]),

            gmul_1(state[0]) ^ gmul_2(state[4]) ^ gmul_3(state[8]) ^ gmul_1(state[12]),
            gmul_1(state[1]) ^ gmul_2(state[5]) ^ gmul_3(state[9]) ^ gmul_1(state[13]),
            gmul_1(state[2]) ^ gmul_2(state[6]) ^ gmul_3(state[10]) ^ gmul_1(state[14]),
            gmul_1(state[3]) ^ gmul_2(state[7]) ^ gmul_3(state[11]) ^ gmul_1(state[15]),

            gmul_1(state[0]) ^ gmul_1(state[4]) ^ gmul_2(state[8]) ^ gmul_3(state[12]),
            gmul_1(state[1]) ^ gmul_1(state[5]) ^ gmul_2(state[9]) ^ gmul_3(state[13]),
            gmul_1(state[2]) ^ gmul_1(state[6]) ^ gmul_2(state[10]) ^ gmul_3(state[14]),
            gmul_1(state[3]) ^ gmul_1(state[7]) ^ gmul_2(state[11]) ^ gmul_3(state[15]),

            gmul_3(state[0]) ^ gmul_1(state[4]) ^ gmul_1(state[8]) ^ gmul_2(state[12]),
            gmul_3(state[1]) ^ gmul_1(state[5]) ^ gmul_1(state[9]) ^ gmul_2(state[13]),
            gmul_3(state[2]) ^ gmul_1(state[6]) ^ gmul_1(state[10]) ^ gmul_2(state[14]),
            gmul_3(state[3]) ^ gmul_1(state[7]) ^ gmul_1(state[11]) ^ gmul_2(state[15]),
        ])
    }

    /// Secure Galois Field (2^8) multiplication of two bytes
    ///
    /// Does not use table lookups and does not contain branches to prevent
    /// side-channel attacks.
    fn gmul(a: u8, b: u8) -> u8 {
        let mut a: i16 = a as i16;
        let mut b: i16 = b as i16;
        let mut output: i16 = 0;

        for _ in 0..8 {
            output ^= -(b & 1) & a;
            let mask = -((a >> 7) & 1);
            // 0b1_0001_1011 is x^8 + x^4 + x^3 + x + 1.
            a = (a << 1) ^ (0b1_0001_1011 & mask);
            b >>= 1;
        }

        output as u8
    }

    pub fn inv_mix_columns(state: State) -> State {
        let gmul_14 = |e| Self::gmul(e, 0x0E);
        let gmul_11 = |e| Self::gmul(e, 0x0B);
        let gmul_13 = |e| Self::gmul(e, 0x0D);
        let gmul_9 = |e| Self::gmul(e, 0x09);

        State::new([
            gmul_14(state[0]) ^ gmul_11(state[4]) ^ gmul_13(state[8]) ^ gmul_9(state[12]),
            gmul_14(state[1]) ^ gmul_11(state[5]) ^ gmul_13(state[9]) ^ gmul_9(state[13]),
            gmul_14(state[2]) ^ gmul_11(state[6]) ^ gmul_13(state[10]) ^ gmul_9(state[14]),
            gmul_14(state[3]) ^ gmul_11(state[7]) ^ gmul_13(state[11]) ^ gmul_9(state[15]),

            gmul_9(state[0]) ^ gmul_14(state[4]) ^ gmul_11(state[8]) ^ gmul_13(state[12]),
            gmul_9(state[1]) ^ gmul_14(state[5]) ^ gmul_11(state[9]) ^ gmul_13(state[13]),
            gmul_9(state[2]) ^ gmul_14(state[6]) ^ gmul_11(state[10]) ^ gmul_13(state[14]),
            gmul_9(state[3]) ^ gmul_14(state[7]) ^ gmul_11(state[11]) ^ gmul_13(state[15]),

            gmul_13(state[0]) ^ gmul_9(state[4]) ^ gmul_14(state[8]) ^ gmul_11(state[12]),
            gmul_13(state[1]) ^ gmul_9(state[5]) ^ gmul_14(state[9]) ^ gmul_11(state[13]),
            gmul_13(state[2]) ^ gmul_9(state[6]) ^ gmul_14(state[10]) ^ gmul_11(state[14]),
            gmul_13(state[3]) ^ gmul_9(state[7]) ^ gmul_14(state[11]) ^ gmul_11(state[15]),

            gmul_11(state[0]) ^ gmul_13(state[4]) ^ gmul_9(state[8]) ^ gmul_14(state[12]),
            gmul_11(state[1]) ^ gmul_13(state[5]) ^ gmul_9(state[9]) ^ gmul_14(state[13]),
            gmul_11(state[2]) ^ gmul_13(state[6]) ^ gmul_9(state[10]) ^ gmul_14(state[14]),
            gmul_11(state[3]) ^ gmul_13(state[7]) ^ gmul_9(state[11]) ^ gmul_14(state[15]),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use super::MixColumns;

    #[test]
    fn mix_columns() {
        let state = MixColumns::mix_columns(State::new([
            0xD4, 0xE0, 0xB8, 0x1E,
            0xBF, 0xB4, 0x41, 0x27,
            0x5D, 0x52, 0x11, 0x98,
            0x30, 0xAE, 0xF1, 0xE5,
        ]));

        assert_eq!(state, State::new([
            0x04, 0xE0, 0x48, 0x28,
            0x66, 0xCB, 0xF8, 0x06,
            0x81, 0x19, 0xD3, 0x26,
            0xE5, 0x9A, 0x7A, 0x4C,
        ]));
    }

    #[test]
    fn inv_mix_columns() {
        let state = MixColumns::inv_mix_columns(State::new([
            0x04, 0xE0, 0x48, 0x28,
            0x66, 0xCB, 0xF8, 0x06,
            0x81, 0x19, 0xD3, 0x26,
            0xE5, 0x9A, 0x7A, 0x4C,
        ]));

        assert_eq!(state, State::new([
            0xD4, 0xE0, 0xB8, 0x1E,
            0xBF, 0xB4, 0x41, 0x27,
            0x5D, 0x52, 0x11, 0x98,
            0x30, 0xAE, 0xF1, 0xE5,
        ]));
    }
}
