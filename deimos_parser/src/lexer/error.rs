use deimos_ast::Location;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum LexErrorKind {
    UnexpectedEOF,
    InvalidNumber,
    InvalidRegister,
    UnexpectedChar(char),
}
impl LexErrorKind {
    pub fn with_loc(self, loc: Location) -> LexError {
        LexError { kind: self, loc }
    }
}
impl Display for LexErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexer error: ")?;
        match self {
            Self::UnexpectedEOF => write!(f, "Unexpected EOF"),
            Self::InvalidNumber => write!(f, "Invalid number"),
            Self::InvalidRegister => write!(f, "Invalid register"),
            Self::UnexpectedChar(c) => write!(f, "Unexpected char '{}'", c),
        }
    }
}

#[derive(Debug)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub loc: Location,
}
impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexer error: {} {}", self.kind, self.loc)
    }
}
impl Error for LexError {}

pub type LexResult<T> = Result<T, LexError>;
