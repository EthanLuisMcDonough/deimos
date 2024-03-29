use crate::lexer::{Keyword, Lexeme};
use deimos_ast::{Located, Location};
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    InvalidOperation(Location),
    UnexpectedToken(Located<Lexeme>),
    ReservedWord(Keyword),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEOF => write!(f, "Unexpected end of file"),
            Self::UnexpectedToken(t) => write!(f, "Unexpected token {:?} at {}", t.data, t.loc),
            Self::InvalidOperation(l) => write!(f, "Invalid operation at {}", l),
            Self::ReservedWord(k) => write!(f, "Reserved word \"{:?}\"", k),
        }
    }
}
impl Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;
