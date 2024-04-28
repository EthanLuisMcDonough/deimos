use crate::error::*;
use crate::{expr::binary::codegen_binary, scope::Scope};
use deimos_ast::*;
use mips_builder::MipsBuilder;

mod binary;
mod logic;
pub mod print;
pub mod rvalue;
pub mod temp;
mod unary;
mod value;

pub use self::temp::{ExprTemp, RegisterBank};

pub fn codegen_expr(
    b: &mut MipsBuilder,
    expr: &Expression,
    s: &Scope,
    reg_bank: &mut RegisterBank,
) -> ValidationResult<ExprTemp> {
    match expr {
        Expression::Unary { operand, op } => unary::codegen_unary(b, operand, s, reg_bank, *op),
        Expression::Binary { left, right, op } => {
            let left_expr = codegen_expr(b, left, s, reg_bank)?;
            let right_expr = codegen_expr(b, right, s, reg_bank)?;
            codegen_binary(b, reg_bank, left_expr, right_expr, *op)
        }
        Expression::Identifier(ident) => value::codegen_ident(b, reg_bank, s, *ident),
        Expression::Cast { value, cast_type } => {
            let expr_val: ExprTemp = codegen_expr(b, &value, s, reg_bank)?;
            value::codegen_cast(b, reg_bank, expr_val, cast_type.clone().into())
        }
        Expression::Primitive(p) => Ok(value::codegen_const(b, reg_bank, p.data)),
    }
}
