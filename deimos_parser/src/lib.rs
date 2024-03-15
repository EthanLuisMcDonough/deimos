use deimos_ast::*;

mod error;
pub mod lexer;
mod parser;

pub use error::*;
pub use lexer::lex;
pub use parser::parse;

pub fn validate(_p: &Program) -> ValidationResult<()> {
    Ok(())
}
