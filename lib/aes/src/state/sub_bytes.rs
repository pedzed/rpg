use super::block;
use super::State;
use super::super::sbox_tables::{SBOX, INV_SBOX};

pub trait SubBytes {
    fn sub_bytes(&mut self);
    fn inv_sub_bytes(&mut self);
}

impl SubBytes for State {
    fn sub_bytes(&mut self) {
        for c in 0..block::COLUMN_COUNT {
            for r in 0..block::ROW_COUNT {
                self.elements[c][r] = SBOX[self.elements[c][r] as usize];
            }
        }
    }

    fn inv_sub_bytes(&mut self) {
        for c in 0..block::COLUMN_COUNT {
            for r in 0..block::ROW_COUNT {
                self.elements[c][r] = INV_SBOX[self.elements[c][r] as usize];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use super::SubBytes;

    #[test]
    fn sub_bytes() {
        let mut state = State::new([
            [0x19, 0x3D, 0xE3, 0xBE],
            [0xA0, 0xF4, 0xE2, 0x2B],
            [0x9A, 0xC6, 0x8D, 0x2A],
            [0xE9, 0xF8, 0x48, 0x08],
        ]);

        state.sub_bytes();

        assert_eq!(state.elements, [
            [0xD4, 0x27, 0x11, 0xAE],
            [0xE0, 0xBF, 0x98, 0xF1],
            [0xB8, 0xB4, 0x5D, 0xE5],
            [0x1E, 0x41, 0x52, 0x30],
        ]);
    }

    #[test]
    fn inv_sub_bytes() {
        let mut state = State::new([
            [0xD4, 0x27, 0x11, 0xAE],
            [0xE0, 0xBF, 0x98, 0xF1],
            [0xB8, 0xB4, 0x5D, 0xE5],
            [0x1E, 0x41, 0x52, 0x30],
        ]);

        state.inv_sub_bytes();

        assert_eq!(state.elements, [
            [0x19, 0x3D, 0xE3, 0xBE],
            [0xA0, 0xF4, 0xE2, 0x2B],
            [0x9A, 0xC6, 0x8D, 0x2A],
            [0xE9, 0xF8, 0x48, 0x08],
        ]);
    }
}
