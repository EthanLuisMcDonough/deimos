use crate::lexer::Lexeme;
use deimos_ast::Located;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(Located<Lexeme>),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEOF => write!(f, "Unexpected end of file"),
            Self::UnexpectedToken(t) => write!(f, "Unexpected token {:?} at {}", t.data, t.loc),
        }
    }
}
impl Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;
