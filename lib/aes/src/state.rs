mod add_round_key;
mod sub_bytes;
mod shift_rows;
mod mix_columns;

use super::block;
use super::block::Block;
use super::round_key::RoundKey;

pub use super::state::add_round_key::AddRoundKey;
pub use super::state::sub_bytes::SubBytes;
pub use super::state::shift_rows::ShiftRows;
pub use super::state::mix_columns::MixColumns;

#[derive(Debug, Default)]
pub struct State {
    pub elements: Block,
}

impl State {
    pub fn new(elements: Block) -> Self {
        Self {
            elements,
        }
    }
}

impl From<State> for [u8; block::SIZE] {
    fn from(state: State) -> Self {
        let mut elements = [0; block::SIZE];

        let mut i: usize = 0;

        for r in 0..block::ROW_COUNT {
            for c in 0..block::COLUMN_COUNT {
                elements[i] = state.elements[r][c];
                i += 1;
            }
        }

        elements

        // [
        //     state.elements[0][0],
        //     state.elements[0][1],
        //     state.elements[0][2],
        //     state.elements[0][3],

        //     state.elements[1][0],
        //     state.elements[1][1],
        //     state.elements[1][2],
        //     state.elements[1][3],

        //     state.elements[2][0],
        //     state.elements[2][1],
        //     state.elements[2][2],
        //     state.elements[2][3],

        //     state.elements[3][0],
        //     state.elements[3][1],
        //     state.elements[3][2],
        //     state.elements[3][3],
        // ]
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn flatten() {
        let state = State::new([
            [0x00, 0x11, 0x22, 0x33],
            [0x44, 0x55, 0x66, 0x77],
            [0x88, 0x99, 0xAA, 0xBB],
            [0xCC, 0xDD, 0xEE, 0xFF],
        ]);

        let state: [u8; 16] = state.into();

        assert_eq!(state, [
            0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77,
            0x88, 0x99, 0xAA, 0xBB,
            0xCC, 0xDD, 0xEE, 0xFF,
        ]);
    }
}
