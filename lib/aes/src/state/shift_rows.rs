use super::block;
use super::State;

pub trait ShiftRows {
    fn shift_rows(&mut self);
    fn inv_shift_rows(&mut self);
}

impl ShiftRows for State {
    fn shift_rows(&mut self) {
        let temp = self.elements;

        for c in 0..block::COLUMN_COUNT {
            for r in 1..block::ROW_COUNT {
                self.elements[c][r] = temp[(c + r) % block::COLUMN_COUNT][r];
            }
        }
    }

    fn inv_shift_rows(&mut self) {
        let mut temp = self.elements;

        for c in 0..block::COLUMN_COUNT {
            for r in 1..block::ROW_COUNT {
                temp[(c + r) % block::COLUMN_COUNT][r] = self.elements[c][r];
            }
        }

        self.elements = temp;
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use super::ShiftRows;

    #[test]
    fn shift_rows() {
        let mut state = State::new([
            [0xD4, 0x27, 0x11, 0xAE],
            [0xE0, 0xBF, 0x98, 0xF1],
            [0xB8, 0xB4, 0x5D, 0xE5],
            [0x1E, 0x41, 0x52, 0x30],
        ]);

        state.shift_rows();

        assert_eq!(state.elements, [
            [0xD4, 0xBF, 0x5D, 0x30],
            [0xE0, 0xB4, 0x52, 0xAE],
            [0xB8, 0x41, 0x11, 0xF1],
            [0x1E, 0x27, 0x98, 0xE5],
        ]);
    }

    #[test]
    fn inv_shift_rows() {
        let mut state = State::new([
            [0xD4, 0xBF, 0x5D, 0x30],
            [0xE0, 0xB4, 0x52, 0xAE],
            [0xB8, 0x41, 0x11, 0xF1],
            [0x1E, 0x27, 0x98, 0xE5],
        ]);

        state.inv_shift_rows();

        assert_eq!(state.elements, [
            [0xD4, 0x27, 0x11, 0xAE],
            [0xE0, 0xBF, 0x98, 0xF1],
            [0xB8, 0xB4, 0x5D, 0xE5],
            [0x1E, 0x41, 0x52, 0x30],
        ]);
    }
}
