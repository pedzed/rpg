pub struct ShiftRows;

use super::state::State;

impl ShiftRows {
    pub fn shift_rows(state: State) -> State {
        let mut new_state = Vec::with_capacity(16);

        new_state.push(state.data[0]);
        new_state.push(state.data[1]);
        new_state.push(state.data[2]);
        new_state.push(state.data[3]);

        new_state.push(state.data[5]);
        new_state.push(state.data[6]);
        new_state.push(state.data[7]);
        new_state.push(state.data[4]);

        new_state.push(state.data[10]);
        new_state.push(state.data[11]);
        new_state.push(state.data[8]);
        new_state.push(state.data[9]);

        new_state.push(state.data[15]);
        new_state.push(state.data[12]);
        new_state.push(state.data[13]);
        new_state.push(state.data[14]);

        new_state.into()
    }

    pub fn inv_shift_rows(state: State) -> State {
        let mut new_state = Vec::with_capacity(16);

        new_state.push(state.data[0]);
        new_state.push(state.data[1]);
        new_state.push(state.data[2]);
        new_state.push(state.data[3]);

        new_state.push(state.data[7]);
        new_state.push(state.data[4]);
        new_state.push(state.data[5]);
        new_state.push(state.data[6]);

        new_state.push(state.data[10]);
        new_state.push(state.data[11]);
        new_state.push(state.data[8]);
        new_state.push(state.data[9]);

        new_state.push(state.data[13]);
        new_state.push(state.data[14]);
        new_state.push(state.data[15]);
        new_state.push(state.data[12]);

        new_state.into()
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use super::ShiftRows;

    #[test]
    fn shift_rows() {
        let state = ShiftRows::shift_rows(State::new(&[
            0xD4, 0xE0, 0xB8, 0x1E,
            0x27, 0xBF, 0xB4, 0x41,
            0x11, 0x98, 0x5D, 0x52,
            0xAE, 0xF1, 0xE5, 0x30,
        ]));

        assert_eq!(state, State::new(&[
            0xD4, 0xE0, 0xB8, 0x1E,
            0xBF, 0xB4, 0x41, 0x27,
            0x5D, 0x52, 0x11, 0x98,
            0x30, 0xAE, 0xF1, 0xE5,
        ]));
    }

    #[test]
    fn inv_shift_rows() {
        let state = ShiftRows::inv_shift_rows(State::new(&[
            0xD4, 0xE0, 0xB8, 0x1E,
            0xBF, 0xB4, 0x41, 0x27,
            0x5D, 0x52, 0x11, 0x98,
            0x30, 0xAE, 0xF1, 0xE5,
        ]));

        assert_eq!(state, State::new(&[
            0xD4, 0xE0, 0xB8, 0x1E,
            0x27, 0xBF, 0xB4, 0x41,
            0x11, 0x98, 0x5D, 0x52,
            0xAE, 0xF1, 0xE5, 0x30,
        ]));
    }
}
