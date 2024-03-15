use deimos_ast::Location;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum ValidationErrorKind {
    MismatchedType,
}
impl Display for ValidationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MismatchedType => "Mismatched type",
            }
        )
    }
}

#[derive(Debug)]
pub struct ValidationError {
    kind: ValidationErrorKind,
    loc: Location,
}
impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Semantic error: {} {}", self.kind, self.loc)
    }
}
impl Error for ValidationError {}

pub type ValidationResult<T> = Result<T, ValidationError>;
