use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.col,)
    }
}

#[derive(Debug)]
pub struct Located<T> {
    pub data: T,
    pub loc: Location,
}

impl<T> Located<T> {
    pub fn new(data: T, loc: Location) -> Self {
        Self { data, loc }
    }
}
