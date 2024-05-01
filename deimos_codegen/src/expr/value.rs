use deimos_ast::*;
use mips_builder::{MipsBuilder, Register};

use super::binary::codegen_add;
use super::temp::{AccessMode, ExprRegister, ExprTemp, ExprType, OrVirtual, RegisterBank};
use crate::error::{ValidationError, ValidationResult};
use crate::names::get_str_name;
use crate::scope::{Scope, ValLocation};

/// Codegen for references to identifiers
pub fn codegen_ident(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    s: &Scope,
    ident: Identifier,
) -> ValidationResult<ExprTemp> {
    let addr = s.get_var(ident)?;
    let expr_type = ExprType::from(addr.val.clone());
    if let ValLocation::RawAddr(addr) = &addr.loc {
        let register = reg_bank.get_register();
        register.use_reg(b, 0, AccessMode::Write, |b, r| {
            b.const_word(*addr, r);
        });
        return Ok(ExprTemp::new(register, expr_type));
    }
    let reg: ExprRegister = match &addr.val {
        DeclType::Array { .. } => {
            let register = reg_bank.get_register();
            register.use_reg(b, 0, AccessMode::Write, |b, r| {
                b.load_addr(r, addr.loc);
            });
            register.into()
        }
        DeclType::Param(Located { data, .. }) => match (data.param_type.data, data.indirection) {
            (PrimitiveType::F32, 0) => {
                let register = reg_bank.get_float_reg();
                register.use_reg(b, 0, AccessMode::Write, |b, r| {
                    b.load_f32(r, addr.loc);
                });
                register.into()
            }
            (PrimitiveType::U8, 0) => {
                let register = reg_bank.get_register();
                register.use_reg_byte(b, 0, AccessMode::Write, |b, r| {
                    b.load_byte(r, addr.loc);
                });
                register.into()
            }
            _ => {
                let register = reg_bank.get_register();
                register.use_reg(b, 0, AccessMode::Write, |b, r| {
                    b.load_word(r, addr.loc);
                });
                register.into()
            }
        },
    };
    Ok(ExprTemp::new(reg, expr_type))
}

fn const_word(b: &mut MipsBuilder, reg_bank: &mut RegisterBank, word: u32) -> OrVirtual<Register> {
    let register = reg_bank.get_register();
    register.use_reg(b, 0, AccessMode::Write, |b, r| {
        b.const_word(word, r);
    });
    register
}

/// Codegen for constant values
pub fn codegen_const(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    val: PrimitiveValue,
) -> ExprTemp {
    match val {
        PrimitiveValue::Float(f) => {
            let register = reg_bank.get_float_reg();
            register.use_reg(b, 0, AccessMode::Write, |b, r| {
                b.const_f32(f, r);
            });
            ExprTemp::new(register, PrimitiveType::F32)
        }
        PrimitiveValue::Int(i) => {
            ExprTemp::new(const_word(b, reg_bank, i as u32), PrimitiveType::I32)
        }
        PrimitiveValue::Unsigned(i) => {
            ExprTemp::new(const_word(b, reg_bank, i), PrimitiveType::U32)
        }
        PrimitiveValue::String(str_id) => {
            let register = reg_bank.get_register();
            register.use_reg(b, 0, AccessMode::Write, |b, r| {
                let str_name = get_str_name(str_id);
                b.load_addr(r, str_name);
            });
            ExprTemp::new(
                register,
                ExprType {
                    base: PrimitiveType::U8,
                    indirection: 1,
                },
            )
        }
    }
}

pub fn codegen_cast(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    expr: ExprTemp,
    typ: ExprType,
) -> ValidationResult<ExprTemp> {
    // We'll treat u8 as a word because the registers are words
    match (
        (expr.computed_type.base, expr.computed_type.indirection),
        (typ.base, typ.indirection),
    ) {
        // Float to int/ptr
        (
            (PrimitiveType::F32, 0),
            (PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::U8, _)
            | (PrimitiveType::F32, 1..),
        ) => {
            let register = reg_bank.get_register();
            let dest_reg = expr.register.get_float()?;
            dest_reg.use_reg(b, 0, AccessMode::Read, |b, float_reg| {
                b.cast_from_f32(float_reg, float_reg);
                register.use_reg(b, 0, AccessMode::Write, |b, r| {
                    b.mov_from_f32(r, float_reg);
                });
            });
            reg_bank.free_reg(expr.register);
            Ok(ExprTemp::new(register, typ))
        }
        // Int/ptr to float
        (
            (PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::U8, _)
            | (PrimitiveType::F32, 1..),
            (PrimitiveType::F32, 0),
        ) => {
            let register = reg_bank.get_float_reg();
            let int_reg = expr.register.get_word()?;
            int_reg.use_reg(b, 0, AccessMode::Read, |b, reg| {
                register.use_reg(b, 0, AccessMode::Write, |b, float_reg| {
                    b.mov_to_f32(float_reg, reg);
                    b.cast_to_f32(float_reg, float_reg);
                });
            });
            reg_bank.free_reg(expr.register);
            Ok(ExprTemp {
                register: register.into(),
                computed_type: PrimitiveType::F32.into(),
            })
        }
        // No actual conversion required
        _ => Ok(ExprTemp {
            register: expr.register,
            computed_type: typ,
        }),
    }
}

/// Gets reference to identifier
pub fn codegen_ident_ref(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    s: &Scope,
    ident: Identifier,
) -> ValidationResult<ExprTemp> {
    let reg = reg_bank.get_register();
    let local_var = s.get_var(ident)?;
    match local_var.val {
        DeclType::Array {
            array_type: Located { loc, .. },
            ..
        } => Err(ValidationError::ArrayReference(loc)),
        DeclType::Param(p) => {
            if let ValLocation::RawAddr(_) = local_var.loc {
                Err(ValidationError::MemReference(p.loc))
            } else {
                reg.use_reg(b, 0, AccessMode::Write, |b, r| {
                    b.load_addr(r, local_var.loc);
                });
                let expr_type = ExprType::from(p.data).ref_type();
                Ok(ExprTemp::new(reg, expr_type))
            }
        }
    }
}

/// Calculate pointer for index access (value + index)
pub fn codegen_index_ref(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    value: ExprTemp,
    index: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    match (value.type_tuple(), index.type_tuple()) {
        // value must be ptr and index must be int
        ((_, 1..), (PrimitiveType::U8 | PrimitiveType::I32 | PrimitiveType::U32, 0)) => {
            codegen_add(b, reg_bank, value, index, loc)
        }
        _ => Err(ValidationError::InvalidBinary(BinaryOp::IndexAccess, loc)),
    }
}
