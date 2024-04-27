use super::temp::{AccessMode, RegisterBank};
use super::value::{codegen_ident_ref, codegen_index_ref};
use super::{codegen_expr, ExprTemp};
use crate::error::*;
use crate::scope::Scope;
use deimos_ast::*;
use mips_builder::MipsBuilder;

fn codegen_rval(
    b: &mut MipsBuilder,
    scope: &Scope,
    reg_bank: &mut RegisterBank,
    rval: &Located<RValue>,
) -> ValidationResult<ExprTemp> {
    match &rval.data {
        RValue::Identifier(ident) => codegen_ident_ref(b, reg_bank, scope, *ident),
        RValue::Deref(expr) => codegen_expr(b, expr, scope, reg_bank),
        RValue::Index { array, value } => {
            let value_expr = codegen_expr(b, array, scope, reg_bank)?;
            let index_expr = codegen_expr(b, value, scope, reg_bank)?;
            codegen_index_ref(b, reg_bank, value_expr, index_expr, rval.loc)
        }
    }
}

pub fn codegen_assignment(
    b: &mut MipsBuilder,
    scope: &Scope,
    assignment: &Assignment,
) -> ValidationResult<()> {
    let mut bank = RegisterBank::default();
    let expr_val = codegen_expr(b, &assignment.lvalue, scope, &mut bank)?;
    let rval = codegen_rval(b, scope, &mut bank, &assignment.rvalue)?;

    let rtype = rval.computed_type;
    if rtype.indirection == 0 {
        return Err(ValidationError::InvalidRValType(assignment.rvalue.loc));
    }

    let ltype = expr_val.computed_type;
    if rtype.deref_type() != ltype {
        return Err(ValidationError::InvalidLValType(assignment.rvalue.loc));
    }

    match rval.type_tuple() {
        (PrimitiveType::F32, 0) => rval.register.use_word(b, 0, AccessMode::Read, |b, fr| {
            expr_val
                .register
                .use_float(b, 1, AccessMode::Read, |b, fl| {
                    b.save_f32(fl, fr);
                })
        })?,
        (PrimitiveType::U8, 0) => rval.register.use_word(b, 0, AccessMode::Read, |b, fr| {
            expr_val.register.use_byte(b, 1, AccessMode::Read, |b, fl| {
                b.save_byte(fl, fr);
            })
        })?,
        _ => rval.register.use_word(b, 0, AccessMode::Read, |b, fr| {
            expr_val.register.use_word(b, 1, AccessMode::Read, |b, fl| {
                b.save_byte(fl, fr);
            })
        })?,
    }
}
