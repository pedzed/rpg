use super::block;
use super::State;

pub trait ShiftRows {
    fn shift_rows(&mut self);
    fn inv_shift_rows(&mut self);
}

impl ShiftRows for State {
    fn shift_rows(&mut self) {
        let temp = self.elements;

        for r in 1..block::ROW_COUNT {
            for c in 0..block::COLUMN_COUNT {
                self.elements[r][c] = temp[r][(r + c) % 4];
            }
        }
    }

    fn inv_shift_rows(&mut self) {
        let mut temp = self.elements;

        for r in 1..block::ROW_COUNT {
            for c in 0..block::COLUMN_COUNT {
                temp[r][(r + c) % 4] = self.elements[r][c];
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
            [0xD4, 0xE0, 0xB8, 0x1E],
            [0x27, 0xBF, 0xB4, 0x41],
            [0x11, 0x98, 0x5D, 0x52],
            [0xAE, 0xF1, 0xE5, 0x30],
        ]);

        state.shift_rows();

        assert_eq!(state.elements, [
            [0xD4, 0xE0, 0xB8, 0x1E],
            [0xBF, 0xB4, 0x41, 0x27],
            [0x5D, 0x52, 0x11, 0x98],
            [0x30, 0xAE, 0xF1, 0xE5],
        ]);
    }

    #[test]
    fn inv_shift_rows() {
        let mut state = State::new([
            [0xD4, 0xE0, 0xB8, 0x1E],
            [0xBF, 0xB4, 0x41, 0x27],
            [0x5D, 0x52, 0x11, 0x98],
            [0x30, 0xAE, 0xF1, 0xE5],
        ]);

        state.inv_shift_rows();

        assert_eq!(state.elements, [
            [0xD4, 0xE0, 0xB8, 0x1E],
            [0x27, 0xBF, 0xB4, 0x41],
            [0x11, 0x98, 0x5D, 0x52],
            [0xAE, 0xF1, 0xE5, 0x30],
        ]);
    }
}
