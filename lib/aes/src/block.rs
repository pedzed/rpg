/// An AES block, consisting of 4x4 bytes
pub type Block = [[u8; ROW_COUNT]; COLUMN_COUNT];

pub const ROW_COUNT: usize = 4;
pub const COLUMN_COUNT: usize = 4;
pub const SIZE: usize = ROW_COUNT * COLUMN_COUNT;
