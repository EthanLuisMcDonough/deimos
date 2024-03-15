use deimos_ast::*;

mod error;
pub mod lexer;

pub use error::*;
pub use lexer::lex;

pub fn parse(_tokens: lexer::Tokens) -> ParseResult<Program> {
    Ok(Program)
}

pub fn validate(_p: &Program) -> ValidationResult<()> {
    Ok(())
}
