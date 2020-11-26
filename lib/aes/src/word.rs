
use super::sbox_tables::SBOX;

pub type Word = [u8; 4];

pub trait SubWord {
    fn sub_word(&mut self);
}

impl SubWord for Word {
    fn sub_word(&mut self) {
        let temp = self.clone();

        for (i, &element) in temp.iter().enumerate() {
            self[i] = SBOX[element as usize];
        }
    }
}

pub trait RotWord {
    fn rot_word(&mut self);
}

impl RotWord for Word {
    fn rot_word(&mut self) {
        self.rotate_left(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_word() {
        let mut word = [0x2B, 0x7E, 0x15, 0x16];
        word.sub_word();

        assert_eq!(word, [0xF1, 0xF3, 0x59, 0x47]);
    }

    #[test]
    fn rot_word() {
        let mut word = [0xF1, 0xF3, 0x59, 0x47];
        word.rot_word();

        assert_eq!(word, [0xF3, 0x59, 0x47, 0xF1]);
    }
}
