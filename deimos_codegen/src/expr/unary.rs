use deimos_ast::*;
use mips_builder::{MipsBuilder, Register};

use super::logic::codegen_logic_not;
use super::temp::{AccessMode, ExprTemp, RegisterBank};
use super::value::{codegen_ident_ref, codegen_index_ref};
use crate::error::{ValidationError, ValidationResult};
use crate::scope::Scope;

pub fn codegen_negation(
    b: &mut MipsBuilder,
    expr: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    match expr.type_tuple() {
        (PrimitiveType::F32, 0) => {
            expr.register
                .get_float()?
                .use_reg(b, 0, AccessMode::ReadWrite, |b, f| {
                    b.neg_f32(f, f);
                });
        }
        (PrimitiveType::I32, 0) => {
            expr.register
                .get_word()?
                .use_reg(b, 0, AccessMode::ReadWrite, |b, r| {
                    b.sub_i32(r, Register::Zero, r);
                });
        }
        _ => {
            return Err(ValidationError::InvalidUnary(UnaryOp::Negation, loc));
        }
    }
    Ok(expr)
}

pub fn codegen_deref(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    expr: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    match expr.type_tuple() {
        (PrimitiveType::F32, 1) => {
            let float_reg = reg_bank.get_float_reg();
            let ptr_reg = expr.register.get_word()?;
            ptr_reg.use_reg(b, 0, AccessMode::Read, |b, r| {
                float_reg.use_reg(b, 0, AccessMode::ReadWrite, |b, f| {
                    b.load_f32(f, r);
                });
            });
            reg_bank.free_reg(expr.register);
            Ok(ExprTemp::new(float_reg, PrimitiveType::F32))
        }
        (_, 1..) => {
            expr.register
                .get_word()?
                .use_reg(b, 0, AccessMode::ReadWrite, |b, r| {
                    b.load_word(r, r);
                });
            Ok(ExprTemp::new(
                expr.register,
                expr.computed_type.deref_type(),
            ))
        }
        _ => Err(ValidationError::InvalidUnary(UnaryOp::Deref, loc)),
    }
}

pub fn codegen_ref(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    s: &Scope,
    expr: &Expression,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    match expr {
        Expression::Identifier(ident) => codegen_ident_ref(b, reg_bank, s, *ident),
        Expression::Binary {
            left,
            right,
            op:
                Located {
                    data: BinaryOp::IndexAccess,
                    loc,
                },
        } => {
            let left_expr = super::codegen_expr(b, left, s, reg_bank)?;
            let right_expr = super::codegen_expr(b, right, s, reg_bank)?;
            codegen_index_ref(b, reg_bank, left_expr, right_expr, *loc)
        }
        _ => Err(ValidationError::InvalidUnary(UnaryOp::Reference, loc)),
    }
}

pub fn codegen_unary(
    b: &mut MipsBuilder,
    expr: &Expression,
    s: &Scope,
    reg_bank: &mut RegisterBank,
    op: Located<UnaryOp>,
) -> ValidationResult<ExprTemp> {
    match op.data {
        UnaryOp::Reference => codegen_ref(b, reg_bank, s, expr, op.loc),
        UnaryOp::LogicNot => {
            let expr_val = super::codegen_expr(b, expr, s, reg_bank)?;
            codegen_logic_not(b, reg_bank, expr_val, op.loc)
        }
        UnaryOp::Deref => {
            let expr_val = super::codegen_expr(b, expr, s, reg_bank)?;
            codegen_deref(b, reg_bank, expr_val, op.loc)
        }
        UnaryOp::Negation => {
            let expr_val = super::codegen_expr(b, expr, s, reg_bank)?;
            codegen_negation(b, expr_val, op.loc)
        }
    }
}
