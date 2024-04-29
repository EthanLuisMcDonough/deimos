use deimos_ast::{BinaryOp, ControlBreak, Identifier, Location, Reg, UnaryOp};
use mips_builder::{FloatRegister, Register};
use std::error::Error;
use std::fmt::Display;

use crate::expr::temp::ExprType;

#[derive(Debug)]
pub enum ValidationError {
    MismatchedType(Location),
    Redefinition(Location),
    UndefinedIdent(Location),
    NotAFunc(Location),
    ShadowedFuncCall(Location),
    FuncInExpr(Location),
    InvalidMemVarType(Location),
    InvalidStaticVar(Location),
    InvalidLocalInit(Location),
    InternalFloatReg(FloatRegister),
    InternalIntReg(Register),
    InvalidUnary(UnaryOp, Location),
    InvalidBinary(BinaryOp, Location),
    ArrayReference(Location),
    MemReference(Location),
    InvalidRValType(Location),
    InvalidLValType(Location),
    InvalidRegTransfer(Identifier, Reg),
    InvalidArgCount(Location),
    InvalidArgType(Location, usize, ExprType),
    FloatInCondition(Location),
    InvalidControlFlow(Location, ControlBreak),
}
impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MismatchedType(loc) => write!(f, "Mismatched type {}", loc),
            Self::Redefinition(loc) => write!(f, "Redefined variable {}", loc),
            Self::UndefinedIdent(loc) => write!(f, "Undefined identifier {}", loc),
            Self::NotAFunc(loc) => write!(f, "Value is not a function {}", loc),
            Self::ShadowedFuncCall(loc) => {
                write!(f, "Call to function shadowed by local variable {}", loc)
            }
            Self::FuncInExpr(loc) => write!(
                f,
                "Functions cannot be referenced inside expressions {}",
                loc
            ),
            Self::InvalidMemVarType(loc) => write!(f, "MemVar must have pointer type {}", loc),
            Self::InvalidStaticVar(loc) => {
                write!(
                    f,
                    "Static var with invalid type or initial expression {}",
                    loc
                )
            }
            Self::InvalidLocalInit(loc) => {
                write!(f, "Invalid local variable initializer statement {}", loc)
            }
            Self::InternalFloatReg(float_reg) => {
                write!(
                    f,
                    "Internal error: int value stored in float register {}",
                    float_reg
                )
            }
            Self::InternalIntReg(reg) => {
                write!(
                    f,
                    "Internal error: float value stored in int register {}",
                    reg
                )
            }
            Self::InvalidUnary(unary_op, loc) => {
                write!(f, "Invalid unary operation {:?} at {}", unary_op, loc)
            }
            Self::InvalidBinary(bin_op, loc) => {
                write!(f, "Invalid binary operation {:?} at {}", bin_op, loc)
            }
            Self::ArrayReference(loc) => {
                write!(
                    f,
                    concat![
                        "Can't reference array at {}, array ident is pointer",
                        "to area on stack. No reference value is stored during execution"
                    ],
                    loc
                )
            }
            Self::MemReference(loc) => {
                write!(f, "Can't reference mem addr at {}", loc)
            }
            Self::InvalidRValType(loc) => {
                write!(f, "RValue at {} has invalid type", loc)
            }
            Self::InvalidLValType(loc) => {
                write!(f, "LValue at {} has mismatching type", loc)
            }
            Self::InvalidRegTransfer(ident, reg) => {
                write!(
                    f,
                    "Ident at {} doesn't correspond with {:?}",
                    ident.loc, reg
                )
            }
            Self::InvalidArgCount(loc) => {
                write!(
                    f,
                    "Function called with incorrect number of arguments at {}",
                    loc
                )
            }
            Self::InvalidArgType(loc, index, expected) => write!(
                f,
                "Invalid argument type at position {} in call at {}. Expected {:?}",
                index, loc, expected,
            ),
            Self::FloatInCondition(loc) => {
                write!(
                    f,
                    "Floating point value found in condition at {}. Condition must be int",
                    loc
                )
            }
            Self::InvalidControlFlow(loc, b) => {
                write!(f, "Invalid {:?} at {}", b, loc)
            }
        }
    }
}

impl Error for ValidationError {}
pub type ValidationResult<T> = Result<T, ValidationError>;
