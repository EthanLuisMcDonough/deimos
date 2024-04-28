use super::ExprTemp;
use crate::error::*;

use deimos_ast::*;
use mips_builder::{FloatRegister, MipsBuilder, Register};

pub fn codegen_print_val(b: &mut MipsBuilder, expr_val: ExprTemp) -> ValidationResult<()> {
    // Load value into correct register
    match expr_val.type_tuple() {
        (PrimitiveType::F32, 0) => {
            let expr_reg = expr_val.register.get_float()?;
            expr_reg.load_to(b, FloatRegister::F12);
        }
        (PrimitiveType::U8, 0) => {
            let expr_reg = expr_val.register.get_word()?;
            expr_reg.load_byte_to(b, Register::A0);
        }
        _ => {
            let expr_reg = expr_val.register.get_word()?;
            expr_reg.load_to(b, Register::A0);
        }
    }

    // Call correct syscall
    let opcode = match expr_val.type_tuple() {
        (PrimitiveType::F32, 0) => 2,
        (PrimitiveType::U8, 0) => 11,
        (PrimitiveType::U8, 1) => 4,
        (PrimitiveType::U32, 0) => 36,
        (PrimitiveType::I32, 0) => 1,
        (_, 1..) => 34,
    };
    b.add_syscall(opcode);

    Ok(())
}
