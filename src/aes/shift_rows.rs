pub struct ShiftRows;

use super::state::State;

impl ShiftRows {
    pub fn shift_rows(state: State) -> State {
        State::new([
            state[0],   state[1],   state[2],   state[3],
            state[5],   state[6],   state[7],   state[4],
            state[10],  state[11],  state[8],   state[9],
            state[15],  state[12],  state[13],  state[14],
        ])
    }

    pub fn inv_shift_rows(state: State) -> State {
        State::new([
            state[0],   state[1],   state[2],   state[3],
            state[7],   state[4],   state[5],   state[6],
            state[10],  state[11],  state[8],   state[9],
            state[13],  state[14],  state[15],  state[12],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use super::ShiftRows;

    #[test]
    fn shift_rows() {
        let state = ShiftRows::shift_rows(State::new([
            0xD4, 0xE0, 0xB8, 0x1E,
            0x27, 0xBF, 0xB4, 0x41,
            0x11, 0x98, 0x5D, 0x52,
            0xAE, 0xF1, 0xE5, 0x30,
        ]));

        assert_eq!(state, State::new([
            0xD4, 0xE0, 0xB8, 0x1E,
            0xBF, 0xB4, 0x41, 0x27,
            0x5D, 0x52, 0x11, 0x98,
            0x30, 0xAE, 0xF1, 0xE5,
        ]));
    }

    #[test]
    fn inv_shift_rows() {
        let state = ShiftRows::inv_shift_rows(State::new([
            0xD4, 0xE0, 0xB8, 0x1E,
            0xBF, 0xB4, 0x41, 0x27,
            0x5D, 0x52, 0x11, 0x98,
            0x30, 0xAE, 0xF1, 0xE5,
        ]));

        assert_eq!(state, State::new([
            0xD4, 0xE0, 0xB8, 0x1E,
            0x27, 0xBF, 0xB4, 0x41,
            0x11, 0x98, 0x5D, 0x52,
            0xAE, 0xF1, 0xE5, 0x30,
        ]));
    }
}
