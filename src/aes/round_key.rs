use std::ops::Index;

pub type Element = u8;
pub type Elements = [Element; RoundKey::CAPACITY];

type Row = usize;
type Column = usize;

/// The RoundKey of an AES block
///
/// Stored as a one-dimensional list of bytes.
///
/// | 0  1  2  3|
/// | 4  5  6  7|
/// | 8  9 10 11|
/// |12 13 14 15|
#[derive(Debug, PartialEq)]
pub struct RoundKey {
    elements: Elements,
}

impl Index<usize> for RoundKey {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}

impl RoundKey {
    const ROWS: Row = 4;
    const COLUMNS: Column = 4;
    pub const CAPACITY: usize = Self::ROWS * Self::COLUMNS;

    pub fn new(elements: Elements) -> Self {
        Self {
            elements,
        }
    }
}
