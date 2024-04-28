use deimos_ast::*;
use mips_builder::{FloatRegister, MipsBuilder, Register};

use super::temp::{AccessMode, ExprTemp, OrVirtual, RegisterBank, FLOAT_TEMP};
use crate::error::{ValidationError, ValidationResult};
use crate::names::{GET_FLOAT_BOOL, GET_FLOAT_BOOL_INV};

fn get_condition_bit(b: &mut MipsBuilder, r: OrVirtual<Register>) {
    b.jump_and_link(GET_FLOAT_BOOL);
    r.store_val(b, Register::V0);
}

fn get_condition_bit_inv(b: &mut MipsBuilder, r: OrVirtual<Register>) {
    b.jump_and_link(GET_FLOAT_BOOL_INV);
    r.store_val(b, Register::V0);
}

pub fn codegen_logic_not(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    expr: ExprTemp,
) -> ValidationResult<ExprTemp> {
    let reg = match expr.type_tuple() {
        (PrimitiveType::F32, 0) => {
            let f_reg = expr.register.get_float()?;
            b.const_f32(0.0, FLOAT_TEMP[0]);
            f_reg.use_reg(b, 1, AccessMode::Read, |b, f| {
                b.equals_f32(f, FLOAT_TEMP[0]);
            });

            let cond_reg = reg_bank.get_register();
            get_condition_bit_inv(b, cond_reg);

            reg_bank.free_reg(expr.register);
            cond_reg
        }
        _ => {
            let reg = expr.register.get_word()?;
            reg.use_reg(b, 0, AccessMode::ReadWrite, |b, r| {
                b.set_neq(r, r, Register::Zero);
            });
            reg
        }
    };
    Ok(ExprTemp::new(reg, PrimitiveType::I32))
}

/// Scaffold function for logic binary ops
fn codegen_logic(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    word_fnc: impl FnOnce(&mut MipsBuilder, Register, Register),
    f32_fnc: impl FnOnce(&mut MipsBuilder, FloatRegister, FloatRegister),
    f32_extr: impl FnOnce(&mut MipsBuilder, OrVirtual<Register>),
    loc: Location,
    op: BinaryOp,
) -> ValidationResult<ExprTemp> {
    let reg = match (left.type_tuple(), right.type_tuple()) {
        ((PrimitiveType::F32, 0), (PrimitiveType::F32, 0)) => {
            let l_reg = left.register.get_float()?;
            let r_reg = right.register.get_float()?;
            l_reg.use_reg(b, 0, AccessMode::Read, |b, f1| {
                r_reg.use_reg(b, 1, AccessMode::Read, |b, f2| {
                    f32_fnc(b, f1, f2);
                });
            });
            let result_reg = reg_bank.get_register();
            f32_extr(b, result_reg);
            reg_bank.free_reg(left.register);
            reg_bank.free_reg(right.register);
            result_reg
        }
        ((typ1, 0), (typ2, 0)) if typ1 == typ2 => {
            let l_reg = left.register.get_word()?;
            let r_reg = right.register.get_word()?;
            l_reg.use_reg(b, 0, AccessMode::ReadWrite, |b, r1| {
                r_reg.use_reg(b, 1, AccessMode::Read, |b, r2| {
                    word_fnc(b, r1, r2);
                });
            });
            reg_bank.free_reg(right.register);
            l_reg
        }
        _ => {
            return Err(ValidationError::InvalidBinary(op, loc));
        }
    };
    return Ok(ExprTemp::new(reg, PrimitiveType::I32));
}

pub fn codgen_logic_eq(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    codegen_logic(
        b,
        reg_bank,
        left,
        right,
        |b, r1, r2| {
            b.set_eq(r1, r1, r2);
        },
        |b, f1, f2| {
            b.equals_f32(f1, f2);
        },
        get_condition_bit,
        loc,
        BinaryOp::Equal,
    )
}

pub fn codgen_logic_not_eq(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    codegen_logic(
        b,
        reg_bank,
        left,
        right,
        |b, r1, r2| {
            b.set_neq(r1, r1, r2);
        },
        |b, f1, f2| {
            b.equals_f32(f1, f2);
        },
        get_condition_bit_inv,
        loc,
        BinaryOp::NotEq,
    )
}

pub fn codgen_logic_greater_than(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    codegen_logic(
        b,
        reg_bank,
        left,
        right,
        |b, r1, r2| {
            b.set_gt(r1, r1, r2);
        },
        |b, f1, f2| {
            b.less_than_or_eq_f32(f1, f2);
        },
        get_condition_bit_inv,
        loc,
        BinaryOp::GreaterThan,
    )
}

pub fn codgen_logic_greater_than_eq(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    codegen_logic(
        b,
        reg_bank,
        left,
        right,
        |b, r1, r2| {
            b.set_ge(r1, r1, r2);
        },
        |b, f1, f2| {
            b.less_than_f32(f1, f2);
        },
        get_condition_bit_inv,
        loc,
        BinaryOp::GreaterThanEq,
    )
}

pub fn codgen_logic_less_than(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    codegen_logic(
        b,
        reg_bank,
        left,
        right,
        |b, r1, r2| {
            b.set_lt(r1, r1, r2);
        },
        |b, f1, f2| {
            b.less_than_f32(f1, f2);
        },
        get_condition_bit,
        loc,
        BinaryOp::LessThan,
    )
}

pub fn codgen_logic_less_than_eq(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    codegen_logic(
        b,
        reg_bank,
        left,
        right,
        |b, r1, r2| {
            b.set_le(r1, r1, r2);
        },
        |b, f1, f2| {
            b.less_than_or_eq_f32(f1, f2);
        },
        get_condition_bit,
        loc,
        BinaryOp::LessThanEq,
    )
}
