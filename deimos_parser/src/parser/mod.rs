use super::lexer::*;
use deimos_ast::*;

mod error;
pub use error::*;

pub fn parse(_tokens: Tokens) -> ParseResult<Program> {
    Ok(Program)
}
