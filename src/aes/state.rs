use std::ops::Index;
use std::slice::Iter;
use std::iter::FromIterator;

type Row = usize;
type Column = usize;

pub type Element = u8;
pub type Elements = [Element; State::CAPACITY];

/// The State of an AES block
///
/// Stored as a one-dimensional list of bytes.
///
/// | 0  1  2  3|
/// | 4  5  6  7|
/// | 8  9 10 11|
/// |12 13 14 15|
#[derive(Debug, PartialEq, Default)]
pub struct State {
    elements: Elements,
}

impl State {
    const ROWS: Row = 4;
    const COLUMNS: Column = 4;
    pub const CAPACITY: usize = Self::ROWS * Self::COLUMNS;

    pub fn new(elements: Elements) -> Self {
        Self {
            elements,
        }
    }

    pub fn iter(&self) -> Iter<Element> {
        self.elements.iter()
    }
}

impl Index<(Row, Column)> for State {
    type Output = Element;

    fn index(&self, index: (Row, Column)) -> &Self::Output {
        let (row, col) = index;
        &self.elements[col + row * 4]
    }
}

impl Index<usize> for State {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}

impl FromIterator<Element> for State {
    fn from_iter<I: IntoIterator<Item=Element>>(iter: I) -> Self {
        let mut state = Self::default();

        iter.into_iter()
            .zip(&mut state.elements)
            .for_each(|(value, slot)| *slot = value);

        state
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn can_index_by_single_indice() {
        let state = State::new([
            0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0E,
        ]);

        assert_eq!(state[7], 0x07);
    }

    #[test]
    fn can_index_top_left() {
        let state = State::new([
            0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0E,
        ]);

        assert_eq!(state[(0, 0)], 0x00);
    }

    #[test]
    fn can_index_top_right() {
        let state = State::new([
            0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0E,
        ]);

        assert_eq!(state[(0, 3)], 0x03);
    }

    #[test]
    fn can_index_bottom_left() {
        let state = State::new([
            0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0E,
        ]);

        assert_eq!(state[(3, 0)], 0x0C);
    }

    #[test]
    fn can_index_bottom_right() {
        let state = State::new([
            0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0E,
        ]);

        assert_eq!(state[(3, 3)], 0x0E);
    }
}
