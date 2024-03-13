use deimos_ast::*;

mod error;
pub use error::*;

pub fn parse_file(_source: String) -> ParseResult<Program> {
    Ok(Program)
}

pub fn validate(_p: &Program) -> ValidationResult<()> {
    Ok(())
}
