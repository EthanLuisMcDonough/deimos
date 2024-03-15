mod loc;

pub use loc::*;

#[derive(Debug, Default)]
pub struct StringBank {
    pub identifiers: Vec<String>,
    pub strings: Vec<String>,
}

#[derive(Debug)]
pub struct Program;
