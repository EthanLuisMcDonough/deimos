use crate::lexer::{Keyword, Lexeme};
use deimos_ast::{Located, Location, Reg};
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    NoBody,
    NakedExpression(Location),
    InvalidRedefinition(Located<usize>),
    BodyRedefinition(Location),
    InvalidOperation(Location),
    DuplicateRegister(Located<Reg>),
    UnexpectedToken(Located<Lexeme>),
    ReservedWord(Keyword),
    ExpectedRValue(Location),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEOF => write!(f, "Unexpected end of file"),
            Self::UnexpectedToken(t) => write!(f, "Unexpected token {:?} at {}", t.data, t.loc),
            Self::InvalidOperation(l) => write!(f, "Invalid operation at {}", l),
            Self::ReservedWord(k) => write!(f, "Reserved word \"{:?}\"", k),
            Self::NoBody => write!(f, "No program body"),
            Self::InvalidRedefinition(i) => write!(f, "Invalid redefinition at {}", i.loc),
            Self::BodyRedefinition(l) => write!(f, "Redefined body at {}", l),
            Self::DuplicateRegister(r) => write!(f, "Duplicate register {:?} at {}", r.data, r.loc),
            Self::NakedExpression(l) => write!(
                f,
                "\"Naked\" expression without assignment or side effects at {}",
                l
            ),
            Self::ExpectedRValue(l) => write!(f, "Expected RValue at {}", l),
        }
    }
}
impl Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;
