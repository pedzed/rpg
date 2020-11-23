pub struct SubBytes;

use super::super::state::State;
use super::super::sbox_tables::{SBOX, INV_SBOX};

impl SubBytes {
    pub fn sub_bytes(state: State) -> State {
        state
            .iter()
            .map(|&e| SBOX[e as usize])
            .collect()
    }

    pub fn inv_sub_bytes(state: State) -> State {
        state
            .iter()
            .map(|&e| INV_SBOX[e as usize])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use super::SubBytes;

    #[test]
    fn sub_bytes() {
        let state = SubBytes::sub_bytes(State::new([
            0x19, 0xA0, 0x9A, 0xE9,
            0x3D, 0xF4, 0xC6, 0xF8,
            0xE3, 0xE2, 0x8D, 0x48,
            0xBE, 0x2B, 0x2A, 0x08,
        ]));

        assert_eq!(state, State::new([
            0xD4, 0xE0, 0xB8, 0x1E,
            0x27, 0xBF, 0xB4, 0x41,
            0x11, 0x98, 0x5D, 0x52,
            0xAE, 0xF1, 0xE5, 0x30,
        ]));
    }

    #[test]
    fn inv_sub_bytes() {
        let state = SubBytes::inv_sub_bytes(State::new([
            0xD4, 0xE0, 0xB8, 0x1E,
            0x27, 0xBF, 0xB4, 0x41,
            0x11, 0x98, 0x5D, 0x52,
            0xAE, 0xF1, 0xE5, 0x30,
        ]));

        assert_eq!(state, State::new([
            0x19, 0xA0, 0x9A, 0xE9,
            0x3D, 0xF4, 0xC6, 0xF8,
            0xE3, 0xE2, 0x8D, 0x48,
            0xBE, 0x2B, 0x2A, 0x08,
        ]));
    }
}
