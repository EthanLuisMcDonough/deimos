use deimos_ast::Location;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum ValidationErrorKind {
    MismatchedType,
    Redefinition,
    UndefinedIdent,
    NotAFunc,
    ShadowedFuncCall,
    FuncInExpr,
    InvalidMemVarType,
    InvalidStaticVar,
    InvalidLocalInit,
}
impl Display for ValidationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MismatchedType => "Mismatched type",
                Self::Redefinition => "Redefined variable",
                Self::UndefinedIdent => "Undefined identifier",
                Self::NotAFunc => "Value is not a function",
                Self::ShadowedFuncCall => "Call to function shadowed by local variable",
                Self::FuncInExpr => "Functions cannot be referenced inside expressions",
                Self::InvalidMemVarType => "MemVar must have pointer type",
                Self::InvalidStaticVar => "Static var with invalid type or initial expression",
                Self::InvalidLocalInit => "Invalid local variable initializer statement",
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
impl ValidationError {
    pub fn new(kind: ValidationErrorKind, loc: Location) -> Self {
        Self { kind, loc }
    }
}

pub type ValidationResult<T> = Result<T, ValidationError>;
