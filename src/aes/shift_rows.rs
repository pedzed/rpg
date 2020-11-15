pub struct ShiftRows;

impl ShiftRows {
    pub fn shift_rows(state: &[u8]) -> Vec<u8> {
        let mut new_state = Vec::with_capacity(16);

        new_state.push(state[0]);
        new_state.push(state[1]);
        new_state.push(state[2]);
        new_state.push(state[3]);

        new_state.push(state[5]);
        new_state.push(state[6]);
        new_state.push(state[7]);
        new_state.push(state[4]);

        new_state.push(state[10]);
        new_state.push(state[11]);
        new_state.push(state[8]);
        new_state.push(state[9]);

        new_state.push(state[15]);
        new_state.push(state[12]);
        new_state.push(state[13]);
        new_state.push(state[14]);

        new_state
    }

    pub fn inv_shift_rows(state: &[u8]) -> Vec<u8> {
        let mut new_state = Vec::with_capacity(16);

        new_state.push(state[0]);
        new_state.push(state[1]);
        new_state.push(state[2]);
        new_state.push(state[3]);

        new_state.push(state[7]);
        new_state.push(state[4]);
        new_state.push(state[5]);
        new_state.push(state[6]);

        new_state.push(state[10]);
        new_state.push(state[11]);
        new_state.push(state[8]);
        new_state.push(state[9]);

        new_state.push(state[13]);
        new_state.push(state[14]);
        new_state.push(state[15]);
        new_state.push(state[12]);

        new_state
    }
}

#[cfg(test)]
mod tests {
    use super::ShiftRows;

    #[test]
    fn shift_rows() {
        let state = ShiftRows::shift_rows(&[
            0xD4, 0xE0, 0xB8, 0x1E,
            0x27, 0xBF, 0xB4, 0x41,
            0x11, 0x98, 0x5D, 0x52,
            0xAE, 0xF1, 0xE5, 0x30,
        ]);

        assert_eq!(state, &[
            0xD4, 0xE0, 0xB8, 0x1E,
            0xBF, 0xB4, 0x41, 0x27,
            0x5D, 0x52, 0x11, 0x98,
            0x30, 0xAE, 0xF1, 0xE5,
        ]);
    }

    #[test]
    fn inv_shift_rows() {
        let state = ShiftRows::inv_shift_rows(&[
            0xD4, 0xE0, 0xB8, 0x1E,
            0xBF, 0xB4, 0x41, 0x27,
            0x5D, 0x52, 0x11, 0x98,
            0x30, 0xAE, 0xF1, 0xE5,
        ]);

        assert_eq!(state, &[
            0xD4, 0xE0, 0xB8, 0x1E,
            0x27, 0xBF, 0xB4, 0x41,
            0x11, 0x98, 0x5D, 0x52,
            0xAE, 0xF1, 0xE5, 0x30,
        ]);
    }
}
