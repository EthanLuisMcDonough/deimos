use deimos_ast::*;
use mips_builder::{FloatRegister, MipsBuilder, Register};

use super::temp::{AccessMode, ExprTemp, RegisterBank};
use super::value::codegen_index_ref;
use crate::error::{ValidationError, ValidationResult};
use crate::expr::unary::codegen_deref;

/// Scaffold function for + and -
fn arith_ptr_num_expr(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    bin_op: BinaryOp,
    loc: Location,
    i32_fnc: impl FnOnce(&mut MipsBuilder, Register, Register),
    u32_fnc: impl FnOnce(&mut MipsBuilder, Register, Register),
    f32_fnc: impl FnOnce(&mut MipsBuilder, FloatRegister, FloatRegister),
) -> ValidationResult<ExprTemp> {
    match (left.type_tuple(), right.type_tuple()) {
        // word ptr + int
        (
            (PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::F32, 1..),
            (PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::U8, 0),
        ) => {
            let ptr_reg = right.register.get_word()?;
            let int_reg = left.register.get_word()?;
            ptr_reg.use_reg(b, 0, AccessMode::Read, |b, r2| {
                // Multiply right value by 4
                b.shift_logical_left(r2, r2, 2);
                // Regular addition
                int_reg.use_reg(b, 1, AccessMode::ReadWrite, |b, r1| {
                    u32_fnc(b, r1, r2);
                })
            });
        }
        // Unsigned + unsigned or u8 ptr + int
        (
            (PrimitiveType::U8, 1..),
            (PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::U8, 0),
        )
        | ((PrimitiveType::U32, 0), (PrimitiveType::U32, 0))
        | ((PrimitiveType::U8, 0), (PrimitiveType::U8, 0)) => {
            let left_reg = left.register.get_word()?;
            let right_reg = right.register.get_word()?;
            left_reg.use_reg(b, 0, AccessMode::ReadWrite, |b, r1| {
                right_reg.use_reg(b, 1, AccessMode::Read, |b, r2| {
                    u32_fnc(b, r1, r2);
                })
            });
        }
        // Floating addition
        ((PrimitiveType::F32, 0), (PrimitiveType::F32, 0)) => {
            let left_reg = left.register.get_float()?;
            let right_reg = right.register.get_float()?;
            left_reg.use_reg(b, 0, AccessMode::ReadWrite, |b, f1| {
                right_reg.use_reg(b, 1, AccessMode::Read, |b, f2| {
                    f32_fnc(b, f1, f2);
                })
            });
        }
        // Addition
        ((PrimitiveType::I32, 0), (PrimitiveType::I32, 0)) => {
            let left_reg = left.register.get_word()?;
            let right_reg = right.register.get_word()?;
            left_reg.use_reg(b, 0, AccessMode::ReadWrite, |b, r1| {
                right_reg.use_reg(b, 1, AccessMode::Read, |b, r2| {
                    i32_fnc(b, r1, r2);
                })
            });
        }
        _ => {
            return Err(ValidationError::InvalidBinary(bin_op, loc));
        }
    }
    reg_bank.free_reg(right.register);
    Ok(left)
}

pub fn codegen_add(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    arith_ptr_num_expr(
        b,
        reg_bank,
        left,
        right,
        BinaryOp::Add,
        loc,
        |b, reg1, reg2| {
            b.add_i32(reg1, reg1, reg2);
        },
        |b, reg1, reg2| {
            b.add_u32(reg1, reg1, reg2);
        },
        |b, reg1, reg2| {
            b.add_f32(reg1, reg1, reg2);
        },
    )
}

pub fn codegen_sub(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    arith_ptr_num_expr(
        b,
        reg_bank,
        left,
        right,
        BinaryOp::Sub,
        loc,
        |b, reg1, reg2| {
            b.sub_i32(reg1, reg1, reg2);
        },
        |b, reg1, reg2| {
            b.sub_u32(reg1, reg1, reg2);
        },
        |b, reg1, reg2| {
            b.sub_f32(reg1, reg1, reg2);
        },
    )
}

