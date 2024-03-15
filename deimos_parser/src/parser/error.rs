use deimos_ast::Location;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum ParseErrorKind {
    UnexpectedEOF,
}
impl Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::UnexpectedEOF => "Unexpected end of file",
            }
        )
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub loc: Location,
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {} {}", self.kind, self.loc)
    }
}
impl Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;
