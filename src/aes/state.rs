#[derive(Debug, PartialEq)]
pub struct State {
    pub rows: usize,
    pub columns: usize,
    pub data: Vec<u8>,
}

impl State {
    pub fn new(data: &[u8]) -> Self {
        Self {
            rows: 4,
            columns: 4,
            data: data.to_vec(),
        }
    }
}

impl From<Vec<u8>> for State {
    fn from(state: Vec<u8>) -> Self {
        Self::new(&state)
    }
}