/// Scaffold for * and /
fn arith_num_expr(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    bin_op: BinaryOp,
    loc: Location,
    word_fnc: impl FnOnce(&mut MipsBuilder, Register, Register),
    f32_fnc: impl FnOnce(&mut MipsBuilder, FloatRegister, FloatRegister),
) -> ValidationResult<ExprTemp> {
    match (left.type_tuple(), right.type_tuple()) {
        ((PrimitiveType::F32, 0), (PrimitiveType::F32, 0)) => {
            let left_reg = left.register.get_float()?;
            let right_reg = right.register.get_float()?;
            left_reg.use_reg(b, 0, AccessMode::ReadWrite, |b, f1| {
                right_reg.use_reg(b, 1, AccessMode::Read, |b, f2| {
                    f32_fnc(b, f1, f2);
                })
            });
        }
        ((ty1, 0), (ty2, 0)) if ty1 == ty2 => {
            let left_reg = left.register.get_word()?;
            let right_reg = right.register.get_word()?;
            left_reg.use_reg(b, 0, AccessMode::ReadWrite, |b, r1| {
                right_reg.use_reg(b, 1, AccessMode::Read, |b, r2| {
                    word_fnc(b, r1, r2);
                })
            });
        }
        _ => {
            return Err(ValidationError::InvalidBinary(bin_op, loc));
        }
    }
    reg_bank.free_reg(right.register);
    Ok(left)
}

pub fn codegen_mult(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    arith_num_expr(
        b,
        reg_bank,
        left,
        right,
        BinaryOp::Mult,
        loc,
        |b, r1, r2| {
            b.mul_i32(r1, r1, r2);
        },
        |b, f1, f2| {
            b.mul_f32(f1, f1, f2);
        },
    )
}

pub fn codegen_div(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    arith_num_expr(
        b,
        reg_bank,
        left,
        right,
        BinaryOp::Div,
        loc,
        |b, r1, r2| {
            b.div_i32(r1, r1, r2);
        },
        |b, f1, f2| {
            b.div_f32(f1, f1, f2);
        },
    )
}

pub fn codegen_mod(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    match (left.type_tuple(), right.type_tuple()) {
        // Mod only accepts matching int types
        ((PrimitiveType::I32, 0), (PrimitiveType::I32, 0))
        | ((PrimitiveType::U32, 0), (PrimitiveType::U32, 0))
        | ((PrimitiveType::U8, 0), (PrimitiveType::U8, 0)) => {
            let left_reg = left.register.get_word()?;
            let right_reg = right.register.get_word()?;
            left_reg.use_reg(b, 0, AccessMode::ReadWrite, |b, r1| {
                right_reg.use_reg(b, 1, AccessMode::Read, |b, r2| {
                    b.mod_i32(r1, r1, r2);
                })
            });
        }
        _ => {
            return Err(ValidationError::InvalidBinary(BinaryOp::Mod, loc));
        }
    }
    reg_bank.free_reg(right.register);
    Ok(left)
}

pub fn codegen_index_access(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    let reference_expr = codegen_index_ref(b, reg_bank, left, right, loc)?;
    codegen_deref(b, reg_bank, reference_expr, loc)
}

pub fn codegen_binary(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    op: Located<BinaryOp>,
) -> ValidationResult<ExprTemp> {
    use super::logic::*;
    match op.data {
        BinaryOp::Add => codegen_add(b, reg_bank, left, right, op.loc),
        BinaryOp::Sub => codegen_sub(b, reg_bank, left, right, op.loc),
        BinaryOp::Mult => codegen_mult(b, reg_bank, left, right, op.loc),
        BinaryOp::Div => codegen_div(b, reg_bank, left, right, op.loc),
        BinaryOp::Mod => codegen_mod(b, reg_bank, left, right, op.loc),
        BinaryOp::And => codgen_logic_and(b, reg_bank, left, right, op.loc),
        BinaryOp::Or => codgen_logic_or(b, reg_bank, left, right, op.loc),
        BinaryOp::Equal => codgen_logic_eq(b, reg_bank, left, right, op.loc),
        BinaryOp::NotEq => codgen_logic_not_eq(b, reg_bank, left, right, op.loc),
        BinaryOp::LessThan => codgen_logic_less_than(b, reg_bank, left, right, op.loc),
        BinaryOp::LessThanEq => codgen_logic_less_than_eq(b, reg_bank, left, right, op.loc),
        BinaryOp::GreaterThan => codgen_logic_greater_than(b, reg_bank, left, right, op.loc),
        BinaryOp::GreaterThanEq => codgen_logic_greater_than_eq(b, reg_bank, left, right, op.loc),
        BinaryOp::IndexAccess => codegen_index_access(b, reg_bank, left, right, op.loc),
    }
}
