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

#[derive(Debug, Clone, Default)]
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

impl From<[u8; block::SIZE]> for State {
    fn from(array_elements: [u8; block::SIZE]) -> Self {
        let mut state = State::default();

        state.elements
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .zip(array_elements.iter())
            .for_each(|(s, v)| *s = *v)
        ;

        state
    }
}

impl From<State> for [u8; block::SIZE] {
    fn from(state: State) -> Self {
        let mut elements = [0; block::SIZE];

        state.elements
            .iter()
            .flat_map(|row| row.iter())
            .zip(&mut elements)
            .for_each(|(v, s)| *s = *v)
        ;

        elements
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

    #[test]
    fn from_1d_array() {
        let elements = [
            0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77,
            0x88, 0x99, 0xAA, 0xBB,
            0xCC, 0xDD, 0xEE, 0xFF,
        ];

        let state: State = elements.into();

        assert_eq!(state.elements, [
            [0x00, 0x11, 0x22, 0x33],
            [0x44, 0x55, 0x66, 0x77],
            [0x88, 0x99, 0xAA, 0xBB],
            [0xCC, 0xDD, 0xEE, 0xFF],
        ]);
    }
}
