use super::block;
use super::State;

pub trait MixColumns {
    fn mix_columns(&mut self);
    fn inv_mix_columns(&mut self);
}

impl MixColumns for State {
    fn mix_columns(&mut self) {
        let mut t: [u8; 4] = [0; 4];

        let gmul_1 = |e| e;
        let gmul_2 = |e| gmul(e, 0x02);
        let gmul_3 = |e| gmul(e, 0x03);

        for c in 0..block::COLUMN_COUNT {
            t[0] = self.elements[0][c];
            t[1] = self.elements[1][c];
            t[2] = self.elements[2][c];
            t[3] = self.elements[3][c];

            self.elements[0][c] = gmul_2(t[0]) ^ gmul_3(t[1]) ^ gmul_1(t[2]) ^ gmul_1(t[3]);
            self.elements[1][c] = gmul_1(t[0]) ^ gmul_2(t[1]) ^ gmul_3(t[2]) ^ gmul_1(t[3]);
            self.elements[2][c] = gmul_1(t[0]) ^ gmul_1(t[1]) ^ gmul_2(t[2]) ^ gmul_3(t[3]);
            self.elements[3][c] = gmul_3(t[0]) ^ gmul_1(t[1]) ^ gmul_1(t[2]) ^ gmul_2(t[3]);
        }
    }

    fn inv_mix_columns(&mut self) {
        let mut t: [u8; 4] = [0; 4];

        let gmul_9 = |e| gmul(e, 0x09);
        let gmul_11 = |e| gmul(e, 0x0B);
        let gmul_13 = |e| gmul(e, 0x0D);
        let gmul_14 = |e| gmul(e, 0x0E);

        for c in 0..block::COLUMN_COUNT {
            t[0] = self.elements[0][c];
            t[1] = self.elements[1][c];
            t[2] = self.elements[2][c];
            t[3] = self.elements[3][c];

            self.elements[0][c] = gmul_14(t[0]) ^ gmul_11(t[1]) ^ gmul_13(t[2]) ^ gmul_9(t[3]);
            self.elements[1][c] = gmul_9(t[0]) ^ gmul_14(t[1]) ^ gmul_11(t[2]) ^ gmul_13(t[3]);
            self.elements[2][c] = gmul_13(t[0]) ^ gmul_9(t[1]) ^ gmul_14(t[2]) ^ gmul_11(t[3]);
            self.elements[3][c] = gmul_11(t[0]) ^ gmul_13(t[1]) ^ gmul_9(t[2]) ^ gmul_14(t[3]);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::State;
    use super::MixColumns;

    #[test]
    fn mix_columns() {
        let mut state = State::new([
            [0xD4, 0xE0, 0xB8, 0x1E],
            [0xBF, 0xB4, 0x41, 0x27],
            [0x5D, 0x52, 0x11, 0x98],
            [0x30, 0xAE, 0xF1, 0xE5],
        ]);

        state.mix_columns();

        assert_eq!(state.elements, [
            [0x04, 0xE0, 0x48, 0x28],
            [0x66, 0xCB, 0xF8, 0x06],
            [0x81, 0x19, 0xD3, 0x26],
            [0xE5, 0x9A, 0x7A, 0x4C],
        ]);
    }

    #[test]
    fn inv_mix_columns() {
        let mut state = State::new([
            [0x04, 0xE0, 0x48, 0x28],
            [0x66, 0xCB, 0xF8, 0x06],
            [0x81, 0x19, 0xD3, 0x26],
            [0xE5, 0x9A, 0x7A, 0x4C],
        ]);

        state.inv_mix_columns();

        assert_eq!(state.elements, [
            [0xD4, 0xE0, 0xB8, 0x1E],
            [0xBF, 0xB4, 0x41, 0x27],
            [0x5D, 0x52, 0x11, 0x98],
            [0x30, 0xAE, 0xF1, 0xE5],
        ]);
    }
}
